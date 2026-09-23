//! Permissions Screen Recording et enumération des displays.

use core_graphics::access::ScreenCaptureAccess;
use screencapturekit::shareable_content::SCShareableContent;
use tracing::debug;

use crate::error::CaptureError;
use crate::types::{DisplayId, DisplayInfo};

/// Vérifie (et demande si besoin) l’accès Screen Recording.
pub(crate) fn ensure_permission() -> Result<(), CaptureError> {
    let access = ScreenCaptureAccess;
    if access.preflight() {
        return Ok(());
    }
    debug!("screen recording permission missing — requesting");
    if access.request() {
        return Ok(());
    }
    Err(CaptureError::PermissionDenied)
}

/// Liste les écrans via `SCShareableContent`.
pub(crate) fn list_displays() -> Result<Vec<DisplayInfo>, CaptureError> {
    ensure_permission()?;
    let content = SCShareableContent::get().map_err(map_sck)?;
    let displays = content.displays();
    if displays.is_empty() {
        return Err(CaptureError::NoDisplays);
    }
    Ok(displays
        .into_iter()
        .enumerate()
        .map(|(index, d)| DisplayInfo {
            id: DisplayId(d.display_id()),
            index,
            name: format!("Display {}", d.display_id()),
            width: d.width(),
            height: d.height(),
        })
        .collect())
}

pub(crate) fn map_sck(err: impl std::fmt::Display) -> CaptureError {
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    if lower.contains("permission") || lower.contains("denied") || lower.contains("not authorized")
    {
        CaptureError::PermissionDenied
    } else {
        CaptureError::Native(msg)
    }
}
