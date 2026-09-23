//! Trait commun de capture d’écran.

use crate::error::CaptureError;
use crate::types::{CaptureConfig, DisplayInfo, Frame};

/// Producteur de frames écran.
pub trait Capturer: Send {
    /// Liste les écrans disponibles.
    ///
    /// # Errors
    ///
    /// Erreurs natives ou plateforme non supportée.
    fn displays() -> Result<Vec<DisplayInfo>, CaptureError>
    where
        Self: Sized;

    /// Démarre la capture selon `config`.
    ///
    /// # Errors
    ///
    /// Config invalide, écran introuvable, ou échec natif.
    fn start(config: CaptureConfig) -> Result<Self, CaptureError>
    where
        Self: Sized;

    /// Tente d’acquérir une frame (ou `None` si pas de mise à jour).
    ///
    /// # Errors
    ///
    /// Perte d’accès ou erreur native.
    fn grab(&mut self) -> Result<Option<Frame>, CaptureError>;

    /// Arrête la capture et libère les ressources.
    ///
    /// # Errors
    ///
    /// Échec de libération native (rare).
    fn stop(self) -> Result<(), CaptureError>
    where
        Self: Sized;
}
