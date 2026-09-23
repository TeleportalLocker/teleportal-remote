//! Backend logiciel OpenH264.

use bytes::Bytes;
use openh264::encoder::{
    BitRate, Encoder, EncoderConfig, FrameRate, FrameType, IntraFramePeriod, RateControlMode,
    UsageType,
};
use openh264::formats::{BgraSliceU8, YUVBuffer};
use openh264::OpenH264API;
use teleportal_capture::{Frame, PixelFormat};
use tracing::debug;

use crate::error::EncodeError;
use crate::traits::VideoEncoder;
use crate::types::{EncodeConfig, EncodedFrame};

/// Encodeur H264 logiciel (Cisco OpenH264).
pub struct OpenH264Encoder {
    config: EncodeConfig,
    encoder: Encoder,
    next_frame_id: u64,
    /// Buffer BGRA dense réutilisé (sans padding de stride).
    packed_bgra: Vec<u8>,
    /// Buffer YUV réutilisé.
    yuv: Option<YUVBuffer>,
    yuv_dims: (usize, usize),
}

impl VideoEncoder for OpenH264Encoder {
    fn start(config: EncodeConfig) -> Result<Self, EncodeError> {
        config.validate()?;

        let oh_config = EncoderConfig::new()
            .bitrate(BitRate::from_bps(config.bitrate_bps))
            .max_frame_rate(FrameRate::from_hz(config.max_fps as f32))
            .usage_type(UsageType::ScreenContentRealTime)
            .rate_control_mode(RateControlMode::Bitrate)
            .adaptive_quantization(false)
            .background_detection(false)
            .skip_frames(false)
            .intra_frame_period(IntraFramePeriod::from_num_frames(config.keyframe_interval));

        let api = OpenH264API::from_source();
        let encoder = Encoder::with_api_config(api, oh_config)
            .map_err(|e| EncodeError::Native(e.to_string()))?;

        debug!(
            max_fps = config.max_fps,
            bitrate_bps = config.bitrate_bps,
            keyframe_interval = config.keyframe_interval,
            "openh264 encoder started"
        );

        Ok(Self {
            config,
            encoder,
            next_frame_id: 0,
            packed_bgra: Vec::new(),
            yuv: None,
            yuv_dims: (0, 0),
        })
    }

    fn encode(&mut self, frame: &Frame) -> Result<Option<EncodedFrame>, EncodeError> {
        if frame.format != PixelFormat::Bgra8 {
            return Err(EncodeError::InvalidFrame(format!(
                "unsupported pixel format {:?}",
                frame.format
            )));
        }
        if !frame.is_well_formed() {
            return Err(EncodeError::InvalidFrame(
                "frame buffer is not well-formed".into(),
            ));
        }

        let (width, height) = even_dims(frame.width, frame.height)?;
        pack_bgra(frame, width, height, &mut self.packed_bgra)?;

        let bgra = BgraSliceU8::new(&self.packed_bgra, (width, height));
        if self.yuv_dims != (width, height) {
            self.yuv = Some(YUVBuffer::new(width, height));
            self.yuv_dims = (width, height);
        }
        let yuv = self
            .yuv
            .as_mut()
            .expect("yuv buffer initialized after dimension check");
        yuv.read_rgb(bgra);

        let bitstream = self
            .encoder
            .encode(yuv)
            .map_err(|e| EncodeError::Native(e.to_string()))?;

        match bitstream.frame_type() {
            FrameType::Skip | FrameType::Invalid => return Ok(None),
            FrameType::IDR | FrameType::I | FrameType::P | FrameType::IPMixed => {}
        }

        let is_keyframe = matches!(bitstream.frame_type(), FrameType::IDR | FrameType::I);
        let data = bitstream.to_vec();
        if data.is_empty() {
            return Ok(None);
        }

        let frame_id = self.next_frame_id;
        self.next_frame_id = self.next_frame_id.wrapping_add(1);

        Ok(Some(EncodedFrame {
            frame_id,
            timestamp_ms: frame.timestamp_ms,
            width: width as u32,
            height: height as u32,
            is_keyframe,
            data: Bytes::from(data),
        }))
    }

    fn force_keyframe(&mut self) {
        self.encoder.force_intra_frame();
    }

    fn stop(self) -> Result<(), EncodeError> {
        debug!(
            encoded_frames = self.next_frame_id,
            max_fps = self.config.max_fps,
            "openh264 encoder stopped"
        );
        Ok(())
    }
}

fn even_dims(width: u32, height: u32) -> Result<(usize, usize), EncodeError> {
    let w = (width as usize) & !1;
    let h = (height as usize) & !1;
    if w == 0 || h == 0 {
        return Err(EncodeError::InvalidFrame(
            "encoded dimensions must be >= 2 after even alignment".into(),
        ));
    }
    Ok((w, h))
}

fn pack_bgra(
    frame: &Frame,
    width: usize,
    height: usize,
    out: &mut Vec<u8>,
) -> Result<(), EncodeError> {
    let dst_stride = width.saturating_mul(4);
    out.resize(dst_stride.saturating_mul(height), 0);
    let src = frame.data.as_ref();
    for y in 0..height {
        let src_off = y.saturating_mul(frame.stride);
        let dst_off = y.saturating_mul(dst_stride);
        let end = src_off.saturating_add(dst_stride);
        if end > src.len() {
            return Err(EncodeError::InvalidFrame(
                "source row exceeds frame buffer".into(),
            ));
        }
        out[dst_off..dst_off + dst_stride].copy_from_slice(&src[src_off..end]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use teleportal_capture::{DisplayId, Frame, PixelFormat};
    use teleportal_protocol::validate_message;

    use crate::traits::VideoEncoder;
    use crate::wire::to_video_message;

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
            timestamp_ms: 42,
            display_id: DisplayId(0),
            data: Bytes::from(data),
        }
    }

    #[test]
    fn encode_keyframe_and_protocol_message() {
        let mut enc = OpenH264Encoder::start(EncodeConfig::default()).expect("start");
        let frame = solid_bgra(128, 72, 10, 20, 30);
        let encoded = enc.encode(&frame).expect("encode").expect("not skipped");
        assert!(encoded.is_keyframe);
        assert!(!encoded.data.is_empty());
        assert_eq!(&encoded.data[..4], &[0, 0, 0, 1]);
        assert!(validate_message(&to_video_message(encoded)).is_ok());
        enc.stop().expect("stop");
    }

    #[test]
    fn encode_second_frame() {
        let mut enc = OpenH264Encoder::start(EncodeConfig {
            keyframe_interval: 0,
            ..EncodeConfig::default()
        })
        .expect("start");
        let f1 = solid_bgra(64, 64, 0, 0, 0);
        let e1 = enc.encode(&f1).expect("e1").expect("kf");
        assert!(e1.is_keyframe);
        let f2 = solid_bgra(64, 64, 255, 0, 0);
        let e2 = enc.encode(&f2).expect("e2").expect("p");
        assert_eq!(e2.frame_id, 1);
        enc.stop().expect("stop");
    }

    #[test]
    fn rejects_odd_tiny_frame() {
        let mut enc = OpenH264Encoder::start(EncodeConfig::default()).expect("start");
        let frame = solid_bgra(1, 1, 0, 0, 0);
        let err = enc.encode(&frame).unwrap_err();
        assert!(matches!(err, EncodeError::InvalidFrame(_)));
    }
}
