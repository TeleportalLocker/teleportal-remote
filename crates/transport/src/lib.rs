//! Couche transport Client ↔ Relay ↔ Client.
//!
//! Abstraction async indépendante du backend.
//! Phase 2 : duplex in-memory. Phase 3 : client WebSocket. Ultérieurement : QUIC.
//!
//! Point d’extension E2E : chiffrer/déchiffrer les frames avant `send` / après
//! `recv` sans modifier les crates métier (non implémenté en MVP).

#![deny(missing_docs)]
#![warn(clippy::all)]

mod connection;
mod error;
mod memory;
mod websocket;

pub use connection::{recv_message, send_message, Connection};
pub use error::TransportError;
pub use memory::InMemoryConnection;
pub use websocket::WebsocketConnection;

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
