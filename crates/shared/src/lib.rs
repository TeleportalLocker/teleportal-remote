//! Types, constantes et utilitaires partagés par tous les crates Teleportal Remote.
//!
//! Ce crate est le socle commun du monorepo : version, observabilité de base,
//! et helpers transverses sans logique métier.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod observability;

/// Version du produit alignée sur le workspace (`0.1.0` en Phase 1).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver_like() {
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())));
    }
}
