//! Types communs d’encodage / décodage.

use bytes::Bytes;

use crate::error::{DecodeError, EncodeError};

/// Configuration de démarrage de l’encodeur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeConfig {
    /// Plafond FPS annoncé à l’encodeur.
    pub max_fps: u32,
    /// Débit cible en bits/s.
    pub bitrate_bps: u32,
    /// Période d’intra (0 = auto / à la demande uniquement).
    pub keyframe_interval: u32,
}

impl Default for EncodeConfig {
    fn default() -> Self {
        Self {
            max_fps: 30,
            // ~4 Mbit/s — cible MVP 1080p30 screen share
            bitrate_bps: 4_000_000,
            keyframe_interval: 60,
        }
    }
}

impl EncodeConfig {
    /// Valide la config.
    ///
    /// # Errors
    ///
    /// `max_fps == 0` ou `bitrate_bps == 0`.
    pub fn validate(&self) -> Result<(), EncodeError> {
        if self.max_fps == 0 {
            return Err(EncodeError::InvalidConfig("max_fps must be >= 1".into()));
        }
        if self.bitrate_bps == 0 {
            return Err(EncodeError::InvalidConfig(
                "bitrate_bps must be >= 1".into(),
            ));
        }
        Ok(())
    }
}

/// Configuration de démarrage du décodeur.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecodeConfig {}

impl DecodeConfig {
    /// Valide la config (MVP : toujours OK).
    ///
    /// # Errors
    ///
    /// Réservé pour options futures.
    pub fn validate(&self) -> Result<(), DecodeError> {
        Ok(())
    }
}

/// Frame H264 Annex-B prête pour le protocole.
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    /// Identifiant monotone de frame.
    pub frame_id: u64,
    /// Horodatage Unix ms (repris de la frame source).
    pub timestamp_ms: u64,
    /// Largeur encodée (paire).
    pub width: u32,
    /// Hauteur encodée (paire).
    pub height: u32,
    /// `true` si IDR / I.
    pub is_keyframe: bool,
    /// Payload Annex-B (start codes `00 00 00 01`).
    pub data: Bytes,
}

/// Frame décodée RGBA8 dense (canvas-ready).
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    /// Identifiant monotone (repris du message).
    pub frame_id: u64,
    /// Horodatage Unix ms.
    pub timestamp_ms: u64,
    /// Largeur.
    pub width: u32,
    /// Hauteur.
    pub height: u32,
    /// Pixels RGBA (stride = `width * 4`).
    pub data: Bytes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        assert!(EncodeConfig::default().validate().is_ok());
        assert!(DecodeConfig::default().validate().is_ok());
    }

    #[test]
    fn rejects_zero_fps_or_bitrate() {
        assert!(matches!(
            EncodeConfig {
                max_fps: 0,
                ..EncodeConfig::default()
            }
            .validate(),
            Err(EncodeError::InvalidConfig(_))
        ));
        assert!(matches!(
            EncodeConfig {
                bitrate_bps: 0,
                ..EncodeConfig::default()
            }
            .validate(),
            Err(EncodeError::InvalidConfig(_))
        ));
    }
}
