//! Types communs de capture.

use bytes::Bytes;

use crate::error::CaptureError;

/// Identifiant opaque d’un écran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisplayId(pub u32);

/// Description d’un écran capturable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayInfo {
    /// Identifiant stable pour la session de capture.
    pub id: DisplayId,
    /// Index d’énumération (0 = primaire MVP).
    pub index: usize,
    /// Nom lisible (device / output).
    pub name: String,
    /// Largeur en pixels.
    pub width: u32,
    /// Hauteur en pixels.
    pub height: u32,
}

/// Format pixel des frames livrées.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// 8 bits par canal, ordre B, G, R, A.
    Bgra8,
}

/// Configuration de démarrage de capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureConfig {
    /// Index d’écran (MVP : un seul à la fois).
    pub display_index: usize,
    /// Plafond de fréquence d’acquisition.
    pub max_fps: u32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            display_index: 0,
            max_fps: 30,
        }
    }
}

impl CaptureConfig {
    /// Valide la config.
    ///
    /// # Errors
    ///
    /// `max_fps == 0`.
    pub fn validate(&self) -> Result<(), CaptureError> {
        if self.max_fps == 0 {
            return Err(CaptureError::InvalidConfig("max_fps must be >= 1".into()));
        }
        Ok(())
    }

    /// Timeout `AcquireNextFrame` dérivé de `max_fps`.
    #[must_use]
    pub fn frame_timeout_ms(&self) -> u32 {
        (1000 / self.max_fps.max(1)).max(1)
    }
}

/// Frame capturée (BGRA CPU).
#[derive(Debug, Clone)]
pub struct Frame {
    /// Largeur.
    pub width: u32,
    /// Hauteur.
    pub height: u32,
    /// Stride en octets (peut être > `width * 4`).
    pub stride: usize,
    /// Format.
    pub format: PixelFormat,
    /// Horodatage Unix ms.
    pub timestamp_ms: u64,
    /// Écran source.
    pub display_id: DisplayId,
    /// Pixels BGRA.
    pub data: Bytes,
}

impl Frame {
    /// Nombre d’octets attendus au minimum.
    #[must_use]
    pub fn expected_min_len(&self) -> usize {
        self.stride.saturating_mul(self.height as usize)
    }

    /// Vérifie cohérence basique largeur/hauteur/stride/data.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.stride >= (self.width as usize).saturating_mul(4)
            && self.data.len() >= self.expected_min_len()
            && self.format == PixelFormat::Bgra8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = CaptureConfig::default();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.display_index, 0);
        assert_eq!(cfg.max_fps, 30);
        assert_eq!(cfg.frame_timeout_ms(), 33);
    }

    #[test]
    fn rejects_zero_fps() {
        let cfg = CaptureConfig {
            display_index: 0,
            max_fps: 0,
        };
        assert!(matches!(
            cfg.validate(),
            Err(CaptureError::InvalidConfig(_))
        ));
    }

    #[test]
    fn frame_well_formed() {
        let width = 2u32;
        let height = 2u32;
        let stride = 8usize;
        let frame = Frame {
            width,
            height,
            stride,
            format: PixelFormat::Bgra8,
            timestamp_ms: 1,
            display_id: DisplayId(0),
            data: Bytes::from(vec![0u8; stride * height as usize]),
        };
        assert!(frame.is_well_formed());
    }
}
