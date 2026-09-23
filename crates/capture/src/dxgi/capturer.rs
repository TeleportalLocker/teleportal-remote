//! Capturer DXGI Desktop Duplication.

use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use tracing::{debug, warn};
use windows::core::Interface;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_CPU_ACCESS_READ,
    D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource, DXGI_ERROR_ACCESS_LOST,
    DXGI_ERROR_WAIT_TIMEOUT, DXGI_OUTDUPL_FRAME_INFO,
};

use crate::dxgi::device::{create_device_for_adapter, map_win, D3dDevice};
use crate::dxgi::output::{list_outputs, open_output, OutputRef};
use crate::error::CaptureError;
use crate::traits::Capturer;
use crate::types::{CaptureConfig, DisplayInfo, Frame, PixelFormat};

/// Capturer Windows basé sur DXGI Desktop Duplication.
pub struct DxgiCapturer {
    config: CaptureConfig,
    output_ref: OutputRef,
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    duplication: IDXGIOutputDuplication,
    staging: Option<ID3D11Texture2D>,
    staging_desc: D3D11_TEXTURE2D_DESC,
}

impl Capturer for DxgiCapturer {
    fn displays() -> Result<Vec<DisplayInfo>, CaptureError> {
        Ok(list_outputs()?.into_iter().map(|o| o.info).collect())
    }

    fn start(config: CaptureConfig) -> Result<Self, CaptureError> {
        config.validate()?;
        let outputs = list_outputs()?;
        let output_ref = outputs
            .into_iter()
            .find(|o| o.info.index == config.display_index)
            .ok_or(CaptureError::DisplayNotFound(config.display_index))?;

        let (adapter, output1) = open_output(output_ref.adapter_index, output_ref.output_index)?;
        let D3dDevice { device, context } = create_device_for_adapter(&adapter)?;
        let duplication = duplicate(&output1, &device)?;

        Ok(Self {
            config,
            output_ref,
            device,
            context,
            duplication,
            staging: None,
            staging_desc: D3D11_TEXTURE2D_DESC::default(),
        })
    }

    fn grab(&mut self) -> Result<Option<Frame>, CaptureError> {
        match self.grab_once() {
            Ok(frame) => Ok(frame),
            Err(CaptureError::AccessLost) => {
                warn!("DXGI access lost — recreating duplication");
                self.recreate_duplication()?;
                self.grab_once()
            }
            Err(e) => Err(e),
        }
    }

    fn stop(self) -> Result<(), CaptureError> {
        debug!(display = %self.output_ref.info.name, "dxgi capturer stopped");
        Ok(())
    }
}

impl DxgiCapturer {
    fn grab_once(&mut self) -> Result<Option<Frame>, CaptureError> {
        let timeout = self.config.frame_timeout_ms();
        let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;

        let acquire = unsafe {
            self.duplication
                .AcquireNextFrame(timeout, &mut frame_info, &mut resource)
        };

        match acquire {
            Ok(()) => {}
            Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => return Ok(None),
            Err(e) if e.code() == DXGI_ERROR_ACCESS_LOST => {
                return Err(CaptureError::AccessLost);
            }
            Err(e) => return Err(map_win(e)),
        }

        let resource = match resource {
            Some(r) => r,
            None => {
                let _ = unsafe { self.duplication.ReleaseFrame() };
                return Ok(None);
            }
        };

        // No desktop update — still must release.
        if frame_info.LastPresentTime == 0 && frame_info.AccumulatedFrames == 0 {
            let _ = unsafe { self.duplication.ReleaseFrame() };
            return Ok(None);
        }

        let result = (|| {
            let texture: ID3D11Texture2D = resource.cast().map_err(map_win)?;
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            unsafe { texture.GetDesc(&mut desc) };

            self.ensure_staging(&desc)?;
            let staging = self
                .staging
                .as_ref()
                .ok_or_else(|| CaptureError::Native("staging texture missing".into()))?;

            unsafe {
                self.context.CopyResource(staging, &texture);
            }

            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            unsafe {
                self.context
                    .Map(staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
            }
            .map_err(map_win)?;

            let width = desc.Width;
            let height = desc.Height;
            let src_pitch = mapped.RowPitch as usize;
            let dst_stride = (width as usize).saturating_mul(4);
            let mut buffer = vec![0u8; dst_stride.saturating_mul(height as usize)];

            unsafe {
                let src = mapped.pData as *const u8;
                for y in 0..height as usize {
                    let src_row = src.add(y * src_pitch);
                    let dst_row = buffer.as_mut_ptr().add(y * dst_stride);
                    std::ptr::copy_nonoverlapping(src_row, dst_row, dst_stride);
                }
            }

            unsafe {
                self.context.Unmap(staging, 0);
            }

            let timestamp_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            Ok(Some(Frame {
                width,
                height,
                stride: dst_stride,
                format: PixelFormat::Bgra8,
                timestamp_ms,
                display_id: self.output_ref.info.id,
                data: Bytes::from(buffer),
            }))
        })();

        let _ = unsafe { self.duplication.ReleaseFrame() };
        result
    }

    fn ensure_staging(&mut self, src: &D3D11_TEXTURE2D_DESC) -> Result<(), CaptureError> {
        let needs_new = self.staging.is_none()
            || self.staging_desc.Width != src.Width
            || self.staging_desc.Height != src.Height
            || self.staging_desc.Format != src.Format;

        if !needs_new {
            return Ok(());
        }

        let mut desc = *src;
        desc.BindFlags = 0;
        desc.MiscFlags = 0;
        desc.Usage = D3D11_USAGE_STAGING;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        desc.SampleDesc = DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        };
        if desc.Format == DXGI_FORMAT(0) {
            desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
        }

        let mut staging: Option<ID3D11Texture2D> = None;
        unsafe { self.device.CreateTexture2D(&desc, None, Some(&mut staging)) }.map_err(map_win)?;

        self.staging = staging;
        self.staging_desc = desc;
        Ok(())
    }

    fn recreate_duplication(&mut self) -> Result<(), CaptureError> {
        let (adapter, output1) =
            open_output(self.output_ref.adapter_index, self.output_ref.output_index)?;
        let D3dDevice { device, context } = create_device_for_adapter(&adapter)?;
        let duplication = duplicate(&output1, &device)?;
        self.device = device;
        self.context = context;
        self.duplication = duplication;
        self.staging = None;
        Ok(())
    }
}

fn duplicate(
    output1: &IDXGIOutput1,
    device: &ID3D11Device,
) -> Result<IDXGIOutputDuplication, CaptureError> {
    unsafe { output1.DuplicateOutput(device) }.map_err(map_win)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_at_least_one_display() {
        let displays = DxgiCapturer::displays().expect("displays");
        assert!(!displays.is_empty());
    }

    #[test]
    fn start_and_grab_does_not_panic() {
        let mut capturer = DxgiCapturer::start(CaptureConfig::default()).expect("start");
        let _ = capturer.grab().expect("grab");
        capturer.stop().expect("stop");
    }
}
