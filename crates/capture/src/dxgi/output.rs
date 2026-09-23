//! Enumération des sorties DXGI.

use windows::core::Interface;
use windows::Win32::Graphics::Dxgi::Common::DXGI_MODE_ROTATION;
use windows::Win32::Graphics::Dxgi::{
    IDXGIAdapter1, IDXGIOutput, IDXGIOutput1, DXGI_ERROR_NOT_FOUND, DXGI_OUTPUT_DESC,
};

use crate::dxgi::device::{create_factory, map_win};
use crate::error::CaptureError;
use crate::types::{DisplayId, DisplayInfo};

#[derive(Clone)]
pub(crate) struct OutputRef {
    pub adapter_index: u32,
    pub output_index: u32,
    pub info: DisplayInfo,
}

pub(crate) fn list_outputs() -> Result<Vec<OutputRef>, CaptureError> {
    let factory = create_factory()?;
    let mut displays = Vec::new();
    let mut global_index = 0usize;

    for adapter_index in 0u32.. {
        let adapter: IDXGIAdapter1 = match unsafe { factory.EnumAdapters1(adapter_index) } {
            Ok(a) => a,
            Err(e) if e.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(e) => return Err(map_win(e)),
        };

        for output_index in 0u32.. {
            let output: IDXGIOutput = match unsafe { adapter.EnumOutputs(output_index) } {
                Ok(o) => o,
                Err(e) if e.code() == DXGI_ERROR_NOT_FOUND => break,
                Err(e) => return Err(map_win(e)),
            };

            let desc = unsafe { output.GetDesc() }.map_err(map_win)?;

            let (width, height) = desktop_size(&desc);
            let name = wide_to_string(&desc.DeviceName);
            let id = DisplayId((adapter_index << 16) | output_index);
            let rect = desc.DesktopCoordinates;

            displays.push(OutputRef {
                adapter_index,
                output_index,
                info: DisplayInfo {
                    id,
                    index: global_index,
                    name,
                    origin_x: rect.left,
                    origin_y: rect.top,
                    width,
                    height,
                },
            });
            global_index += 1;
        }
    }

    if displays.is_empty() {
        return Err(CaptureError::NoDisplays);
    }
    Ok(displays)
}

pub(crate) fn open_output(
    adapter_index: u32,
    output_index: u32,
) -> Result<(IDXGIAdapter1, IDXGIOutput1), CaptureError> {
    let factory = create_factory()?;
    let adapter: IDXGIAdapter1 =
        unsafe { factory.EnumAdapters1(adapter_index) }.map_err(map_win)?;
    let output: IDXGIOutput = unsafe { adapter.EnumOutputs(output_index) }.map_err(map_win)?;
    let output1: IDXGIOutput1 = output.cast().map_err(map_win)?;
    Ok((adapter, output1))
}

fn desktop_size(desc: &DXGI_OUTPUT_DESC) -> (u32, u32) {
    let rect = desc.DesktopCoordinates;
    let mut w = (rect.right - rect.left).max(0) as u32;
    let mut h = (rect.bottom - rect.top).max(0) as u32;
    // Swap on 90/270 rotation
    match desc.Rotation {
        DXGI_MODE_ROTATION(2) | DXGI_MODE_ROTATION(4) => std::mem::swap(&mut w, &mut h),
        _ => {}
    }
    (w, h)
}

fn wide_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}
