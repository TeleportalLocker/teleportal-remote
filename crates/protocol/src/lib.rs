//! Définition des messages et de la sérialisation du protocole Teleportal.
//!
//! Format filaire : length-prefixed (`u32` big-endian) + MessagePack.
//! L’exemple JSON du document de référence reste illustratif uniquement.

#![deny(missing_docs)]
#![warn(clippy::all)]

mod codec;
mod error;
mod ids;
mod messages;
mod role;
mod validate;

pub use codec::{decode_message, encode_message, peek_payload_len};
pub use error::ProtocolError;
pub use ids::{CursorId, PeerId, SessionCode, SessionId};
pub use messages::{KeyModifiers, Message, MouseButton, VideoCodec, PROTOCOL_VERSION};
pub use role::Role;
pub use validate::{validate_message, MAX_FRAME_BYTES, MAX_VIDEO_PAYLOAD};

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
