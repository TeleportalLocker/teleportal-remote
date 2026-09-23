//! Trait commun d’injection d’événements.

use teleportal_protocol::Message;

use crate::error::InputError;
use crate::types::InjectConfig;

/// Injecteur d’événements souris / clavier / scroll.
pub trait InputInjector: Send {
    /// Démarre l’injecteur (permissions + résolution écran).
    ///
    /// # Errors
    ///
    /// Config invalide, permission refusée, ou plateforme non supportée.
    fn start(config: InjectConfig) -> Result<Self, InputError>
    where
        Self: Sized;

    /// Injecte un message de contrôle (`Mouse*` / `KeyEvent`).
    ///
    /// Les autres messages sont ignorés (`Ok(())`).
    ///
    /// # Errors
    ///
    /// Échec natif ou touche inconnue.
    fn inject(&mut self, msg: &Message) -> Result<(), InputError>;

    /// Arrête l’injecteur.
    ///
    /// # Errors
    ///
    /// Échec de libération native (rare).
    fn stop(self) -> Result<(), InputError>
    where
        Self: Sized;
}
