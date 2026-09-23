//! Registre de sessions en mémoire.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use teleportal_protocol::{encode_message, Message, PeerId, Role, SessionCode, SessionId};
use tokio::sync::{mpsc, RwLock};

use crate::code::generate_session_code;
use crate::session::{PeerSlot, Session};

/// Erreurs du registre.
#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// Code inconnu ou session absente.
    NotFound,
    /// Slot guest déjà pris / session pleine.
    SessionFull,
    /// Impossible de générer un code unique.
    CodeExhausted,
    /// Échec d’encodage protocolaire.
    Encode(String),
}

/// Registre partagé des sessions actives.
#[derive(Debug, Default, Clone)]
pub struct SessionRegistry {
    inner: Arc<RwLock<RegistryInner>>,
}

#[derive(Debug, Default)]
struct RegistryInner {
    by_id: HashMap<SessionId, Session>,
    by_code: HashMap<String, SessionId>,
}

impl SessionRegistry {
    /// Crée un registre vide.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Crée une session et y attache le host.
    pub async fn create_session(
        &self,
        host_outbound: mpsc::Sender<Bytes>,
    ) -> Result<(SessionId, SessionCode, PeerId), RegistryError> {
        let mut guard = self.inner.write().await;
        let mut code = None;
        for _ in 0..32 {
            let candidate =
                generate_session_code().map_err(|e| RegistryError::Encode(e.to_string()))?;
            if !guard.by_code.contains_key(candidate.as_str()) {
                code = Some(candidate);
                break;
            }
        }
        let code = code.ok_or(RegistryError::CodeExhausted)?;
        let session_id = SessionId::new();
        let peer_id = PeerId::new();
        let mut session = Session::new(session_id, code.clone());
        session.host = Some(PeerSlot {
            peer_id,
            role: Role::Host,
            outbound: host_outbound,
        });
        guard.by_code.insert(code.as_str().to_owned(), session_id);
        guard.by_id.insert(session_id, session);
        Ok((session_id, code, peer_id))
    }

    /// Fait rejoindre un guest via code.
    pub async fn join_session(
        &self,
        code: &SessionCode,
        guest_outbound: mpsc::Sender<Bytes>,
    ) -> Result<(SessionId, PeerId, PeerId), RegistryError> {
        let mut guard = self.inner.write().await;
        let session_id = *guard
            .by_code
            .get(code.as_str())
            .ok_or(RegistryError::NotFound)?;
        let session = guard
            .by_id
            .get_mut(&session_id)
            .ok_or(RegistryError::NotFound)?;
        if session.guest.is_some() {
            return Err(RegistryError::SessionFull);
        }
        let guest_id = PeerId::new();
        let host_id = session
            .host
            .as_ref()
            .map(|h| h.peer_id)
            .ok_or(RegistryError::NotFound)?;
        session.guest = Some(PeerSlot {
            peer_id: guest_id,
            role: Role::Guest,
            outbound: guest_outbound,
        });
        session.touch();
        Ok((session_id, guest_id, host_id))
    }

    /// Envoie un message encodé à l’autre pair de la session.
    pub async fn forward_to_other(
        &self,
        session_id: SessionId,
        from: PeerId,
        message: &Message,
    ) -> Result<(), RegistryError> {
        let frame = encode_message(message).map_err(|e| RegistryError::Encode(e.to_string()))?;
        let guard = self.inner.read().await;
        let session = guard
            .by_id
            .get(&session_id)
            .ok_or(RegistryError::NotFound)?;
        let other = session.other_peer(from).ok_or(RegistryError::NotFound)?;
        let _ = other.outbound.try_send(Bytes::from(frame));
        Ok(())
    }

    /// Notifie un pair précis (signaling).
    pub async fn send_to_peer(
        &self,
        session_id: SessionId,
        peer_id: PeerId,
        message: &Message,
    ) -> Result<(), RegistryError> {
        let frame = encode_message(message).map_err(|e| RegistryError::Encode(e.to_string()))?;
        let guard = self.inner.read().await;
        let session = guard
            .by_id
            .get(&session_id)
            .ok_or(RegistryError::NotFound)?;
        let target = match (&session.host, &session.guest) {
            (Some(h), _) if h.peer_id == peer_id => h,
            (_, Some(g)) if g.peer_id == peer_id => g,
            _ => return Err(RegistryError::NotFound),
        };
        let _ = target.outbound.try_send(Bytes::from(frame));
        Ok(())
    }

    /// Touche l’activité d’une session.
    pub async fn touch(&self, session_id: SessionId) {
        let mut guard = self.inner.write().await;
        if let Some(session) = guard.by_id.get_mut(&session_id) {
            session.touch();
        }
    }

    /// Retire un pair ; détruit la session si vide. Notifie l’autre avec `PeerLeft`.
    pub async fn detach_peer(&self, session_id: SessionId, peer_id: PeerId) {
        let other_outbound_and_msg = {
            let mut guard = self.inner.write().await;
            let Some(session) = guard.by_id.get_mut(&session_id) else {
                return;
            };
            let other = session
                .other_peer(peer_id)
                .map(|p| (p.outbound.clone(), Message::PeerLeft { peer_id }));
            session.remove_peer(peer_id);
            let empty = session.peer_count() == 0;
            let code = session.code.as_str().to_owned();
            if empty {
                guard.by_id.remove(&session_id);
                guard.by_code.remove(&code);
            }
            other
        };

        if let Some((tx, msg)) = other_outbound_and_msg {
            if let Ok(frame) = encode_message(&msg) {
                let _ = tx.try_send(Bytes::from(frame));
            }
        }
    }

    /// Expire les sessions inactives ; retourne les outbounds à notifier `SessionExpired`.
    pub async fn expire_stale(&self, ttl: Duration) -> Vec<mpsc::Sender<Bytes>> {
        let now = Instant::now();
        let mut guard = self.inner.write().await;
        let stale: Vec<SessionId> = guard
            .by_id
            .iter()
            .filter(|(_, s)| now.duration_since(s.last_activity) > ttl)
            .map(|(id, _)| *id)
            .collect();

        let mut notify = Vec::new();
        for id in stale {
            if let Some(session) = guard.by_id.remove(&id) {
                guard.by_code.remove(session.code.as_str());
                if let Some(h) = session.host {
                    notify.push(h.outbound);
                }
                if let Some(g) = session.guest {
                    notify.push(g.outbound);
                }
            }
        }
        notify
    }

    /// Nombre de sessions (tests).
    #[cfg(test)]
    pub async fn session_count(&self) -> usize {
        self.inner.read().await.by_id.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn create_and_join() {
        let reg = SessionRegistry::new();
        let (tx_h, _rx_h) = mpsc::channel(8);
        let (sid, code, _hid) = reg.create_session(tx_h).await.unwrap();
        let (tx_g, _rx_g) = mpsc::channel(8);
        let (sid2, _gid, _host) = reg.join_session(&code, tx_g).await.unwrap();
        assert_eq!(sid, sid2);
        assert_eq!(reg.session_count().await, 1);
    }

    #[tokio::test]
    async fn join_unknown_fails() {
        let reg = SessionRegistry::new();
        let (tx, _) = mpsc::channel(1);
        let code = SessionCode::parse("000000").unwrap();
        assert_eq!(
            reg.join_session(&code, tx).await.unwrap_err(),
            RegistryError::NotFound
        );
    }

    #[tokio::test]
    async fn second_guest_rejected() {
        let reg = SessionRegistry::new();
        let (tx_h, _) = mpsc::channel(1);
        let (_, code, _) = reg.create_session(tx_h).await.unwrap();
        let (tx_g1, _) = mpsc::channel(1);
        reg.join_session(&code, tx_g1).await.unwrap();
        let (tx_g2, _) = mpsc::channel(1);
        assert_eq!(
            reg.join_session(&code, tx_g2).await.unwrap_err(),
            RegistryError::SessionFull
        );
    }

    #[tokio::test]
    async fn expire_stale_removes_session() {
        let reg = SessionRegistry::new();
        let (tx_h, _) = mpsc::channel(1);
        reg.create_session(tx_h).await.unwrap();
        // Force expiry by using zero TTL after touch in past — expire with 0 duration
        tokio::time::sleep(Duration::from_millis(5)).await;
        let notified = reg.expire_stale(Duration::from_millis(1)).await;
        assert_eq!(reg.session_count().await, 0);
        assert!(!notified.is_empty());
    }
}
