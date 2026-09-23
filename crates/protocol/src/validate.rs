//! Validation des messages MVP.

use crate::error::ProtocolError;
use crate::messages::{Message, VideoCodec, PROTOCOL_VERSION};

/// Taille maximale d’une frame filaire (en-tête + payload), 4 MiB.
pub const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;

/// Taille maximale du payload vidéo seul.
pub const MAX_VIDEO_PAYLOAD: usize = MAX_FRAME_BYTES - 256;

fn coords_ok(x: f32, y: f32) -> bool {
    (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) && x.is_finite() && y.is_finite()
}

/// Valide un message selon les règles MVP.
///
/// # Errors
///
/// Retourne [`ProtocolError::Validation`] si une règle est violée.
pub fn validate_message(message: &Message) -> Result<(), ProtocolError> {
    match message {
        Message::Hello {
            protocol_version, ..
        } => {
            if *protocol_version != PROTOCOL_VERSION {
                return Err(ProtocolError::Validation(format!(
                    "unsupported protocol_version {protocol_version}, expected {PROTOCOL_VERSION}"
                )));
            }
        }
        Message::JoinRejected { reason }
        | Message::Error {
            message: reason, ..
        } => {
            if reason.is_empty() {
                return Err(ProtocolError::Validation(
                    "reason/message must not be empty".into(),
                ));
            }
        }
        Message::CursorMove { x, y, .. }
        | Message::MouseMove { x, y, .. }
        | Message::MouseButton { x, y, .. }
        | Message::MouseScroll { x, y, .. } => {
            if !coords_ok(*x, *y) {
                return Err(ProtocolError::Validation(format!(
                    "coordinates out of range: ({x}, {y})"
                )));
            }
        }
        Message::KeyEvent { key, .. } => {
            if key.is_empty() {
                return Err(ProtocolError::Validation("key must not be empty".into()));
            }
        }
        Message::VideoFrame {
            width,
            height,
            codec,
            data,
            ..
        } => {
            if *codec != VideoCodec::H264 {
                return Err(ProtocolError::Validation("MVP accepts H264 only".into()));
            }
            if *width == 0 || *height == 0 {
                return Err(ProtocolError::Validation(
                    "video frame dimensions must be non-zero".into(),
                ));
            }
            if data.len() > MAX_VIDEO_PAYLOAD {
                return Err(ProtocolError::FrameTooLarge {
                    size: data.len(),
                    max: MAX_VIDEO_PAYLOAD,
                });
            }
        }
        Message::CreateSession
        | Message::SessionCreated { .. }
        | Message::JoinSession { .. }
        | Message::JoinAccepted { .. }
        | Message::LeaveSession
        | Message::PeerJoined { .. }
        | Message::PeerLeft { .. }
        | Message::Heartbeat { .. }
        | Message::SessionExpired => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::CursorId;
    use crate::messages::Message;
    use crate::role::Role;

    #[test]
    fn rejects_bad_protocol_version() {
        let msg = Message::Hello {
            protocol_version: 99,
            role: Role::Host,
        };
        assert!(validate_message(&msg).is_err());
    }

    #[test]
    fn rejects_cursor_out_of_range() {
        let msg = Message::CursorMove {
            cursor_id: CursorId::new(),
            x: 1.5,
            y: 0.2,
            timestamp_ms: 0,
        };
        assert!(validate_message(&msg).is_err());
    }

    #[test]
    fn rejects_h265() {
        let msg = Message::VideoFrame {
            frame_id: 1,
            timestamp_ms: 0,
            width: 1920,
            height: 1080,
            codec: VideoCodec::H265,
            data: vec![0],
        };
        assert!(validate_message(&msg).is_err());
    }

    #[test]
    fn rejects_oversized_video() {
        let msg = Message::VideoFrame {
            frame_id: 1,
            timestamp_ms: 0,
            width: 1920,
            height: 1080,
            codec: VideoCodec::H264,
            data: vec![0; MAX_VIDEO_PAYLOAD + 1],
        };
        assert!(matches!(
            validate_message(&msg),
            Err(ProtocolError::FrameTooLarge { .. })
        ));
    }
}
