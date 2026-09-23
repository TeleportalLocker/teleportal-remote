//! Fenêtre overlay Host transparente (curseur distant).

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tracing::{info, warn};

use crate::session::SessionState;

/// Label de la fenêtre overlay.
pub const OVERLAY_LABEL: &str = "cursor-overlay";

/// Indique si l’état Host nécessite l’overlay bureau.
#[must_use]
pub fn host_needs_overlay(state: &SessionState) -> bool {
    matches!(
        state,
        SessionState::InSession {
            role,
            peer_connected: true,
            ..
        } if role == "host"
    )
}

/// Ouvre ou resynchronise la fenêtre overlay sur le moniteur primaire.
///
/// # Errors
///
/// Échec création fenêtre / moniteur introuvable.
pub fn open_host_overlay(app: &AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(OVERLAY_LABEL) {
        apply_primary_bounds(&existing)?;
        let _ = existing.set_always_on_top(true);
        let _ = existing.set_ignore_cursor_events(true);
        let _ = existing.show();
        return Ok(());
    }

    let (x, y, w, h) = primary_logical_bounds(app)?;
    let window = WebviewWindowBuilder::new(
        app,
        OVERLAY_LABEL,
        WebviewUrl::App("index.html#overlay".into()),
    )
    .title("Teleportal Cursor Overlay")
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .visible(true)
    .focused(false)
    .position(x, y)
    .inner_size(w, h)
    .build()
    .map_err(|e| format!("overlay window: {e}"))?;

    window
        .set_ignore_cursor_events(true)
        .map_err(|e| format!("overlay ignore cursor: {e}"))?;

    info!(x, y, w, h, "host cursor overlay opened");
    Ok(())
}

/// Ferme la fenêtre overlay si elle existe.
pub fn close_host_overlay(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        if let Err(e) = win.close() {
            warn!(error = %e, "failed to close cursor overlay");
        } else {
            info!("host cursor overlay closed");
        }
    }
}

/// Ouvre ou ferme selon l’état session.
pub fn sync_host_overlay(app: &AppHandle, state: &SessionState) {
    if host_needs_overlay(state) {
        if let Err(e) = open_host_overlay(app) {
            warn!(error = %e, "failed to open host overlay");
        }
    } else {
        close_host_overlay(app);
    }
}

fn primary_logical_bounds(app: &AppHandle) -> Result<(f64, f64, f64, f64), String> {
    let monitor = app
        .primary_monitor()
        .map_err(|e| format!("primary_monitor: {e}"))?
        .ok_or_else(|| "aucun moniteur primaire".to_owned())?;
    let scale = monitor.scale_factor().max(0.1);
    let pos = monitor.position();
    let size = monitor.size();
    Ok((
        f64::from(pos.x) / scale,
        f64::from(pos.y) / scale,
        f64::from(size.width) / scale,
        f64::from(size.height) / scale,
    ))
}

fn apply_primary_bounds(window: &tauri::WebviewWindow) -> Result<(), String> {
    let (x, y, w, h) = primary_logical_bounds(window.app_handle())?;
    window
        .set_position(tauri::LogicalPosition::new(x, y))
        .map_err(|e| format!("overlay set_position: {e}"))?;
    window
        .set_size(tauri::LogicalSize::new(w, h))
        .map_err(|e| format!("overlay set_size: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_overlay_only_host_with_peer() {
        assert!(!host_needs_overlay(&SessionState::Idle));
        assert!(!host_needs_overlay(&SessionState::Hosting {
            code: "123456".into(),
            session_id: "s".into(),
        }));
        assert!(host_needs_overlay(&SessionState::InSession {
            role: "host".into(),
            session_id: "s".into(),
            code: Some("123456".into()),
            peer_connected: true,
        }));
        assert!(!host_needs_overlay(&SessionState::InSession {
            role: "host".into(),
            session_id: "s".into(),
            code: Some("123456".into()),
            peer_connected: false,
        }));
        assert!(!host_needs_overlay(&SessionState::InSession {
            role: "guest".into(),
            session_id: "s".into(),
            code: None,
            peer_connected: true,
        }));
    }
}
