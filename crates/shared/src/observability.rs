//! Observabilité : initialisation tracing et points d’extension futurs.
//!
//! Phase 1 : subscriber `tracing` console uniquement.
//! Phases ultérieures : abstractions prêtes pour OpenTelemetry, Loki, Tempo, Pyroscope
//! (non implémentées ici).

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialise le subscriber `tracing` global.
///
/// Lit `RUST_LOG` si présent, sinon `info` par défaut.
///
/// # Panics
///
/// Panique si un subscriber global est déjà installé (appel unique attendu au démarrage).
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}

/// Placeholder pour une future intégration OpenTelemetry.
///
/// Aucune implémentation en Phase 1 : l’API documente l’intention architecturale.
pub fn init_opentelemetry_placeholder() {
    tracing::debug!("OpenTelemetry integration not implemented yet");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opentelemetry_placeholder_is_callable() {
        init_opentelemetry_placeholder();
    }
}
