//! Conversion `CMSampleBuffer` → `Frame` BGRA8.

use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use core_media_rs::cm_sample_buffer::CMSampleBuffer;
use core_video_rs::cv_pixel_buffer::lock::LockTrait;

use crate::error::CaptureError;
use crate::types::{DisplayId, Frame, PixelFormat};

/// Extrait une frame BGRA depuis un sample ScreenCaptureKit.
pub(crate) fn frame_from_sample(
    sample: &CMSampleBuffer,
    display_id: DisplayId,
) -> Result<Option<Frame>, CaptureError> {
    let pixel_buffer = match sample.get_pixel_buffer() {
        Ok(buf) => buf,
        Err(_) => return Ok(None),
    };

    let width = pixel_buffer.get_width();
    let height = pixel_buffer.get_height();
    if width == 0 || height == 0 {
        return Ok(None);
    }

    let src_stride = pixel_buffer.get_bytes_per_row() as usize;
    let dst_stride = (width as usize).saturating_mul(4);

    let guard = pixel_buffer
        .lock()
        .map_err(|e| CaptureError::Native(format!("lock pixel buffer: {e}")))?;
    let src = guard.as_slice();

    let mut buffer = vec![0u8; dst_stride.saturating_mul(height as usize)];
    for y in 0..height as usize {
        let src_off = y.saturating_mul(src_stride);
        let dst_off = y.saturating_mul(dst_stride);
        let end = src_off.saturating_add(dst_stride);
        if end > src.len() {
            return Err(CaptureError::Native(
                "pixel buffer shorter than expected stride".into(),
            ));
        }
        buffer[dst_off..dst_off + dst_stride].copy_from_slice(&src[src_off..end]);
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
        display_id,
        data: Bytes::from(buffer),
    }))
}
