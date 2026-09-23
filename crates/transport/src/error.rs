//! Erreurs de la couche transport.

use teleportal_protocol::ProtocolError;
use thiserror::Error;

/// Erreurs d’envoi / réception sur une connexion.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Connexion fermée.
    #[error("connection closed")]
    Closed,

    /// Échec I/O ou canal interne.
    #[error("transport I/O: {0}")]
    Io(String),

    /// Échec codec / protocole.
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}
