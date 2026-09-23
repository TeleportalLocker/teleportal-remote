//! Poll curseur Windows (`GetCursorPos`).

use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

use crate::error::CursorError;
use crate::types::normalize_in_display;

pub(crate) fn poll(
    origin_x: i32,
    origin_y: i32,
    display_w: u32,
    display_h: u32,
) -> Result<Option<(f32, f32)>, CursorError> {
    let mut pt = POINT::default();
    unsafe {
        GetCursorPos(&mut pt).map_err(|e| CursorError::Native(e.to_string()))?;
    }
    Ok(normalize_in_display(
        pt.x, pt.y, origin_x, origin_y, display_w, display_h,
    ))
}
