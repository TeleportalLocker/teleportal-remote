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
        .map(|(index, d)| {
            let display_id = d.display_id();
            let (origin_x, origin_y) = cg_display_origin(display_id);
            DisplayInfo {
                id: DisplayId(display_id),
                index,
                name: format!("Display {display_id}"),
                origin_x,
                origin_y,
                width: d.width(),
                height: d.height(),
            }
        })
        .collect())
}

fn cg_display_origin(display_id: u32) -> (i32, i32) {
    use core_graphics::display::CGDisplay;
    let bounds = CGDisplay::new(display_id).bounds();
    (
        bounds.origin.x.round() as i32,
        bounds.origin.y.round() as i32,
    )
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
