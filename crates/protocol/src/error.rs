//! Erreurs du protocole Teleportal.

use thiserror::Error;

/// Erreurs de sérialisation, framing ou validation protocolaire.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    /// Identifiant ou code de session mal formé.
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),

    /// Message rejeté par les règles de validation MVP.
    #[error("validation failed: {0}")]
    Validation(String),

    /// Frame trop grande pour les limites MVP.
    #[error("frame too large: {size} bytes (max {max})")]
    FrameTooLarge {
        /// Taille observée.
        size: usize,
        /// Limite autorisée.
        max: usize,
    },

    /// Buffer trop court pour lire l’en-tête ou le corps.
    #[error("truncated frame")]
    TruncatedFrame,

    /// Échec MessagePack / serde.
    #[error("codec error: {0}")]
    Codec(String),
}
