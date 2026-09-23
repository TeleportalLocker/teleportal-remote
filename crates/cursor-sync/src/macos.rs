//! Poll curseur macOS (`CGEvent` location).

use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use crate::error::CursorError;
use crate::types::normalize_in_display;

pub(crate) fn poll(
    origin_x: i32,
    origin_y: i32,
    display_w: u32,
    display_h: u32,
) -> Result<Option<(f32, f32)>, CursorError> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|()| CursorError::Native("CGEventSource::new failed".into()))?;
    let event =
        CGEvent::new(source).map_err(|()| CursorError::Native("CGEvent::new failed".into()))?;
    let loc = event.location();
    Ok(normalize_in_display(
        loc.x.round() as i32,
        loc.y.round() as i32,
        origin_x,
        origin_y,
        display_w,
        display_h,
    ))
}
