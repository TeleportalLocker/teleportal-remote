//! Poll position curseur OS (Host).

use crate::error::CursorError;

#[cfg(windows)]
use crate::windows as os;

#[cfg(target_os = "macos")]
use crate::macos as os;

#[cfg(not(any(windows, target_os = "macos")))]
use crate::stub as os;

/// Lit la position du curseur OS et la normalise sur le display capturé.
///
/// Retourne `Ok(None)` si le curseur est hors du rectangle du display.
///
/// # Errors
///
/// Dimensions nulles, plateforme non supportée, ou échec API native.
pub fn poll_os_cursor(
    origin_x: i32,
    origin_y: i32,
    display_w: u32,
    display_h: u32,
) -> Result<Option<(f32, f32)>, CursorError> {
    if display_w == 0 || display_h == 0 {
        return Err(CursorError::InvalidDisplay(
            "display dimensions must be >= 1".into(),
        ));
    }
    os::poll(origin_x, origin_y, display_w, display_h)
}
