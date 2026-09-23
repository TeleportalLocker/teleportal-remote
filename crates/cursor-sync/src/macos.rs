//! Poll curseur macOS (`CGEvent` location).

use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use crate::error::CursorError;
use crate::types::normalize_pixels;

pub(crate) fn poll(display_w: u32, display_h: u32) -> Result<(f32, f32), CursorError> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|()| CursorError::Native("CGEventSource::new failed".into()))?;
    let event =
        CGEvent::new(source).map_err(|()| CursorError::Native("CGEvent::new failed".into()))?;
    let loc = event.location();
    Ok(normalize_pixels(
        loc.x.round() as i32,
        loc.y.round() as i32,
        display_w,
        display_h,
    ))
}
