//! Throttle d’envoi ~20 Hz.

use crate::types::SyncConfig;

/// Garde le dernier instant d’envoi.
#[derive(Debug, Clone, Copy, Default)]
pub struct SendThrottle {
    last_sent_ms: Option<u64>,
}

impl SendThrottle {
    /// Crée un throttle vierge.
    #[must_use]
    pub const fn new() -> Self {
        Self { last_sent_ms: None }
    }

    /// Indique si un envoi est autorisé à `now_ms` ; si oui, enregistre l’instant.
    pub fn try_send(&mut self, now_ms: u64, config: &SyncConfig) -> bool {
        let interval = config.send_interval_ms();
        match self.last_sent_ms {
            Some(last) if now_ms.saturating_sub(last) < interval => false,
            _ => {
                self.last_sent_ms = Some(now_ms);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throttles_to_20hz() {
        let cfg = SyncConfig::default();
        let mut t = SendThrottle::new();
        assert!(t.try_send(0, &cfg));
        assert!(!t.try_send(40, &cfg));
        assert!(t.try_send(50, &cfg));
        assert!(!t.try_send(99, &cfg));
        assert!(t.try_send(100, &cfg));
    }
}
