//! Synchronisation des curseurs collaboratifs (local + distant).
//!
//! Protocole `CursorMove`, envoi ~20 Hz, interpolation fluide côté récepteur.

#![deny(missing_docs)]
#![warn(clippy::all)]

mod error;
mod platform;
mod throttle;
mod tracker;
mod types;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(any(windows, target_os = "macos")))]
mod stub;

pub use error::CursorError;
pub use platform::poll_os_cursor;
pub use tracker::CursorSync;
pub use types::{normalize_pixels, CursorPose, SyncConfig};

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
    fn poll_unsupported() {
        let err = poll_os_cursor(1920, 1080).unwrap_err();
        assert!(matches!(err, CursorError::UnsupportedPlatform));
    }
}
