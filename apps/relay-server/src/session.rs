//! Modèle de session en mémoire (pas de persistance métier).

use std::time::Instant;

use bytes::Bytes;
use teleportal_protocol::{PeerId, Role, SessionCode, SessionId};
use tokio::sync::mpsc;

/// Canal sortant vers un pair (messages déjà encodés).
pub type OutboundTx = mpsc::Sender<Bytes>;

/// Pair connecté à une session.
#[derive(Debug)]
pub struct PeerSlot {
    /// Identifiant du pair.
    pub peer_id: PeerId,
    /// Rôle Host ou Guest.
    pub role: Role,
    /// File d’envoi vers le WebSocket du pair.
    pub outbound: OutboundTx,
}

/// Session relay : au plus un host et un guest.
#[derive(Debug)]
pub struct Session {
    /// Identifiant de session.
    pub id: SessionId,
    /// Code à 6 chiffres.
    pub code: SessionCode,
    /// Slot host.
    pub host: Option<PeerSlot>,
    /// Slot guest.
    pub guest: Option<PeerSlot>,
    /// Dernière activité (create / join / forward).
    pub last_activity: Instant,
}

impl Session {
    /// Nouvelle session vide (sans pairs).
    #[must_use]
    pub fn new(id: SessionId, code: SessionCode) -> Self {
        Self {
            id,
            code,
            host: None,
            guest: None,
            last_activity: Instant::now(),
        }
    }

    /// Touche l’horloge d’activité.
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Nombre de pairs connectés.
    #[must_use]
    pub fn peer_count(&self) -> usize {
        usize::from(self.host.is_some()) + usize::from(self.guest.is_some())
    }

    /// Retourne le slot de l’autre pair, s’il existe.
    #[must_use]
    pub fn other_peer(&self, peer_id: PeerId) -> Option<&PeerSlot> {
        match (&self.host, &self.guest) {
            (Some(h), _) if h.peer_id == peer_id => self.guest.as_ref(),
            (_, Some(g)) if g.peer_id == peer_id => self.host.as_ref(),
            _ => None,
        }
    }

    /// Retire un pair ; retourne `true` s’il était présent.
    pub fn remove_peer(&mut self, peer_id: PeerId) -> bool {
        let mut removed = false;
        if self.host.as_ref().is_some_and(|p| p.peer_id == peer_id) {
            self.host = None;
            removed = true;
        }
        if self.guest.as_ref().is_some_and(|p| p.peer_id == peer_id) {
            self.guest = None;
            removed = true;
        }
        if removed {
            self.touch();
        }
        removed
    }
}
