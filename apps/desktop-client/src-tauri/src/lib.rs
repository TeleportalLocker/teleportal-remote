//! Bibliothèque Tauri du client Teleportal Remote.

#![deny(missing_docs)]
#![warn(clippy::all)]

mod config;
mod error;
mod overlay;
mod session;

use std::sync::Mutex;

use config::ClientConfig;
use error::SessionError;
use session::{ActiveSession, SessionState};
use tauri::{AppHandle, Emitter, Manager, State};
use teleportal_protocol::{validate_message, Message};

/// État global de l’application.
pub struct AppState {
    /// Session WebSocket active éventuelle.
    session: Mutex<Option<ActiveSession>>,
    /// Dernier état connu (pour get).
    last_state: Mutex<SessionState>,
    /// Config.
    config: ClientConfig,
}

impl AppState {
    fn new() -> Self {
        Self {
            session: Mutex::new(None),
            last_state: Mutex::new(SessionState::Idle),
            config: ClientConfig::from_env(),
        }
    }
}

fn emit_state(app: &AppHandle, state: &AppState, snapshot: SessionState) {
    if let Ok(mut guard) = state.last_state.lock() {
        *guard = snapshot.clone();
    }
    overlay::sync_host_overlay(app, &snapshot);
    let _ = app.emit("session-state", snapshot);
}

fn emit_error(app: &AppHandle, message: String) {
    let _ = app.emit("session-error", message);
}

fn on_session_state(app: &AppHandle, snapshot: SessionState) {
    if let Some(st) = app.try_state::<AppState>() {
        if let Ok(mut guard) = st.last_state.lock() {
            *guard = snapshot.clone();
        }
        // PeerLeft / fin de boucle → Idle|Error : libère le slot session.
        if matches!(snapshot, SessionState::Idle | SessionState::Error { .. }) {
            if let Ok(mut guard) = st.session.lock() {
                let _ = guard.take();
            }
        }
    }
    overlay::sync_host_overlay(app, &snapshot);
    let _ = app.emit("session-state", snapshot);
}

/// Version applicative.
#[tauri::command]
fn app_version() -> String {
    teleportal_shared::VERSION.to_owned()
}

/// Config client (URL relay).
#[tauri::command]
fn get_config(state: State<'_, AppState>) -> ClientConfig {
    state.config.clone()
}

/// Dernier état session connu.
#[tauri::command]
fn get_session_state(state: State<'_, AppState>) -> SessionState {
    state
        .last_state
        .lock()
        .map(|g| g.clone())
        .unwrap_or(SessionState::Idle)
}

/// Démarre une session hôte.
#[tauri::command]
async fn host_start(
    app: AppHandle,
    state: State<'_, AppState>,
    relay_url: String,
) -> Result<SessionState, SessionError> {
    {
        let guard = state.session.lock().map_err(|_| SessionError::Busy)?;
        if guard.is_some() {
            return Err(SessionError::Busy);
        }
    }

    let app_for_cb = app.clone();
    let app_for_video = app.clone();
    let app_for_cursor = app.clone();
    let (active, snapshot) = session::start_host(
        &relay_url,
        move |s| on_session_state(&app_for_cb, s),
        move |frame| {
            let _ = app_for_video.emit("video-frame", frame);
        },
        move |cursor| {
            let _ = app_for_cursor.emit("remote-cursor", cursor);
        },
    )
    .await
    .inspect_err(|e| emit_error(&app, e.to_string()))?;

    emit_state(&app, &state, snapshot.clone());
    *state.session.lock().map_err(|_| SessionError::Busy)? = Some(active);
    Ok(snapshot)
}

/// Rejoint une session via code.
#[tauri::command]
async fn guest_join(
    app: AppHandle,
    state: State<'_, AppState>,
    relay_url: String,
    code: String,
) -> Result<SessionState, SessionError> {
    {
        let guard = state.session.lock().map_err(|_| SessionError::Busy)?;
        if guard.is_some() {
            return Err(SessionError::Busy);
        }
    }

    let app_for_cb = app.clone();
    let app_for_video = app.clone();
    let app_for_cursor = app.clone();
    let (active, snapshot) = session::start_guest(
        &relay_url,
        &code,
        move |s| on_session_state(&app_for_cb, s),
        move |frame| {
            let _ = app_for_video.emit("video-frame", frame);
        },
        move |cursor| {
            let _ = app_for_cursor.emit("remote-cursor", cursor);
        },
    )
    .await
    .inspect_err(|e| emit_error(&app, e.to_string()))?;

    emit_state(&app, &state, snapshot.clone());
    *state.session.lock().map_err(|_| SessionError::Busy)? = Some(active);
    Ok(snapshot)
}

/// Quitte la session courante.
#[tauri::command]
async fn leave_session(app: AppHandle, state: State<'_, AppState>) -> Result<(), SessionError> {
    let active = {
        let mut guard = state.session.lock().map_err(|_| SessionError::Busy)?;
        guard.take()
    };
    match active {
        Some(session) => {
            session.leave().await;
            emit_state(&app, &state, SessionState::Idle);
            Ok(())
        }
        None => Err(SessionError::NotConnected),
    }
}

/// Envoie un message de contrôle (Guest → Host via relay).
#[tauri::command]
async fn send_control(state: State<'_, AppState>, message: Message) -> Result<(), SessionError> {
    match &message {
        Message::MouseMove { .. }
        | Message::MouseButton { .. }
        | Message::MouseScroll { .. }
        | Message::KeyEvent { .. } => {}
        _ => {
            return Err(SessionError::Protocol(
                "send_control n’accepte que Mouse*/KeyEvent".into(),
            ));
        }
    }
    validate_message(&message)?;
    let handle = {
        let guard = state.session.lock().map_err(|_| SessionError::Busy)?;
        let session = guard.as_ref().ok_or(SessionError::NotConnected)?;
        session.control_handle()
    };
    handle.send(message).await
}

/// Envoie la position curseur locale (Guest) — wrappée en `CursorMove` côté Rust.
#[tauri::command]
async fn send_cursor_pos(state: State<'_, AppState>, x: f32, y: f32) -> Result<(), SessionError> {
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return Err(SessionError::Protocol(
            "send_cursor_pos: coords hors [0,1]".into(),
        ));
    }
    let handle = {
        let guard = state.session.lock().map_err(|_| SessionError::Busy)?;
        let session = guard.as_ref().ok_or(SessionError::NotConnected)?;
        session.control_handle()
    };
    handle.send_cursor_pos(x, y).await
}

/// Point d’entrée Tauri.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    teleportal_shared::observability::init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            app_version,
            get_config,
            get_session_state,
            host_start,
            guest_join,
            leave_session,
            send_control,
            send_cursor_pos,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Teleportal Remote");
}
