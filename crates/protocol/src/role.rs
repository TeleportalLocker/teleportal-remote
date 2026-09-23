//! Rôle d’un pair dans une session.

use serde::{Deserialize, Serialize};

/// Rôle d’un participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Hôte : partage l’écran et peut recevoir le contrôle.
    Host,
    /// Invité : reçoit le flux et peut contrôler / collaborer.
    Guest,
}
