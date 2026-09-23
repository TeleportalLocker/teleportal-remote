//! Backend décodeur OpenH264.

use bytes::Bytes;
use openh264::decoder::Decoder;
use openh264::formats::YUVSource;
use openh264::nal_units;
use openh264::OpenH264API;
use tracing::debug;

use crate::error::DecodeError;
use crate::traits::VideoDecoder;
use crate::types::{DecodeConfig, DecodedFrame};

/// Décodeur H264 logiciel (Cisco OpenH264).
pub struct OpenH264Decoder {
    decoder: Decoder,
    rgba: Vec<u8>,
}

impl VideoDecoder for OpenH264Decoder {
    fn start(config: DecodeConfig) -> Result<Self, DecodeError> {
        config.validate()?;
        let api = OpenH264API::from_source();
        let decoder = Decoder::with_api_config(api, openh264::decoder::DecoderConfig::new())
            .map_err(|e| DecodeError::Native(e.to_string()))?;
        debug!("openh264 decoder started");
        Ok(Self {
            decoder,
            rgba: Vec::new(),
        })
    }

    fn decode(
        &mut self,
        frame_id: u64,
        timestamp_ms: u64,
        annex_b: &[u8],
    ) -> Result<Option<DecodedFrame>, DecodeError> {
        if annex_b.is_empty() {
            return Err(DecodeError::InvalidBitstream(
                "empty annex-b payload".into(),
            ));
        }

        let mut latest: Option<(u32, u32)> = None;
        for nal in nal_units(annex_b) {
            match self.decoder.decode(nal) {
                Ok(Some(yuv)) => {
                    let (w, h) = yuv.dimensions();
                    let needed = w.saturating_mul(h).saturating_mul(4);
                    self.rgba.resize(needed, 0);
                    yuv.write_rgba8(&mut self.rgba);
                    latest = Some((w as u32, h as u32));
                }
                Ok(None) => {}
                Err(e) => {
                    // Avant la première IDR, OpenH264 peut échouer sur des NAL isolées.
                    debug!(error = %e, "openh264 decode nal skipped");
                }
            }
        }

        let Some((width, height)) = latest else {
            return Ok(None);
        };

        Ok(Some(DecodedFrame {
            frame_id,
            timestamp_ms,
            width,
            height,
            data: Bytes::from(self.rgba.clone()),
        }))
    }

    fn stop(self) -> Result<(), DecodeError> {
        debug!("openh264 decoder stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use teleportal_capture::{DisplayId, Frame, PixelFormat};

    use crate::openh264_enc::OpenH264Encoder;
    use crate::traits::{VideoDecoder, VideoEncoder};
    use crate::types::{DecodeConfig, EncodeConfig};

    fn solid_bgra(width: u32, height: u32, b: u8, g: u8, r: u8) -> Frame {
        let stride = (width as usize) * 4;
        let mut data = vec![0u8; stride * height as usize];
        for px in data.as_chunks_mut::<4>().0 {
            px[0] = b;
            px[1] = g;
            px[2] = r;
            px[3] = 255;
        }
        Frame {
            width,
            height,
            stride,
            format: PixelFormat::Bgra8,
            timestamp_ms: 99,
            display_id: DisplayId(0),
            data: Bytes::from(data),
        }
    }

    #[test]
    fn encode_decode_round_trip() {
        let mut enc = OpenH264Encoder::start(EncodeConfig::default()).expect("enc");
        let mut dec = OpenH264Decoder::start(DecodeConfig::default()).expect("dec");
        let src = solid_bgra(128, 72, 10, 40, 200);
        let encoded = enc.encode(&src).expect("encode").expect("kf");
        assert!(encoded.is_keyframe);

        let decoded = dec
            .decode(encoded.frame_id, encoded.timestamp_ms, &encoded.data)
            .expect("decode")
            .expect("image");
        assert_eq!(decoded.width, 128);
        assert_eq!(decoded.height, 72);
        assert_eq!(decoded.data.len(), 128 * 72 * 4);
        assert_eq!(decoded.frame_id, encoded.frame_id);
        // Alpha opaque
        assert_eq!(decoded.data[3], 255);
        enc.stop().expect("enc stop");
        dec.stop().expect("dec stop");
    }
}
