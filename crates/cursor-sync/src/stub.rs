//! Stub poll OS (Linux / autres).

use crate::error::CursorError;

pub(crate) fn poll(
    _origin_x: i32,
    _origin_y: i32,
    _display_w: u32,
    _display_h: u32,
) -> Result<Option<(f32, f32)>, CursorError> {
    Err(CursorError::UnsupportedPlatform)
}
