//! Génération de codes de session à 6 chiffres.

use rand::Rng;
use teleportal_protocol::{ProtocolError, SessionCode};

/// Génère un code aléatoire à 6 chiffres.
///
/// # Errors
///
/// Ne devrait jamais échouer si `rand` est sain ; propage [`ProtocolError`] si parse échoue.
pub fn generate_session_code() -> Result<SessionCode, ProtocolError> {
    let n: u32 = rand::rng().random_range(0..1_000_000);
    SessionCode::parse(format!("{n:06}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn codes_are_six_digits() {
        for _ in 0..32 {
            let code = generate_session_code().unwrap();
            assert_eq!(code.as_str().len(), 6);
            assert!(code.as_str().chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn codes_have_variety() {
        let mut set = HashSet::new();
        for _ in 0..64 {
            set.insert(generate_session_code().unwrap().as_str().to_owned());
        }
        assert!(set.len() > 1);
    }
}
