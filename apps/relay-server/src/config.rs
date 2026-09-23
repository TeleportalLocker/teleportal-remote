//! Configuration runtime du relay.

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

/// Paramètres du serveur relay.
#[derive(Debug, Clone)]
pub struct Config {
    /// Adresse d’écoute HTTP/WS.
    pub bind: SocketAddr,
    /// Durée de vie max d’une session sans activité.
    pub session_ttl: Duration,
    /// Intervalle du sweep d’expiration.
    pub cleanup_interval: Duration,
    /// Tentatives Create/Join max par IP et par minute (`0` = désactivé).
    pub rate_limit_per_min: u32,
}

impl Config {
    /// Charge la config depuis l’environnement (avec défauts MVP).
    ///
    /// - `TELEPORTAL_RELAY_BIND` (défaut `0.0.0.0:7800`)
    /// - `TELEPORTAL_SESSION_TTL_SECS` (défaut `600`)
    /// - `TELEPORTAL_RATE_LIMIT_PER_MIN` (défaut `30`)
    pub fn from_env() -> anyhow::Result<Self> {
        let bind: SocketAddr = env::var("TELEPORTAL_RELAY_BIND")
            .unwrap_or_else(|_| "0.0.0.0:7800".into())
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid TELEPORTAL_RELAY_BIND: {e}"))?;

        let ttl_secs: u64 = env::var("TELEPORTAL_SESSION_TTL_SECS")
            .unwrap_or_else(|_| "600".into())
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid TELEPORTAL_SESSION_TTL_SECS: {e}"))?;

        let rate_limit_per_min: u32 = env::var("TELEPORTAL_RATE_LIMIT_PER_MIN")
            .unwrap_or_else(|_| "30".into())
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid TELEPORTAL_RATE_LIMIT_PER_MIN: {e}"))?;

        Ok(Self {
            bind,
            session_ttl: Duration::from_secs(ttl_secs),
            cleanup_interval: Duration::from_secs(30),
            rate_limit_per_min,
        })
    }
}
