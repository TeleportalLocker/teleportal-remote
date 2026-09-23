//! Erreurs de synchronisation curseur.

use thiserror::Error;

/// Erreur `teleportal-cursor-sync`.
#[derive(Debug, Error)]
pub enum CursorError {
    /// Plateforme non supportée (poll OS).
    #[error("cursor poll unsupported on this platform")]
    UnsupportedPlatform,
    /// Dimensions d’écran invalides.
    #[error("invalid display size: {0}")]
    InvalidDisplay(String),
    /// Configuration invalide.
    #[error("invalid cursor sync config: {0}")]
    InvalidConfig(String),
    /// Échec API native.
    #[error("native cursor poll failed: {0}")]
    Native(String),
}
