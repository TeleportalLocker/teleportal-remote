//! Configuration client desktop.

use serde::Serialize;

/// URL WebSocket du relay par défaut.
pub const DEFAULT_RELAY_URL: &str = "wss://relay.teleportal.fr/ws";

/// Config exposée au frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfig {
    /// URL du relay WebSocket.
    pub relay_url: String,
}

impl ClientConfig {
    /// Charge depuis `TELEPORTAL_RELAY_URL` ou la valeur par défaut.
    #[must_use]
    pub fn from_env() -> Self {
        let relay_url =
            std::env::var("TELEPORTAL_RELAY_URL").unwrap_or_else(|_| DEFAULT_RELAY_URL.to_owned());
        Self { relay_url }
    }
}
