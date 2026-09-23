//! Stub de capture hors Windows / macOS.

use crate::error::CaptureError;
use crate::traits::Capturer;
use crate::types::{CaptureConfig, DisplayInfo, Frame};

/// Capturer non implémenté hors Windows / macOS.
#[derive(Debug, Default)]
pub struct UnsupportedCapturer;

impl Capturer for UnsupportedCapturer {
    fn displays() -> Result<Vec<DisplayInfo>, CaptureError> {
        Err(CaptureError::UnsupportedPlatform)
    }

    fn start(_config: CaptureConfig) -> Result<Self, CaptureError> {
        Err(CaptureError::UnsupportedPlatform)
    }

    fn grab(&mut self) -> Result<Option<Frame>, CaptureError> {
        Err(CaptureError::UnsupportedPlatform)
    }

    fn stop(self) -> Result<(), CaptureError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_fails_on_unsupported_os() {
        let err = UnsupportedCapturer::start(CaptureConfig::default()).unwrap_err();
        assert!(matches!(err, CaptureError::UnsupportedPlatform));
    }
}
