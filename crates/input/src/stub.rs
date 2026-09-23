//! Stub hors Windows / macOS.

use teleportal_protocol::Message;

use crate::error::InputError;
use crate::traits::InputInjector;
use crate::types::InjectConfig;

/// Injecteur non implémenté hors Windows / macOS.
#[derive(Debug, Default)]
pub struct UnsupportedInjector;

impl InputInjector for UnsupportedInjector {
    fn start(_config: InjectConfig) -> Result<Self, InputError> {
        Err(InputError::UnsupportedPlatform)
    }

    fn inject(&mut self, _msg: &Message) -> Result<(), InputError> {
        Err(InputError::UnsupportedPlatform)
    }

    fn stop(self) -> Result<(), InputError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_fails() {
        let err = UnsupportedInjector::start(InjectConfig::default()).unwrap_err();
        assert!(matches!(err, InputError::UnsupportedPlatform));
    }
}
