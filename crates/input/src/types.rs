//! Types de configuration d’injection.

use crate::error::InputError;

/// Configuration de l’injecteur (écran cible).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InjectConfig {
    /// Largeur de l’écran primaire en pixels.
    pub display_width: u32,
    /// Hauteur de l’écran primaire en pixels.
    pub display_height: u32,
}

impl Default for InjectConfig {
    fn default() -> Self {
        Self {
            display_width: 1920,
            display_height: 1080,
        }
    }
}

impl InjectConfig {
    /// Valide la config.
    ///
    /// # Errors
    ///
    /// Dimensions nulles.
    pub fn validate(&self) -> Result<(), InputError> {
        if self.display_width == 0 || self.display_height == 0 {
            return Err(InputError::InvalidConfig(
                "display dimensions must be >= 1".into(),
            ));
        }
        Ok(())
    }

    /// Convertit des coords normalisées `[0,1]` en pixels.
    #[must_use]
    pub fn to_pixels(&self, x: f32, y: f32) -> (i32, i32) {
        let nx = x.clamp(0.0, 1.0);
        let ny = y.clamp(0.0, 1.0);
        let px = (nx * (self.display_width.saturating_sub(1) as f32)).round() as i32;
        let py = (ny * (self.display_height.saturating_sub(1) as f32)).round() as i32;
        (px, py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_pixels_corners() {
        let cfg = InjectConfig {
            display_width: 100,
            display_height: 50,
        };
        assert_eq!(cfg.to_pixels(0.0, 0.0), (0, 0));
        assert_eq!(cfg.to_pixels(1.0, 1.0), (99, 49));
    }

    #[test]
    fn rejects_zero_dims() {
        assert!(InjectConfig {
            display_width: 0,
            display_height: 10,
        }
        .validate()
        .is_err());
    }
}
