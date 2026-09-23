//! Types de pose et configuration sync.

use teleportal_protocol::CursorId;

use crate::error::CursorError;

/// Pose de curseur (coords normalisées).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorPose {
    /// Identifiant du curseur.
    pub cursor_id: CursorId,
    /// Abscisse `[0,1]`.
    pub x: f32,
    /// Ordonnée `[0,1]`.
    pub y: f32,
    /// Horodatage Unix ms (échantillon ou interpolé).
    pub timestamp_ms: u64,
}

/// Configuration du synchroniseur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyncConfig {
    /// Fréquence d’envoi cible (Hz).
    pub send_hz: u32,
    /// Durée max d’extrapolation / hold après le dernier sample (ms).
    pub max_interp_ms: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            send_hz: 20,
            max_interp_ms: 200,
        }
    }
}

impl SyncConfig {
    /// Valide la config.
    ///
    /// # Errors
    ///
    /// `send_hz` nul.
    pub fn validate(&self) -> Result<(), CursorError> {
        if self.send_hz == 0 {
            return Err(CursorError::InvalidConfig("send_hz must be >= 1".into()));
        }
        Ok(())
    }

    /// Intervalle minimum entre deux envois (ms).
    #[must_use]
    pub fn send_interval_ms(&self) -> u64 {
        1000 / u64::from(self.send_hz.max(1))
    }
}

/// Normalise des pixels vers `[0,1]`.
#[must_use]
pub fn normalize_pixels(px: i32, py: i32, display_w: u32, display_h: u32) -> (f32, f32) {
    let max_x = display_w.saturating_sub(1).max(1) as f32;
    let max_y = display_h.saturating_sub(1).max(1) as f32;
    let x = (px as f32 / max_x).clamp(0.0, 1.0);
    let y = (py as f32 / max_y).clamp(0.0, 1.0);
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_corners() {
        assert_eq!(normalize_pixels(0, 0, 100, 50), (0.0, 0.0));
        assert_eq!(normalize_pixels(99, 49, 100, 50), (1.0, 1.0));
    }

    #[test]
    fn default_interval_20hz() {
        assert_eq!(SyncConfig::default().send_interval_ms(), 50);
    }
}
