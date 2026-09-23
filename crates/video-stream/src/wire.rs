//! Conversion `EncodedFrame` → message protocole.

use teleportal_protocol::{Message, VideoCodec};

use crate::types::EncodedFrame;

/// Construit un [`Message::VideoFrame`] H264 depuis une frame encodée.
#[must_use]
pub fn to_video_message(encoded: EncodedFrame) -> Message {
    Message::VideoFrame {
        frame_id: encoded.frame_id,
        timestamp_ms: encoded.timestamp_ms,
        width: encoded.width,
        height: encoded.height,
        codec: VideoCodec::H264,
        data: encoded.data.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use teleportal_protocol::{validate_message, Message, VideoCodec};

    use super::*;
    use crate::types::EncodedFrame;

    #[test]
    fn builds_valid_h264_message() {
        let msg = to_video_message(EncodedFrame {
            frame_id: 7,
            timestamp_ms: 1000,
            width: 128,
            height: 72,
            is_keyframe: true,
            data: Bytes::from(vec![0, 0, 0, 1, 0x65]),
        });
        assert!(validate_message(&msg).is_ok());
        match msg {
            Message::VideoFrame {
                frame_id,
                codec,
                data,
                ..
            } => {
                assert_eq!(frame_id, 7);
                assert_eq!(codec, VideoCodec::H264);
                assert_eq!(data, vec![0, 0, 0, 1, 0x65]);
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }
}
