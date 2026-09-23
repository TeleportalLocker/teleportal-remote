//! Injection macOS via CGEvent.

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGMouseButton, ScrollEventUnit,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use teleportal_protocol::{KeyModifiers, Message, MouseButton};
use tracing::debug;

use crate::error::InputError;
use crate::mapping::mac_keycode;
use crate::traits::InputInjector;
use crate::types::InjectConfig;

/// Injecteur macOS (`CGEvent`).
pub struct MacInjector {
    config: InjectConfig,
}

impl InputInjector for MacInjector {
    fn start(mut config: InjectConfig) -> Result<Self, InputError> {
        ensure_accessibility()?;
        if let Some((w, h)) = primary_display_size() {
            config.display_width = w;
            config.display_height = h;
        }
        config.validate()?;
        debug!(
            width = config.display_width,
            height = config.display_height,
            "mac injector started"
        );
        Ok(Self { config })
    }

    fn inject(&mut self, msg: &Message) -> Result<(), InputError> {
        match msg {
            Message::MouseMove { x, y, .. } => self.mouse_move(*x, *y),
            Message::MouseButton {
                button,
                pressed,
                x,
                y,
                ..
            } => {
                self.mouse_move(*x, *y)?;
                self.mouse_button(*button, *pressed, *x, *y)
            }
            Message::MouseScroll { dx, dy, x, y, .. } => {
                self.mouse_move(*x, *y)?;
                self.mouse_scroll(*dx, *dy)
            }
            Message::KeyEvent {
                key,
                pressed,
                modifiers,
                ..
            } => self.key_event(key, *pressed, modifiers),
            _ => Ok(()),
        }
    }

    fn stop(self) -> Result<(), InputError> {
        debug!("mac injector stopped");
        Ok(())
    }
}

impl MacInjector {
    fn source(&self) -> Result<CGEventSource, InputError> {
        CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|()| InputError::Native("CGEventSource::new failed".into()))
    }

    fn point(&self, x: f32, y: f32) -> CGPoint {
        let (px, py) = self.config.to_pixels(x, y);
        CGPoint::new(f64::from(px), f64::from(py))
    }

    fn mouse_move(&self, x: f32, y: f32) -> Result<(), InputError> {
        let event = CGEvent::new_mouse_event(
            self.source()?,
            CGEventType::MouseMoved,
            self.point(x, y),
            CGMouseButton::Left,
        )
        .map_err(|()| InputError::Native("CGEvent mouse move failed".into()))?;
        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    fn mouse_button(
        &self,
        button: MouseButton,
        pressed: bool,
        x: f32,
        y: f32,
    ) -> Result<(), InputError> {
        let (etype, mbtn) = match (button, pressed) {
            (MouseButton::Left, true) => (CGEventType::LeftMouseDown, CGMouseButton::Left),
            (MouseButton::Left, false) => (CGEventType::LeftMouseUp, CGMouseButton::Left),
            (MouseButton::Right, true) => (CGEventType::RightMouseDown, CGMouseButton::Right),
            (MouseButton::Right, false) => (CGEventType::RightMouseUp, CGMouseButton::Right),
            (MouseButton::Middle, true) => (CGEventType::OtherMouseDown, CGMouseButton::Center),
            (MouseButton::Middle, false) => (CGEventType::OtherMouseUp, CGMouseButton::Center),
        };
        let event = CGEvent::new_mouse_event(self.source()?, etype, self.point(x, y), mbtn)
            .map_err(|()| InputError::Native("CGEvent mouse button failed".into()))?;
        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    fn mouse_scroll(&self, dx: f32, dy: f32) -> Result<(), InputError> {
        let wheel1 = (-dy * 3.0).round() as i32; // axis 1 = vertical
        let wheel2 = (dx * 3.0).round() as i32;
        if wheel1 == 0 && wheel2 == 0 {
            return Ok(());
        }
        let event =
            CGEvent::new_scroll_event(self.source()?, ScrollEventUnit::LINE, 2, wheel1, wheel2, 0)
                .map_err(|()| InputError::Native("CGEvent scroll failed".into()))?;
        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    fn key_event(
        &self,
        key: &str,
        pressed: bool,
        modifiers: &KeyModifiers,
    ) -> Result<(), InputError> {
        let code = mac_keycode(key)?;
        let event = CGEvent::new_keyboard_event(self.source()?, code.0, pressed)
            .map_err(|()| InputError::Native("CGEvent keyboard failed".into()))?;
        let mut flags = CGEventFlags::CGEventFlagNull;
        if modifiers.shift {
            flags.insert(CGEventFlags::CGEventFlagShift);
        }
        if modifiers.ctrl {
            flags.insert(CGEventFlags::CGEventFlagControl);
        }
        if modifiers.alt {
            flags.insert(CGEventFlags::CGEventFlagAlternate);
        }
        if modifiers.meta {
            flags.insert(CGEventFlags::CGEventFlagCommand);
        }
        event.set_flags(flags);
        event.post(CGEventTapLocation::HID);
        Ok(())
    }
}

fn ensure_accessibility() -> Result<(), InputError> {
    if is_process_trusted(false) {
        return Ok(());
    }
    debug!("accessibility permission missing — prompting");
    let _ = is_process_trusted(true);
    if is_process_trusted(false) {
        return Ok(());
    }
    Err(InputError::PermissionDenied)
}

fn is_process_trusted(prompt: bool) -> bool {
    unsafe {
        if prompt {
            let key = CFString::new("AXTrustedCheckOptionPrompt");
            let value = CFBoolean::true_value();
            let opts = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
            AXIsProcessTrustedWithOptions(opts.as_concrete_TypeRef())
        } else {
            AXIsProcessTrusted()
        }
    }
}

fn primary_display_size() -> Option<(u32, u32)> {
    use core_graphics::display::CGDisplay;
    let main = CGDisplay::main();
    let bounds = main.bounds();
    let w = bounds.size.width.round() as u32;
    let h = bounds.size.height.round() as u32;
    if w > 0 && h > 0 {
        Some((w, h))
    } else {
        None
    }
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef)
        -> bool;
}
