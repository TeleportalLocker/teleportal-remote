//! Identifiants et code de session.

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::error::ProtocolError;

/// Identifiant de session (UUID v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Génère un nouvel identifiant de session.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Accès à l’UUID sous-jacent.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Identifiant de pair (UUID v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PeerId(Uuid);

impl PeerId {
    /// Génère un nouvel identifiant de pair.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Accès à l’UUID sous-jacent.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for PeerId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Identifiant de curseur collaboratif (UUID v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CursorId(Uuid);

impl CursorId {
    /// Génère un nouvel identifiant de curseur.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Accès à l’UUID sous-jacent.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for CursorId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CursorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Code de session à exactement 6 chiffres ASCII.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionCode(String);

impl SessionCode {
    /// Construit un code après validation (exactement 6 chiffres).
    ///
    /// # Errors
    ///
    /// Retourne [`ProtocolError::InvalidIdentifier`] si le format est invalide.
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, ProtocolError> {
        let raw = raw.as_ref();
        if raw.len() != 6 || !raw.chars().all(|c| c.is_ascii_digit()) {
            return Err(ProtocolError::InvalidIdentifier(format!(
                "session code must be exactly 6 ASCII digits, got {raw:?}"
            )));
        }
        Ok(Self(raw.to_owned()))
    }

    /// Représentation textuelle du code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for SessionCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SessionCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SessionCodeVisitor;

        impl Visitor<'_> for SessionCodeVisitor {
            type Value = SessionCode;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a 6-digit session code")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                SessionCode::parse(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(SessionCodeVisitor)
    }
}

impl std::fmt::Display for SessionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_code_accepts_six_digits() {
        assert!(SessionCode::parse("123456").is_ok());
    }

    #[test]
    fn session_code_rejects_invalid() {
        assert!(SessionCode::parse("12345").is_err());
        assert!(SessionCode::parse("1234567").is_err());
        assert!(SessionCode::parse("12a456").is_err());
        assert!(SessionCode::parse("").is_err());
    }
}
