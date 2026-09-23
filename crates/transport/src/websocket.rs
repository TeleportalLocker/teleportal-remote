//! Client WebSocket implémentant [`Connection`].
//!
//! Une message binaire WS = une frame protocolaire length-prefixed (Phase 2 / ADR 0005).

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WsError, Message as WsMessage},
    MaybeTlsStream, WebSocketStream,
};

use crate::connection::Connection;
use crate::error::TransportError;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Connexion client WebSocket vers le relay.
pub struct WebsocketConnection {
    stream: Option<WsStream>,
}

impl WebsocketConnection {
    /// Établit une connexion WebSocket vers `url` (ex. `ws://127.0.0.1:7800/ws`).
    ///
    /// # Errors
    ///
    /// Échec DNS, TCP, handshake WS, ou URL invalide.
    pub async fn connect(url: &str) -> Result<Self, TransportError> {
        let (stream, _response) = connect_async(url)
            .await
            .map_err(|e| TransportError::Io(e.to_string()))?;
        Ok(Self {
            stream: Some(stream),
        })
    }

    fn stream_mut(&mut self) -> Result<&mut WsStream, TransportError> {
        self.stream.as_mut().ok_or(TransportError::Closed)
    }
}

#[async_trait]
impl Connection for WebsocketConnection {
    async fn send(&mut self, frame: Bytes) -> Result<(), TransportError> {
        let stream = self.stream_mut()?;
        stream
            .send(WsMessage::Binary(frame))
            .await
            .map_err(map_ws_error)
    }

    async fn recv(&mut self) -> Result<Bytes, TransportError> {
        loop {
            let next = {
                let stream = self.stream_mut()?;
                stream.next().await
            };
            match next {
                None => return Err(TransportError::Closed),
                Some(Err(e)) => return Err(map_ws_error(e)),
                Some(Ok(WsMessage::Binary(data))) => return Ok(data),
                Some(Ok(WsMessage::Close(_))) => {
                    self.stream.take();
                    return Err(TransportError::Closed);
                }
                Some(Ok(WsMessage::Ping(payload))) => {
                    let stream = self.stream_mut()?;
                    stream
                        .send(WsMessage::Pong(payload))
                        .await
                        .map_err(map_ws_error)?;
                }
                Some(Ok(WsMessage::Pong(_))) | Some(Ok(WsMessage::Frame(_))) => {}
                Some(Ok(WsMessage::Text(_))) => {
                    return Err(TransportError::Io("unexpected text WebSocket frame".into()));
                }
            }
        }
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.close(None).await;
        }
        Ok(())
    }
}

fn map_ws_error(err: WsError) -> TransportError {
    match err {
        WsError::ConnectionClosed | WsError::AlreadyClosed => TransportError::Closed,
        other => TransportError::Io(other.to_string()),
    }
}
