//! Erreurs d’injection d’input.

use thiserror::Error;

/// Erreurs du crate `teleportal-input`.
#[derive(Debug, Error)]
pub enum InputError {
    /// Plateforme non supportée.
    #[error("input injection unsupported on this platform")]
    UnsupportedPlatform,

    /// Permission système refusée (Accessibility macOS).
    #[error("accessibility permission denied")]
    PermissionDenied,

    /// Configuration invalide.
    #[error("invalid inject config: {0}")]
    InvalidConfig(String),

    /// Touche / code inconnu.
    #[error("unknown key code: {0}")]
    UnknownKey(String),

    /// Échec API native.
    #[error("native input error: {0}")]
    Native(String),
}
