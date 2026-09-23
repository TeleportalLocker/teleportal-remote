//! Contrôle distant clavier, souris et scroll.
//!
//! - Windows : SendInput
//! - macOS : CGEvent (+ permission Accessibility)

#![deny(missing_docs)]
#![warn(clippy::all)]

mod error;
mod mapping;
mod traits;
mod types;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(any(windows, target_os = "macos")))]
mod stub;

pub use error::InputError;
pub use mapping::{mac_keycode, win_vk, MacKeyCode, WinVk};
pub use traits::InputInjector;
pub use types::InjectConfig;

#[cfg(windows)]
pub use windows::WinInjector;

#[cfg(target_os = "macos")]
pub use macos::MacInjector;

#[cfg(not(any(windows, target_os = "macos")))]
pub use stub::UnsupportedInjector;

#[cfg(windows)]
/// Injecteur natif de la plateforme courante.
pub type PlatformInjector = WinInjector;

#[cfg(target_os = "macos")]
/// Injecteur natif de la plateforme courante.
pub type PlatformInjector = MacInjector;

#[cfg(not(any(windows, target_os = "macos")))]
/// Injecteur natif de la plateforme courante.
pub type PlatformInjector = UnsupportedInjector;

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
    fn platform_unsupported() {
        let err = PlatformInjector::start(InjectConfig::default()).unwrap_err();
        assert!(matches!(err, InputError::UnsupportedPlatform));
    }
}
