//! Erreurs du client session.

use thiserror::Error;

/// Erreurs surface UI / commands.
#[derive(Debug, Error)]
pub enum SessionError {
    /// Session déjà active.
    #[error("une session est déjà en cours")]
    Busy,

    /// Aucune session active.
    #[error("aucune session active")]
    NotConnected,

    /// Rejet ou échec protocolaire.
    #[error("{0}")]
    Protocol(String),

    /// Transport / réseau.
    #[error("transport: {0}")]
    Transport(String),

    /// Code de session invalide.
    #[error("code invalide: {0}")]
    InvalidCode(String),
}

impl From<teleportal_transport::TransportError> for SessionError {
    fn from(value: teleportal_transport::TransportError) -> Self {
        Self::Transport(value.to_string())
    }
}

impl From<teleportal_protocol::ProtocolError> for SessionError {
    fn from(value: teleportal_protocol::ProtocolError) -> Self {
        Self::Protocol(value.to_string())
    }
}

impl serde::Serialize for SessionError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
