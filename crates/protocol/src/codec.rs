//! Codec length-prefixed MessagePack.

use crate::error::ProtocolError;
use crate::messages::Message;
use crate::validate::{validate_message, MAX_FRAME_BYTES};

/// Encode un message : validation puis `[u32 BE length][msgpack payload]`.
///
/// # Errors
///
/// Erreurs de validation ou de sérialisation MessagePack.
pub fn encode_message(message: &Message) -> Result<Vec<u8>, ProtocolError> {
    validate_message(message)?;
    let payload =
        rmp_serde::to_vec_named(message).map_err(|e| ProtocolError::Codec(e.to_string()))?;
    let len = payload.len();
    if len > MAX_FRAME_BYTES {
        return Err(ProtocolError::FrameTooLarge {
            size: len,
            max: MAX_FRAME_BYTES,
        });
    }
    let mut out = Vec::with_capacity(4 + len);
    out.extend_from_slice(&(len as u32).to_be_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}

/// Décode une frame complète length-prefixed et valide le message.
///
/// # Errors
///
/// Frame tronquée, trop grande, codec invalide, ou validation échouée.
pub fn decode_message(frame: &[u8]) -> Result<Message, ProtocolError> {
    if frame.len() < 4 {
        return Err(ProtocolError::TruncatedFrame);
    }
    let len = u32::from_be_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;
    if len > MAX_FRAME_BYTES {
        return Err(ProtocolError::FrameTooLarge {
            size: len,
            max: MAX_FRAME_BYTES,
        });
    }
    if frame.len() < 4 + len {
        return Err(ProtocolError::TruncatedFrame);
    }
    if frame.len() != 4 + len {
        return Err(ProtocolError::Validation(
            "frame contains trailing bytes".into(),
        ));
    }
    let message: Message =
        rmp_serde::from_slice(&frame[4..]).map_err(|e| ProtocolError::Codec(e.to_string()))?;
    validate_message(&message)?;
    Ok(message)
}

/// Lit la longueur déclarée sans consommer le payload (pour lecteurs streaming).
///
/// # Errors
///
/// Moins de 4 octets disponibles, ou longueur au-delà de [`MAX_FRAME_BYTES`].
pub fn peek_payload_len(header: &[u8]) -> Result<usize, ProtocolError> {
    if header.len() < 4 {
        return Err(ProtocolError::TruncatedFrame);
    }
    let len = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
    if len > MAX_FRAME_BYTES {
        return Err(ProtocolError::FrameTooLarge {
            size: len,
            max: MAX_FRAME_BYTES,
        });
    }
    Ok(len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{CursorId, PeerId, SessionCode, SessionId};
    use crate::messages::{KeyModifiers, Message, MouseButton, VideoCodec, PROTOCOL_VERSION};
    use crate::role::Role;

    fn round_trip(message: Message) {
        let encoded = encode_message(&message).expect("encode");
        let decoded = decode_message(&encoded).expect("decode");
        assert_eq!(decoded, message);
    }

    #[test]
    fn round_trip_hello() {
        round_trip(Message::Hello {
            protocol_version: PROTOCOL_VERSION,
            role: Role::Host,
        });
    }

    #[test]
    fn round_trip_session_created() {
        round_trip(Message::SessionCreated {
            session_id: SessionId::new(),
            code: SessionCode::parse("654321").unwrap(),
        });
    }

    #[test]
    fn round_trip_join_flow() {
        round_trip(Message::JoinSession {
            code: SessionCode::parse("111222").unwrap(),
        });
        round_trip(Message::JoinAccepted {
            session_id: SessionId::new(),
            peer_id: PeerId::new(),
            role: Role::Guest,
        });
        round_trip(Message::JoinRejected {
            reason: "expired".into(),
        });
    }

    #[test]
    fn round_trip_cursor_and_input() {
        round_trip(Message::CursorMove {
            cursor_id: CursorId::new(),
            x: 0.5,
            y: 0.25,
            timestamp_ms: 42,
        });
        round_trip(Message::MouseMove {
            x: 0.1,
            y: 0.9,
            timestamp_ms: 1,
        });
        round_trip(Message::MouseButton {
            button: MouseButton::Left,
            pressed: true,
            x: 0.0,
            y: 1.0,
            timestamp_ms: 2,
        });
        round_trip(Message::MouseScroll {
            dx: 0.0,
            dy: -1.0,
            x: 0.5,
            y: 0.5,
            timestamp_ms: 3,
        });
        round_trip(Message::KeyEvent {
            key: "KeyA".into(),
            pressed: true,
            modifiers: KeyModifiers {
                shift: true,
                ..KeyModifiers::default()
            },
            timestamp_ms: 4,
        });
    }

    #[test]
    fn round_trip_video_frame() {
        round_trip(Message::VideoFrame {
            frame_id: 7,
            timestamp_ms: 100,
            width: 1920,
            height: 1080,
            codec: VideoCodec::H264,
            data: vec![0x00, 0x01, 0x02],
        });
    }

    #[test]
    fn decode_rejects_truncated() {
        assert_eq!(decode_message(&[0, 0]), Err(ProtocolError::TruncatedFrame));
    }

    #[test]
    fn encode_rejects_invalid_hello() {
        let msg = Message::Hello {
            protocol_version: 0,
            role: Role::Guest,
        };
        assert!(encode_message(&msg).is_err());
    }
}
