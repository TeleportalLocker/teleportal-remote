//! Binaire du serveur relay Teleportal Remote.

use std::sync::Arc;

use teleportal_relay_server::cleanup::spawn_cleanup;
use teleportal_relay_server::rate_limit::RateLimiter;
use teleportal_relay_server::ws::{serve, AppState};
use teleportal_relay_server::{Config, SessionRegistry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    teleportal_shared::observability::init_tracing();

    let config = Config::from_env()?;
    tracing::info!(
        version = teleportal_shared::VERSION,
        bind = %config.bind,
        ttl_secs = config.session_ttl.as_secs(),
        rate_limit_per_min = config.rate_limit_per_min,
        "starting teleportal-relay-server"
    );

    let registry = SessionRegistry::new();
    spawn_cleanup(
        registry.clone(),
        config.session_ttl,
        config.cleanup_interval,
    );

    let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_per_min));
    let state = AppState {
        registry,
        config: Arc::new(config.clone()),
        rate_limiter,
    };

    serve(config.bind, state).await
}
