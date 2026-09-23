//! Erreurs d’encodage / décodage vidéo.

use thiserror::Error;

/// Erreurs d’encodage.
#[derive(Debug, Error)]
pub enum EncodeError {
    /// Configuration invalide.
    #[error("invalid encode config: {0}")]
    InvalidConfig(String),

    /// Frame source invalide (format, dimensions, buffer).
    #[error("invalid source frame: {0}")]
    InvalidFrame(String),

    /// Échec encodeur natif (OpenH264 ou futur backend HW).
    #[error("native encoder error: {0}")]
    Native(String),
}

/// Erreurs de décodage.
#[derive(Debug, Error)]
pub enum DecodeError {
    /// Configuration invalide.
    #[error("invalid decode config: {0}")]
    InvalidConfig(String),

    /// Bitstream invalide ou corrompu.
    #[error("invalid bitstream: {0}")]
    InvalidBitstream(String),

    /// Échec décodeur natif.
    #[error("native decoder error: {0}")]
    Native(String),
}
