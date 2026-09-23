//! Machine à états session + orchestration WebSocket + média + curseur.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use teleportal_capture::{CaptureConfig, CaptureError, Capturer, PlatformCapturer};
use teleportal_cursor_sync::{poll_os_cursor, CursorPose, CursorSync, SyncConfig};
use teleportal_input::{InjectConfig, InputError, InputInjector, PlatformInjector};
use teleportal_protocol::{
    Message, PeerId, Role, SessionCode, SessionId, VideoCodec, PROTOCOL_VERSION,
};
use teleportal_transport::{recv_message, send_message, Connection, WebsocketConnection};
use teleportal_video_stream::{
    to_video_message, DecodeConfig, DecodedFrame, EncodeConfig, PlatformDecoder, PlatformEncoder,
    VideoDecoder, VideoEncoder,
};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::error::SessionError;

/// Instantané sérialisable pour le frontend.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SessionState {
    /// Aucune connexion.
    Idle,
    /// Connexion / handshake en cours.
    Connecting,
    /// Hôte en attente d’un pair (code affiché).
    Hosting {
        /// Code 6 chiffres.
        code: String,
        /// Identifiant de session.
        session_id: String,
    },
    /// Session établie.
    InSession {
        /// Rôle local.
        role: String,
        /// Identifiant de session.
        session_id: String,
        /// Code (hôte uniquement).
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
        /// Un pair distant est présent.
        peer_connected: bool,
    },
    /// Erreur terminale (retour Accueil possible).
    Error {
        /// Message humain.
        message: String,
    },
}

/// Frame RGBA prête pour le canvas (IPC JSON via base64).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoFrameEvent {
    /// Identifiant monotone.
    pub frame_id: u64,
    /// Largeur.
    pub width: u32,
    /// Hauteur.
    pub height: u32,
    /// Horodatage Unix ms.
    pub timestamp_ms: u64,
    /// Pixels RGBA8 encodés en base64 (compatible événement JSON Tauri).
    pub rgba_base64: String,
}

impl From<DecodedFrame> for VideoFrameEvent {
    fn from(frame: DecodedFrame) -> Self {
        Self {
            frame_id: frame.frame_id,
            width: frame.width,
            height: frame.height,
            timestamp_ms: frame.timestamp_ms,
            rgba_base64: BASE64.encode(frame.data.as_ref()),
        }
    }
}

/// Pose du curseur distant (interpolée) pour le frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteCursorEvent {
    /// Identifiant du curseur distant.
    pub cursor_id: String,
    /// Abscisse normalisée.
    pub x: f32,
    /// Ordonnée normalisée.
    pub y: f32,
    /// Horodatage Unix ms.
    pub timestamp_ms: u64,
}

impl From<CursorPose> for RemoteCursorEvent {
    fn from(pose: CursorPose) -> Self {
        Self {
            cursor_id: pose.cursor_id.to_string(),
            x: pose.x,
            y: pose.y,
            timestamp_ms: pose.timestamp_ms,
        }
    }
}

fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn primary_display_size() -> (u32, u32) {
    if let Ok(displays) = PlatformCapturer::displays() {
        if let Some(d) = displays.first() {
            return (d.width.max(1), d.height.max(1));
        }
    }
    (1920, 1080)
}

fn peer_connected(state: &SessionState) -> bool {
    matches!(
        state,
        SessionState::InSession {
            peer_connected: true,
            ..
        }
    )
}

/// Applique un message signaling entrant sur l’état affiché (logique pure, testable).
#[must_use]
pub fn apply_signal(state: SessionState, message: &Message) -> SessionState {
    match (state, message) {
        (SessionState::Hosting { code, session_id }, Message::PeerJoined { .. }) => {
            SessionState::InSession {
                role: "host".into(),
                session_id,
                code: Some(code),
                peer_connected: true,
            }
        }
        (
            SessionState::InSession {
                role,
                session_id,
                code,
                ..
            },
            Message::PeerJoined { .. },
        ) => SessionState::InSession {
            role,
            session_id,
            code,
            peer_connected: true,
        },
        (
            SessionState::InSession {
                role,
                session_id,
                code,
                ..
            },
            Message::PeerLeft { .. },
        ) => SessionState::InSession {
            role,
            session_id,
            code,
            peer_connected: false,
        },
        (_, Message::SessionExpired) => SessionState::Error {
            message: "session expirée".into(),
        },
        (_, Message::Error { message, .. }) => SessionState::Error {
            message: message.clone(),
        },
        (_, Message::JoinRejected { reason }) => SessionState::Error {
            message: format!("connexion refusée: {reason}"),
        },
        (other, _) => other,
    }
}

fn host_should_stream(state: &SessionState) -> bool {
    matches!(
        state,
        SessionState::InSession {
            role,
            peer_connected: true,
            ..
        } if role == "host"
    )
}

/// Commande interne vers la boucle session.
enum SessionCommand {
    Leave,
    Send(Message),
    CursorPos { x: f32, y: f32 },
}

enum MediaOut {
    Frame(Message),
    Fatal(String),
}

/// Handle d’une session active (tâche + canal leave / contrôle).
pub struct ActiveSession {
    cmd_tx: mpsc::Sender<SessionCommand>,
    join: tokio::task::JoinHandle<()>,
}

/// Envoi de messages de contrôle sans retenir le Mutex session.
pub struct ControlHandle {
    tx: mpsc::Sender<SessionCommand>,
}

impl ControlHandle {
    /// Enfile un message de contrôle.
    pub async fn send(&self, msg: Message) -> Result<(), SessionError> {
        self.tx
            .send(SessionCommand::Send(msg))
            .await
            .map_err(|_| SessionError::NotConnected)
    }

    /// Enfile une position curseur locale (Guest) — throttle côté `CursorSync`.
    pub async fn send_cursor_pos(&self, x: f32, y: f32) -> Result<(), SessionError> {
        self.tx
            .send(SessionCommand::CursorPos { x, y })
            .await
            .map_err(|_| SessionError::NotConnected)
    }
}

impl ActiveSession {
    /// Demande l’arrêt propre.
    pub async fn leave(self) {
        let _ = self.cmd_tx.send(SessionCommand::Leave).await;
        let _ = self.join.await;
    }

    /// Handle d’envoi contrôle (clonable hors Mutex).
    #[must_use]
    pub fn control_handle(&self) -> ControlHandle {
        ControlHandle {
            tx: self.cmd_tx.clone(),
        }
    }
}

/// Démarre une session hôte.
pub async fn start_host(
    relay_url: &str,
    on_state: impl Fn(SessionState) + Send + Sync + 'static,
    on_video: impl Fn(VideoFrameEvent) + Send + Sync + 'static,
    on_remote_cursor: impl Fn(RemoteCursorEvent) + Send + Sync + 'static,
) -> Result<(ActiveSession, SessionState), SessionError> {
    on_state(SessionState::Connecting);
    let mut conn = WebsocketConnection::connect(relay_url).await?;
    send_message(
        &mut conn,
        &Message::Hello {
            protocol_version: PROTOCOL_VERSION,
            role: Role::Host,
        },
    )
    .await?;
    send_message(&mut conn, &Message::CreateSession).await?;

    let created = recv_message(&mut conn).await?;
    let (session_id, code) = match created {
        Message::SessionCreated { session_id, code } => (session_id, code),
        Message::Error { message, .. } => return Err(SessionError::Protocol(message)),
        other => {
            return Err(SessionError::Protocol(format!(
                "attendu SessionCreated, reçu {other:?}"
            )));
        }
    };

    let state = SessionState::Hosting {
        code: code.as_str().to_owned(),
        session_id: session_id.to_string(),
    };
    on_state(state.clone());
    info!(%session_id, code = %code, "hosting session");

    let active = spawn_session_loop(
        conn,
        state.clone(),
        Role::Host,
        Some(code),
        session_id,
        on_state,
        on_video,
        on_remote_cursor,
    );
    Ok((active, state))
}

/// Rejoint une session guest.
pub async fn start_guest(
    relay_url: &str,
    code_raw: &str,
    on_state: impl Fn(SessionState) + Send + Sync + 'static,
    on_video: impl Fn(VideoFrameEvent) + Send + Sync + 'static,
    on_remote_cursor: impl Fn(RemoteCursorEvent) + Send + Sync + 'static,
) -> Result<(ActiveSession, SessionState), SessionError> {
    let code =
        SessionCode::parse(code_raw).map_err(|e| SessionError::InvalidCode(e.to_string()))?;
    on_state(SessionState::Connecting);
    let mut conn = WebsocketConnection::connect(relay_url).await?;
    send_message(
        &mut conn,
        &Message::Hello {
            protocol_version: PROTOCOL_VERSION,
            role: Role::Guest,
        },
    )
    .await?;
    send_message(&mut conn, &Message::JoinSession { code }).await?;

    let response = recv_message(&mut conn).await?;
    let (session_id, peer_id) = match response {
        Message::JoinAccepted {
            session_id,
            peer_id,
            ..
        } => (session_id, peer_id),
        Message::JoinRejected { reason } => {
            return Err(SessionError::Protocol(format!(
                "connexion refusée: {reason}"
            )));
        }
        Message::Error { message, .. } => return Err(SessionError::Protocol(message)),
        other => {
            return Err(SessionError::Protocol(format!(
                "attendu JoinAccepted, reçu {other:?}"
            )));
        }
    };

    let _ = peer_id;
    let state = SessionState::InSession {
        role: "guest".into(),
        session_id: session_id.to_string(),
        code: None,
        peer_connected: true,
    };
    on_state(state.clone());
    info!(%session_id, "joined session as guest");

    let active = spawn_session_loop(
        conn,
        state.clone(),
        Role::Guest,
        None,
        session_id,
        on_state,
        on_video,
        on_remote_cursor,
    );
    Ok((active, state))
}

fn map_capture_error(err: CaptureError) -> String {
    match err {
        CaptureError::PermissionDenied => {
            "permission Screen Recording refusée — activez-la dans Réglages système".into()
        }
        CaptureError::NoDisplays => "aucun écran détecté".into(),
        CaptureError::UnsupportedPlatform => {
            "capture d’écran non supportée sur cette plateforme".into()
        }
        other => format!("capture: {other}"),
    }
}

fn map_input_error(err: InputError) -> String {
    match err {
        InputError::PermissionDenied => {
            "permission Accessibilité refusée — activez-la dans Réglages système pour Teleportal"
                .into()
        }
        InputError::UnsupportedPlatform => {
            "injection d’input non supportée sur cette plateforme".into()
        }
        other => format!("input: {other}"),
    }
}

fn is_control_message(msg: &Message) -> bool {
    matches!(
        msg,
        Message::MouseMove { .. }
            | Message::MouseButton { .. }
            | Message::MouseScroll { .. }
            | Message::KeyEvent { .. }
    )
}

fn start_host_injector() -> Result<PlatformInjector, String> {
    let mut config = InjectConfig::default();
    if let Ok(displays) = PlatformCapturer::displays() {
        if let Some(d) = displays.first() {
            config.display_width = d.width.max(1);
            config.display_height = d.height.max(1);
        }
    }
    PlatformInjector::start(config).map_err(map_input_error)
}

fn start_host_capture(media_tx: mpsc::Sender<MediaOut>) -> (Arc<AtomicBool>, JoinHandle<()>) {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&stop);
    let join = std::thread::Builder::new()
        .name("teleportal-host-capture".into())
        .spawn(move || {
            let mut capturer = match PlatformCapturer::start(CaptureConfig::default()) {
                Ok(c) => c,
                Err(e) => {
                    let _ = media_tx.blocking_send(MediaOut::Fatal(map_capture_error(e)));
                    return;
                }
            };
            let mut encoder = match PlatformEncoder::start(EncodeConfig::default()) {
                Ok(e) => e,
                Err(e) => {
                    let _ = capturer.stop();
                    let _ = media_tx.blocking_send(MediaOut::Fatal(format!("encode: {e}")));
                    return;
                }
            };

            while !stop_flag.load(Ordering::Relaxed) {
                match capturer.grab() {
                    Ok(Some(frame)) => match encoder.encode(&frame) {
                        Ok(Some(encoded)) => {
                            let msg = to_video_message(encoded);
                            if media_tx.blocking_send(MediaOut::Frame(msg)).is_err() {
                                break;
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            warn!(error = %e, "encode frame failed");
                        }
                    },
                    Ok(None) => {}
                    Err(e) => {
                        let _ = media_tx.blocking_send(MediaOut::Fatal(map_capture_error(e)));
                        break;
                    }
                }
            }

            let _ = encoder.stop();
            let _ = capturer.stop();
        })
        .expect("spawn host capture thread");
    (stop, join)
}

fn stop_host_capture(stop: &Option<Arc<AtomicBool>>, join: &mut Option<JoinHandle<()>>) {
    if let Some(flag) = stop {
        flag.store(true, Ordering::Relaxed);
    }
    if let Some(handle) = join.take() {
        let _ = handle.join();
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_session_loop(
    mut conn: WebsocketConnection,
    initial: SessionState,
    role: Role,
    _code: Option<SessionCode>,
    _session_id: SessionId,
    on_state: impl Fn(SessionState) + Send + Sync + 'static,
    on_video: impl Fn(VideoFrameEvent) + Send + Sync + 'static,
    on_remote_cursor: impl Fn(RemoteCursorEvent) + Send + Sync + 'static,
) -> ActiveSession {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<SessionCommand>(64);
    let (media_tx, mut media_rx) = mpsc::channel::<MediaOut>(8);

    let join = tokio::spawn(async move {
        let mut state = initial;
        let mut decoder = if matches!(role, Role::Guest) {
            match PlatformDecoder::start(DecodeConfig::default()) {
                Ok(d) => Some(d),
                Err(e) => {
                    on_state(SessionState::Error {
                        message: format!("décodeur: {e}"),
                    });
                    return;
                }
            }
        } else {
            None
        };

        let mut injector: Option<PlatformInjector> = None;
        let mut cursor_sync = CursorSync::new(SyncConfig::default());
        let (display_w, display_h) = primary_display_size();
        let mut last_host_cursor_poll = Instant::now()
            .checked_sub(Duration::from_millis(50))
            .unwrap_or_else(Instant::now);
        let mut cursor_tick = tokio::time::interval(Duration::from_millis(16));
        cursor_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut host_stop: Option<Arc<AtomicBool>> = None;
        let mut host_join: Option<JoinHandle<()>> = None;

        if matches!(role, Role::Host) && host_should_stream(&state) {
            match start_host_injector() {
                Ok(inj) => injector = Some(inj),
                Err(message) => {
                    on_state(SessionState::Error { message });
                    return;
                }
            }
            let (stop, handle) = start_host_capture(media_tx.clone());
            host_stop = Some(stop);
            host_join = Some(handle);
        }

        loop {
            tokio::select! {
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(SessionCommand::Leave) | None => {
                            stop_host_capture(&host_stop, &mut host_join);
                            host_stop = None;
                            let _ = send_message(&mut conn, &Message::LeaveSession).await;
                            let _ = conn.close().await;
                            on_state(SessionState::Idle);
                            break;
                        }
                        Some(SessionCommand::Send(msg)) => {
                            if matches!(role, Role::Guest) && is_control_message(&msg) {
                                if let Err(e) = send_message(&mut conn, &msg).await {
                                    warn!(error = %e, "failed to send control");
                                }
                            }
                        }
                        Some(SessionCommand::CursorPos { x, y }) => {
                            if matches!(role, Role::Guest) && peer_connected(&state) {
                                let now = unix_now_ms();
                                if let Some(msg) = cursor_sync.push_local(x, y, now) {
                                    if let Err(e) = send_message(&mut conn, &msg).await {
                                        warn!(error = %e, "failed to send cursor");
                                    }
                                }
                            }
                        }
                    }
                }
                _ = cursor_tick.tick() => {
                    let now = unix_now_ms();
                    if matches!(role, Role::Host)
                        && peer_connected(&state)
                        && last_host_cursor_poll.elapsed() >= Duration::from_millis(50)
                    {
                        last_host_cursor_poll = Instant::now();
                        match poll_os_cursor(display_w, display_h) {
                            Ok((x, y)) => {
                                if let Some(msg) = cursor_sync.push_local(x, y, now) {
                                    if let Err(e) = send_message(&mut conn, &msg).await {
                                        warn!(error = %e, "failed to send host cursor");
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(error = %e, "host cursor poll failed");
                            }
                        }
                    }
                    if let Some(pose) = cursor_sync.remote_pose(now) {
                        on_remote_cursor(RemoteCursorEvent::from(pose));
                    }
                }
                media = media_rx.recv(), if matches!(role, Role::Host) => {
                    match media {
                        Some(MediaOut::Frame(msg)) => {
                            if let Err(e) = send_message(&mut conn, &msg).await {
                                warn!(error = %e, "failed to send video frame");
                                stop_host_capture(&host_stop, &mut host_join);
                                host_stop = None;
                                on_state(SessionState::Error {
                                    message: format!("envoi vidéo: {e}"),
                                });
                                break;
                            }
                        }
                        Some(MediaOut::Fatal(message)) => {
                            stop_host_capture(&host_stop, &mut host_join);
                            host_stop = None;
                            on_state(SessionState::Error { message });
                            break;
                        }
                        None => {}
                    }
                }
                incoming = recv_message(&mut conn) => {
                    match incoming {
                        Ok(msg) => {
                            if let Message::VideoFrame {
                                frame_id,
                                timestamp_ms,
                                codec,
                                data,
                                ..
                            } = &msg
                            {
                                if matches!(role, Role::Guest) && *codec == VideoCodec::H264 {
                                    if let Some(dec) = decoder.as_mut() {
                                        match dec.decode(*frame_id, *timestamp_ms, data) {
                                            Ok(Some(frame)) => on_video(VideoFrameEvent::from(frame)),
                                            Ok(None) => {}
                                            Err(e) => warn!(error = %e, "decode failed"),
                                        }
                                    }
                                }
                                continue;
                            }

                            if matches!(role, Role::Host) && is_control_message(&msg) {
                                if let Some(inj) = injector.as_mut() {
                                    if let Err(e) = inj.inject(&msg) {
                                        warn!(error = %e, "inject failed");
                                        if matches!(e, InputError::PermissionDenied) {
                                            stop_host_capture(&host_stop, &mut host_join);
                                            host_stop = None;
                                            on_state(SessionState::Error {
                                                message: map_input_error(e),
                                            });
                                            break;
                                        }
                                    }
                                }
                                continue;
                            }

                            if matches!(&msg, Message::CursorMove { .. }) {
                                cursor_sync.on_remote(&msg);
                                if let Some(pose) = cursor_sync.remote_pose(unix_now_ms()) {
                                    on_remote_cursor(RemoteCursorEvent::from(pose));
                                }
                                continue;
                            }

                            let was_streaming = host_should_stream(&state);
                            let next = apply_signal(state.clone(), &msg);
                            if next != state {
                                state = next.clone();
                                on_state(next);
                            }

                            if matches!(role, Role::Host) {
                                let now_streaming = host_should_stream(&state);
                                if !was_streaming && now_streaming {
                                    match start_host_injector() {
                                        Ok(inj) => injector = Some(inj),
                                        Err(message) => {
                                            stop_host_capture(&host_stop, &mut host_join);
                                            host_stop = None;
                                            on_state(SessionState::Error { message });
                                            break;
                                        }
                                    }
                                    stop_host_capture(&host_stop, &mut host_join);
                                    let (stop, handle) = start_host_capture(media_tx.clone());
                                    host_stop = Some(stop);
                                    host_join = Some(handle);
                                    info!("host capture + injector started");
                                } else if was_streaming && !now_streaming {
                                    stop_host_capture(&host_stop, &mut host_join);
                                    host_stop = None;
                                    if let Some(inj) = injector.take() {
                                        let _ = inj.stop();
                                    }
                                    info!("host capture + injector stopped");
                                }
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "session recv ended");
                            stop_host_capture(&host_stop, &mut host_join);
                            host_stop = None;
                            on_state(SessionState::Error {
                                message: format!("connexion perdue: {e}"),
                            });
                            break;
                        }
                    }
                }
            }
        }

        stop_host_capture(&host_stop, &mut host_join);
        if let Some(dec) = decoder.take() {
            let _ = dec.stop();
        }
        if let Some(inj) = injector.take() {
            let _ = inj.stop();
        }
    });
    ActiveSession { cmd_tx, join }
}

/// Helper test : peer id unused warning silence via allow.
#[allow(dead_code)]
fn _peer_id_marker(_: PeerId) {}

#[cfg(test)]
mod tests {
    use super::*;
    use teleportal_protocol::CursorId;

    #[test]
    fn peer_joined_promotes_hosting() {
        let state = SessionState::Hosting {
            code: "123456".into(),
            session_id: "sid".into(),
        };
        let next = apply_signal(
            state,
            &Message::PeerJoined {
                peer_id: PeerId::new(),
                role: Role::Guest,
            },
        );
        assert_eq!(
            next,
            SessionState::InSession {
                role: "host".into(),
                session_id: "sid".into(),
                code: Some("123456".into()),
                peer_connected: true,
            }
        );
    }

    #[test]
    fn peer_left_clears_flag() {
        let state = SessionState::InSession {
            role: "guest".into(),
            session_id: "sid".into(),
            code: None,
            peer_connected: true,
        };
        let next = apply_signal(
            state,
            &Message::PeerLeft {
                peer_id: PeerId::new(),
            },
        );
        assert!(!matches!(
            next,
            SessionState::InSession {
                peer_connected: true,
                ..
            }
        ));
    }

    #[test]
    fn session_expired_to_error() {
        let state = SessionState::InSession {
            role: "host".into(),
            session_id: "sid".into(),
            code: Some("111222".into()),
            peer_connected: false,
        };
        let next = apply_signal(state, &Message::SessionExpired);
        assert!(matches!(next, SessionState::Error { .. }));
    }

    #[test]
    fn cursor_move_keeps_state() {
        let state = SessionState::InSession {
            role: "host".into(),
            session_id: "sid".into(),
            code: None,
            peer_connected: true,
        };
        let next = apply_signal(
            state.clone(),
            &Message::CursorMove {
                cursor_id: CursorId::new(),
                x: 0.5,
                y: 0.5,
                timestamp_ms: 1,
            },
        );
        assert_eq!(next, state);
    }

    #[test]
    fn host_should_stream_only_when_peer_connected() {
        assert!(!host_should_stream(&SessionState::Hosting {
            code: "1".into(),
            session_id: "s".into(),
        }));
        assert!(host_should_stream(&SessionState::InSession {
            role: "host".into(),
            session_id: "s".into(),
            code: Some("1".into()),
            peer_connected: true,
        }));
        assert!(!host_should_stream(&SessionState::InSession {
            role: "guest".into(),
            session_id: "s".into(),
            code: None,
            peer_connected: true,
        }));
    }
}
