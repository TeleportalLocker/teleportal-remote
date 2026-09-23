//! Auto-update du client (canal `stable`, téléchargement en arrière-plan).
//!
//! Le runtime d’update est fourni par `tauri-plugin-updater` dans le desktop-client.
//! Ce crate expose les constantes et helpers partagés (canal, endpoint, semver).

#![deny(missing_docs)]
#![warn(clippy::all)]

use teleportal_shared::VERSION;

/// Canal de mise à jour produit (MVP).
pub const STABLE_CHANNEL: &str = "stable";

/// URL du manifeste GitHub Releases (`latest.json`).
pub const LATEST_JSON_URL: &str =
    "https://github.com/teleportal/teleportal-remote/releases/latest/download/latest.json";

/// Retourne la version workspace (smoke link vers `teleportal-shared`).
#[must_use]
pub fn crate_version() -> &'static str {
    VERSION
}

/// Canal configuré pour le client.
#[must_use]
pub fn stable_channel() -> &'static str {
    STABLE_CHANNEL
}

/// Endpoint manifeste stable.
#[must_use]
pub fn latest_manifest_url() -> &'static str {
    LATEST_JSON_URL
}

/// Vérifie qu’une chaîne ressemble à un semver `MAJOR.MINOR.PATCH` (suffixe pré-release ignore).
#[must_use]
pub fn is_semver_like(version: &str) -> bool {
    let core = version.split('-').next().unwrap_or(version);
    let mut parts = core.split('.');
    let Some(major) = parts.next() else {
        return false;
    };
    let Some(minor) = parts.next() else {
        return false;
    };
    let Some(patch) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    major.parse::<u32>().is_ok() && minor.parse::<u32>().is_ok() && patch.parse::<u32>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_shared_version() {
        assert_eq!(crate_version(), teleportal_shared::VERSION);
    }

    #[test]
    fn stable_channel_is_stable() {
        assert_eq!(stable_channel(), "stable");
    }

    #[test]
    fn latest_url_points_at_github_releases() {
        assert!(latest_manifest_url().ends_with("/latest.json"));
        assert!(latest_manifest_url().contains("teleportal-remote"));
    }

    #[test]
    fn semver_like_accepts_workspace_version() {
        assert!(is_semver_like(crate_version()));
        assert!(is_semver_like("1.2.3"));
        assert!(is_semver_like("0.1.0-beta.1"));
        assert!(!is_semver_like(""));
        assert!(!is_semver_like("1.2"));
        assert!(!is_semver_like("a.b.c"));
    }
}
