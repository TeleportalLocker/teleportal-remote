//! Tracker local + distant avec interpolation.

use teleportal_protocol::{CursorId, Message};

use crate::throttle::SendThrottle;
use crate::types::{CursorPose, SyncConfig};

#[derive(Debug, Clone, Copy)]
struct Sample {
    x: f32,
    y: f32,
    timestamp_ms: u64,
}

/// Synchroniseur de curseurs collaboratifs.
#[derive(Debug)]
pub struct CursorSync {
    config: SyncConfig,
    local_id: CursorId,
    throttle: SendThrottle,
    /// Dernière position locale (même hors throttle).
    last_local: Option<Sample>,
    /// Deux derniers samples distants (plus récent en last).
    remote_prev: Option<Sample>,
    remote_last: Option<Sample>,
    remote_id: Option<CursorId>,
}

impl CursorSync {
    /// Crée un synchroniseur avec nouvel `CursorId` local.
    ///
    /// # Panics
    ///
    /// Jamais si `config` est valide ; ignore une config invalide en appliquant les défauts.
    #[must_use]
    pub fn new(config: SyncConfig) -> Self {
        let config = if config.validate().is_ok() {
            config
        } else {
            SyncConfig::default()
        };
        Self {
            config,
            local_id: CursorId::new(),
            throttle: SendThrottle::new(),
            last_local: None,
            remote_prev: None,
            remote_last: None,
            remote_id: None,
        }
    }

    /// Identifiant du curseur local.
    #[must_use]
    pub const fn local_id(&self) -> CursorId {
        self.local_id
    }

    /// Pousse une position locale ; retourne un `CursorMove` si le throttle autorise l’envoi.
    pub fn push_local(&mut self, x: f32, y: f32, now_ms: u64) -> Option<Message> {
        let x = x.clamp(0.0, 1.0);
        let y = y.clamp(0.0, 1.0);
        self.last_local = Some(Sample {
            x,
            y,
            timestamp_ms: now_ms,
        });
        if !self.throttle.try_send(now_ms, &self.config) {
            return None;
        }
        Some(Message::CursorMove {
            cursor_id: self.local_id,
            x,
            y,
            timestamp_ms: now_ms,
        })
    }

    /// Intègre un message distant (ignore non-`CursorMove` et son propre id).
    pub fn on_remote(&mut self, msg: &Message) {
        let Message::CursorMove {
            cursor_id,
            x,
            y,
            timestamp_ms,
        } = msg
        else {
            return;
        };
        if *cursor_id == self.local_id {
            return;
        }
        let sample = Sample {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            timestamp_ms: *timestamp_ms,
        };
        if self.remote_id != Some(*cursor_id) {
            self.remote_id = Some(*cursor_id);
            self.remote_prev = None;
            self.remote_last = Some(sample);
            return;
        }
        if let Some(last) = self.remote_last {
            if sample.timestamp_ms < last.timestamp_ms {
                return;
            }
            self.remote_prev = Some(last);
        }
        self.remote_last = Some(sample);
    }

    /// Pose distante interpolée à `now_ms`, si disponible.
    #[must_use]
    pub fn remote_pose(&self, now_ms: u64) -> Option<CursorPose> {
        let last = self.remote_last?;
        let cursor_id = self.remote_id?;
        if now_ms.saturating_sub(last.timestamp_ms) > self.config.max_interp_ms {
            return Some(CursorPose {
                cursor_id,
                x: last.x,
                y: last.y,
                timestamp_ms: last.timestamp_ms,
            });
        }
        let (x, y) = match self.remote_prev {
            Some(prev) if last.timestamp_ms > prev.timestamp_ms => {
                let span = (last.timestamp_ms - prev.timestamp_ms) as f32;
                let t = ((now_ms.saturating_sub(prev.timestamp_ms)) as f32 / span).clamp(0.0, 1.0);
                // Au-delà du dernier sample : hold (pas d’extrapolation agressive).
                let t = if now_ms >= last.timestamp_ms { 1.0 } else { t };
                (
                    prev.x + (last.x - prev.x) * t,
                    prev.y + (last.y - prev.y) * t,
                )
            }
            _ => (last.x, last.y),
        };
        Some(CursorPose {
            cursor_id,
            x,
            y,
            timestamp_ms: now_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_local_throttles() {
        let mut sync = CursorSync::new(SyncConfig::default());
        assert!(sync.push_local(0.1, 0.2, 0).is_some());
        assert!(sync.push_local(0.2, 0.3, 10).is_none());
        assert!(sync.push_local(0.3, 0.4, 50).is_some());
    }

    #[test]
    fn ignores_own_cursor_id() {
        let mut sync = CursorSync::new(SyncConfig::default());
        let id = sync.local_id();
        sync.on_remote(&Message::CursorMove {
            cursor_id: id,
            x: 0.5,
            y: 0.5,
            timestamp_ms: 1,
        });
        assert!(sync.remote_pose(1).is_none());
    }

    #[test]
    fn interpolates_between_samples() {
        let mut sync = CursorSync::new(SyncConfig::default());
        let remote = CursorId::new();
        sync.on_remote(&Message::CursorMove {
            cursor_id: remote,
            x: 0.0,
            y: 0.0,
            timestamp_ms: 0,
        });
        sync.on_remote(&Message::CursorMove {
            cursor_id: remote,
            x: 1.0,
            y: 1.0,
            timestamp_ms: 100,
        });
        let mid = sync.remote_pose(50).expect("pose");
        assert!((mid.x - 0.5).abs() < 0.01);
        assert!((mid.y - 0.5).abs() < 0.01);
        let end = sync.remote_pose(100).expect("pose");
        assert!((end.x - 1.0).abs() < 0.01);
    }
}
