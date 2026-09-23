//! Poll curseur Windows (`GetCursorPos`).

use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

use crate::error::CursorError;
use crate::types::normalize_pixels;

pub(crate) fn poll(display_w: u32, display_h: u32) -> Result<(f32, f32), CursorError> {
    let mut pt = POINT::default();
    unsafe {
        GetCursorPos(&mut pt).map_err(|e| CursorError::Native(e.to_string()))?;
    }
    Ok(normalize_pixels(pt.x, pt.y, display_w, display_h))
}
