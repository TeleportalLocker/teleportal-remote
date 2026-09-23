//! Trait de connexion async indépendant du backend (WebSocket, QUIC, …).

use async_trait::async_trait;
use bytes::Bytes;
use teleportal_protocol::{decode_message, encode_message, Message};

use crate::error::TransportError;

/// Connexion bidirectionnelle transportant des frames brutes (bytes).
///
/// Implémentation WebSocket prévue en Phase 3 ; QUIC plus tard.
/// Ne pas ajouter de logique métier ici.
#[async_trait]
pub trait Connection: Send + Sync {
    /// Envoie une frame complète (déjà length-prefixed si message protocolaire).
    async fn send(&mut self, frame: Bytes) -> Result<(), TransportError>;

    /// Reçoit la prochaine frame complète.
    async fn recv(&mut self) -> Result<Bytes, TransportError>;

    /// Ferme la connexion proprement.
    async fn close(&mut self) -> Result<(), TransportError>;
}

/// Encode et envoie un [`Message`] protocolaire.
///
/// # Errors
///
/// Propagates codec or transport errors.
pub async fn send_message<C: Connection + ?Sized>(
    conn: &mut C,
    message: &Message,
) -> Result<(), TransportError> {
    let frame = encode_message(message)?;
    conn.send(Bytes::from(frame)).await
}

/// Reçoit et décode un [`Message`] protocolaire.
///
/// # Errors
///
/// Propagates transport or codec/validation errors.
pub async fn recv_message<C: Connection + ?Sized>(conn: &mut C) -> Result<Message, TransportError> {
    let frame = conn.recv().await?;
    Ok(decode_message(&frame)?)
}
