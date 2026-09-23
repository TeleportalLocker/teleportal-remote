//! Boucle de nettoyage des sessions expirées.

use std::time::Duration;

use bytes::Bytes;
use teleportal_protocol::{encode_message, Message};
use tracing::info;

use crate::state::SessionRegistry;

/// Boucle de sweep TTL.
pub async fn run_cleanup_loop(registry: SessionRegistry, ttl: Duration, interval: Duration) {
    let mut ticker = tokio::time::interval(interval);
    loop {
        ticker.tick().await;
        let notify = registry.expire_stale(ttl).await;
        if !notify.is_empty() {
            info!(count = notify.len(), "expired stale sessions");
        }
        let frame = match encode_message(&Message::SessionExpired) {
            Ok(f) => Bytes::from(f),
            Err(_) => continue,
        };
        for tx in notify {
            let _ = tx.try_send(frame.clone());
        }
    }
}

/// Spawn la tâche de cleanup en arrière-plan.
pub fn spawn_cleanup(registry: SessionRegistry, ttl: Duration, interval: Duration) {
    tokio::spawn(async move {
        run_cleanup_loop(registry, ttl, interval).await;
    });
}
