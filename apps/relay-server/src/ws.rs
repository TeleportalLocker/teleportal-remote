//! Handlers HTTP / WebSocket Axum.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::{Message as AxumWsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use teleportal_protocol::{decode_message, encode_message, Message};
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::config::Config;
use crate::handler::{handle_message, ConnPhase, HandlerOutput};
use crate::rate_limit::RateLimiter;
use crate::state::SessionRegistry;

/// État partagé Axum.
#[derive(Clone)]
pub struct AppState {
    /// Registre de sessions.
    pub registry: SessionRegistry,
    /// Config (TTL, etc.).
    pub config: Arc<Config>,
    /// Limiteur Create/Join par IP.
    pub rate_limiter: Arc<RateLimiter>,
}

/// Construit le router HTTP.
pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_upgrade))
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    "ok"
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let peer_ip = client_ip(&headers, addr.ip());
    ws.on_upgrade(move |socket| handle_socket(socket, state, peer_ip))
}

/// IP client : `X-Forwarded-For` / `X-Real-IP` (reverse-proxy) sinon peer TCP.
fn client_ip(headers: &HeaderMap, fallback: IpAddr) -> IpAddr {
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = xff.split(',').next() {
            if let Ok(ip) = first.trim().parse::<IpAddr>() {
                return ip;
            }
        }
    }
    if let Some(real) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        if let Ok(ip) = real.trim().parse::<IpAddr>() {
            return ip;
        }
    }
    fallback
}

async fn handle_socket(socket: WebSocket, state: AppState, peer_ip: IpAddr) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (out_tx, mut out_rx) = mpsc::channel::<Bytes>(64);

    let writer = tokio::spawn(async move {
        while let Some(frame) = out_rx.recv().await {
            if ws_tx.send(AxumWsMessage::Binary(frame)).await.is_err() {
                break;
            }
        }
        let _ = ws_tx.close().await;
    });

    let mut phase = ConnPhase::ExpectHello;
    let mut session_peer: Option<(teleportal_protocol::SessionId, teleportal_protocol::PeerId)> =
        None;

    while let Some(Ok(msg)) = ws_rx.next().await {
        let AxumWsMessage::Binary(data) = msg else {
            if matches!(msg, AxumWsMessage::Close(_)) {
                break;
            }
            continue;
        };

        let message = match decode_message(&data) {
            Ok(m) => m,
            Err(e) => {
                warn!(error = %e, "invalid frame");
                let err = Message::Error {
                    code: "invalid_frame".into(),
                    message: e.to_string(),
                };
                if let Ok(frame) = encode_message(&err) {
                    let _ = out_tx.send(Bytes::from(frame)).await;
                }
                break;
            }
        };

        let result = handle_message(
            &state.registry,
            &state.rate_limiter,
            peer_ip,
            phase,
            &out_tx,
            message,
        )
        .await;
        phase = result.phase;
        if let ConnPhase::InSession {
            session_id,
            peer_id,
            ..
        } = phase
        {
            session_peer = Some((session_id, peer_id));
        }

        let mut should_close = false;
        for output in result.outputs {
            match output {
                HandlerOutput::Reply(m) => match encode_message(&m) {
                    Ok(frame) => {
                        if out_tx.send(Bytes::from(frame)).await.is_err() {
                            should_close = true;
                            break;
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "encode reply failed");
                        should_close = true;
                        break;
                    }
                },
                HandlerOutput::Close(maybe) => {
                    if let Some(m) = maybe {
                        if let Ok(frame) = encode_message(&m) {
                            let _ = out_tx.send(Bytes::from(frame)).await;
                        }
                    }
                    should_close = true;
                }
            }
        }
        if should_close {
            break;
        }
    }

    if let Some((session_id, peer_id)) = session_peer {
        debug!(%session_id, %peer_id, "peer disconnected");
        state.registry.detach_peer(session_id, peer_id).await;
    }

    drop(out_tx);
    let _ = writer.await;
}

fn make_service(
    state: AppState,
) -> axum::extract::connect_info::IntoMakeServiceWithConnectInfo<Router, SocketAddr> {
    app_router(state).into_make_service_with_connect_info::<SocketAddr>()
}

/// Démarre le serveur sur `bind` (utilisé par main et tests).
pub async fn serve(bind: SocketAddr, state: AppState) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, make_service(state)).await?;
    Ok(())
}

/// Bind sur port éphémère et retourne l’adresse réelle.
pub async fn serve_ephemeral(
    state: AppState,
) -> anyhow::Result<(SocketAddr, tokio::task::JoinHandle<anyhow::Result<()>>)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        axum::serve(listener, make_service(state))
            .await
            .map_err(Into::into)
    });
    Ok((addr, handle))
}

#[cfg(test)]
mod client_ip_tests {
    use super::*;
    use axum::http::HeaderValue;
    use std::net::Ipv4Addr;

    #[test]
    fn prefers_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.10, 10.0.0.1"),
        );
        let ip = client_ip(&headers, IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 10)));
    }

    #[test]
    fn falls_back_to_peer() {
        let headers = HeaderMap::new();
        let ip = client_ip(&headers, IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)));
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)));
    }
}
