//! Capture d’écran multiplateforme pour Teleportal Remote.
//!
//! - Windows : DXGI Desktop Duplication API (Phase 5)
//! - macOS : ScreenCaptureKit (Phase 6)

#![deny(missing_docs)]
#![warn(clippy::all)]

mod error;
mod traits;
mod types;

#[cfg(windows)]
mod dxgi;

#[cfg(target_os = "macos")]
mod sck;

#[cfg(not(any(windows, target_os = "macos")))]
mod stub;

pub use error::CaptureError;
pub use traits::Capturer;
pub use types::{CaptureConfig, DisplayId, DisplayInfo, Frame, PixelFormat};

#[cfg(windows)]
pub use dxgi::DxgiCapturer;

#[cfg(target_os = "macos")]
pub use sck::SckCapturer;

#[cfg(not(any(windows, target_os = "macos")))]
pub use stub::UnsupportedCapturer;

#[cfg(windows)]
/// Capturer natif de la plateforme courante.
pub type PlatformCapturer = DxgiCapturer;

#[cfg(target_os = "macos")]
/// Capturer natif de la plateforme courante.
pub type PlatformCapturer = SckCapturer;

#[cfg(not(any(windows, target_os = "macos")))]
/// Capturer natif de la plateforme courante.
pub type PlatformCapturer = UnsupportedCapturer;

use teleportal_shared::VERSION;

/// Retourne la version workspace du crate.
#[must_use]
pub fn crate_version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_shared_version() {
        assert_eq!(crate_version(), teleportal_shared::VERSION);
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    #[test]
    fn platform_start_unsupported() {
        let err = PlatformCapturer::start(CaptureConfig::default()).unwrap_err();
        assert!(matches!(err, CaptureError::UnsupportedPlatform));
    }
}
