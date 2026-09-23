//! Stub poll OS (Linux / autres).

use crate::error::CursorError;

pub(crate) fn poll(_display_w: u32, _display_h: u32) -> Result<(f32, f32), CursorError> {
    Err(CursorError::UnsupportedPlatform)
}
