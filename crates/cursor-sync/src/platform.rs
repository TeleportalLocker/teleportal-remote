//! Poll position curseur OS (Host).

use crate::error::CursorError;

#[cfg(windows)]
use crate::windows as os;

#[cfg(target_os = "macos")]
use crate::macos as os;

#[cfg(not(any(windows, target_os = "macos")))]
use crate::stub as os;

/// Lit la position du curseur OS et la normalise sur l’écran primaire.
///
/// # Errors
///
/// Dimensions nulles, plateforme non supportée, ou échec API native.
pub fn poll_os_cursor(display_w: u32, display_h: u32) -> Result<(f32, f32), CursorError> {
    if display_w == 0 || display_h == 0 {
        return Err(CursorError::InvalidDisplay(
            "display dimensions must be >= 1".into(),
        ));
    }
    os::poll(display_w, display_h)
}
