//! Erreurs de capture d’écran.

use thiserror::Error;

/// Erreurs du crate `teleportal-capture`.
#[derive(Debug, Error)]
pub enum CaptureError {
    /// Plateforme non supportée pour ce backend.
    #[error("capture unsupported on this platform")]
    UnsupportedPlatform,

    /// Index d’écran invalide.
    #[error("display index out of range: {0}")]
    DisplayNotFound(usize),

    /// Aucune sortie vidéo détectée.
    #[error("no displays found")]
    NoDisplays,

    /// Configuration invalide.
    #[error("invalid capture config: {0}")]
    InvalidConfig(String),

    /// Perte d’accès DXGI (changement de mode, UAC, etc.).
    #[error("desktop duplication access lost")]
    AccessLost,

    /// Permission Screen Recording refusée (macOS).
    #[error("screen recording permission denied")]
    PermissionDenied,

    /// Échec API native / DXGI / D3D11 / ScreenCaptureKit.
    #[error("native capture error: {0}")]
    Native(String),
}
