//! Capturer ScreenCaptureKit.

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use core_media_rs::cm_sample_buffer::CMSampleBuffer;
use core_media_rs::cm_time::CMTime;
use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::pixel_format::PixelFormat as SckPixelFormat;
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;
use screencapturekit::stream::output_trait::SCStreamOutputTrait;
use screencapturekit::stream::output_type::SCStreamOutputType;
use screencapturekit::stream::SCStream;
use tracing::{debug, warn};

use crate::error::CaptureError;
use crate::sck::content::{ensure_permission, list_displays, map_sck};
use crate::sck::sample::frame_from_sample;
use crate::traits::Capturer;
use crate::types::{CaptureConfig, DisplayId, DisplayInfo, Frame};

struct LatestFrame {
    frame: Option<Frame>,
    generation: u64,
}

struct FrameHandler {
    display_id: DisplayId,
    latest: Arc<(Mutex<LatestFrame>, Condvar)>,
}

impl SCStreamOutputTrait for FrameHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if !matches!(output_type, SCStreamOutputType::Screen) {
            return;
        }
        match frame_from_sample(&sample, self.display_id) {
            Ok(Some(frame)) => {
                let (lock, cvar) = &*self.latest;
                if let Ok(mut guard) = lock.lock() {
                    guard.generation = guard.generation.wrapping_add(1);
                    guard.frame = Some(frame);
                    cvar.notify_one();
                }
            }
            Ok(None) => {}
            Err(e) => warn!(error = %e, "failed to convert sample buffer"),
        }
    }
}

/// Capturer macOS basé sur ScreenCaptureKit.
pub struct SckCapturer {
    config: CaptureConfig,
    display_id: DisplayId,
    stream: SCStream,
    latest: Arc<(Mutex<LatestFrame>, Condvar)>,
    last_seen_generation: u64,
}

impl Capturer for SckCapturer {
    fn displays() -> Result<Vec<DisplayInfo>, CaptureError> {
        list_displays()
    }

    fn start(config: CaptureConfig) -> Result<Self, CaptureError> {
        config.validate()?;
        ensure_permission()?;

        let content = SCShareableContent::get().map_err(map_sck)?;
        let displays = content.displays();
        let display = displays
            .get(config.display_index)
            .ok_or(CaptureError::DisplayNotFound(config.display_index))?;

        let display_id = DisplayId(display.display_id());
        let width = display.width();
        let height = display.height();

        let filter = SCContentFilter::new().with_display_excluding_windows(display, &[]);

        let fps = i32::try_from(config.max_fps.max(1)).unwrap_or(30);
        let interval = CMTime {
            value: 1,
            timescale: fps,
            flags: 1, // kCMTimeFlags_Valid
            epoch: 0,
        };

        let config_sck = SCStreamConfiguration::new()
            .set_width(width)
            .map_err(map_sck)?
            .set_height(height)
            .map_err(map_sck)?
            .set_pixel_format(SckPixelFormat::BGRA)
            .map_err(map_sck)?
            .set_shows_cursor(false)
            .map_err(map_sck)?
            .set_minimum_frame_interval(&interval)
            .map_err(map_sck)?;

        let latest = Arc::new((
            Mutex::new(LatestFrame {
                frame: None,
                generation: 0,
            }),
            Condvar::new(),
        ));

        let handler = FrameHandler {
            display_id,
            latest: Arc::clone(&latest),
        };

        let mut stream = SCStream::new(&filter, &config_sck);
        stream.add_output_handler(handler, SCStreamOutputType::Screen);
        stream.start_capture().map_err(map_sck)?;

        debug!(
            display_id = display_id.0,
            width,
            height,
            max_fps = config.max_fps,
            "sck capturer started"
        );

        Ok(Self {
            config,
            display_id,
            stream,
            latest,
            last_seen_generation: 0,
        })
    }

    fn grab(&mut self) -> Result<Option<Frame>, CaptureError> {
        let timeout = Duration::from_millis(u64::from(self.config.frame_timeout_ms()));
        let (lock, cvar) = &*self.latest;
        let mut guard = lock
            .lock()
            .map_err(|_| CaptureError::Native("frame mutex poisoned".into()))?;

        if guard.generation != self.last_seen_generation {
            self.last_seen_generation = guard.generation;
            return Ok(guard.frame.take());
        }

        let (updated, wait_result) = cvar
            .wait_timeout_while(guard, timeout, |g| {
                g.generation == self.last_seen_generation
            })
            .map_err(|_| CaptureError::Native("frame condvar poisoned".into()))?;
        guard = updated;

        if wait_result.timed_out() && guard.generation == self.last_seen_generation {
            return Ok(None);
        }

        self.last_seen_generation = guard.generation;
        Ok(guard.frame.take())
    }

    fn stop(self) -> Result<(), CaptureError> {
        debug!(display_id = self.display_id.0, "stopping sck capturer");
        self.stream.stop_capture().map_err(map_sck)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_or_permission() {
        match SckCapturer::displays() {
            Ok(list) => assert!(!list.is_empty()),
            Err(CaptureError::PermissionDenied) => {}
            Err(e) => panic!("unexpected error: {e}"),
        }
    }

    #[test]
    fn start_grab_or_permission() {
        match SckCapturer::start(CaptureConfig::default()) {
            Ok(mut capturer) => {
                let _ = capturer.grab().expect("grab");
                capturer.stop().expect("stop");
            }
            Err(CaptureError::PermissionDenied) => {}
            Err(CaptureError::NoDisplays) => {}
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
}
