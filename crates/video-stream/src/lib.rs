//! Pipeline de streaming vidéo Teleportal Remote.
//!
//! Phase 7–8 : encode / decode H264 (OpenH264) depuis frames BGRA8.
//! Abstraction prête pour backends hardware futurs.
//! Affichage canvas Tauri : client desktop (Phase 8).

#![deny(missing_docs)]
#![warn(clippy::all)]

mod error;
mod openh264_dec;
mod openh264_enc;
mod traits;
mod types;
mod wire;

pub use error::{DecodeError, EncodeError};
pub use openh264_dec::OpenH264Decoder;
pub use openh264_enc::OpenH264Encoder;
pub use traits::{VideoDecoder, VideoEncoder};
pub use types::{DecodeConfig, DecodedFrame, EncodeConfig, EncodedFrame};
pub use wire::to_video_message;

/// Alias MVP : encodeur logiciel multiplateforme.
pub type PlatformEncoder = OpenH264Encoder;

/// Alias MVP : décodeur logiciel multiplateforme.
pub type PlatformDecoder = OpenH264Decoder;

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
}
