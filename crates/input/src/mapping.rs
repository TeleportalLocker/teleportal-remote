//! Mapping `KeyboardEvent.code` → codes plateforme.

use crate::error::InputError;

/// Code virtuel Windows (`VIRTUAL_KEY` as u16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinVk(pub u16);

/// Keycode macOS (`CGKeyCode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacKeyCode(pub u16);

/// Résout un `KeyboardEvent.code` vers VK Windows.
///
/// # Errors
///
/// Code inconnu.
pub fn win_vk(code: &str) -> Result<WinVk, InputError> {
    Ok(WinVk(match code {
        "KeyA" => 0x41,
        "KeyB" => 0x42,
        "KeyC" => 0x43,
        "KeyD" => 0x44,
        "KeyE" => 0x45,
        "KeyF" => 0x46,
        "KeyG" => 0x47,
        "KeyH" => 0x48,
        "KeyI" => 0x49,
        "KeyJ" => 0x4A,
        "KeyK" => 0x4B,
        "KeyL" => 0x4C,
        "KeyM" => 0x4D,
        "KeyN" => 0x4E,
        "KeyO" => 0x4F,
        "KeyP" => 0x50,
        "KeyQ" => 0x51,
        "KeyR" => 0x52,
        "KeyS" => 0x53,
        "KeyT" => 0x54,
        "KeyU" => 0x55,
        "KeyV" => 0x56,
        "KeyW" => 0x57,
        "KeyX" => 0x58,
        "KeyY" => 0x59,
        "KeyZ" => 0x5A,
        "Digit0" | "Numpad0" => 0x30,
        "Digit1" | "Numpad1" => 0x31,
        "Digit2" | "Numpad2" => 0x32,
        "Digit3" | "Numpad3" => 0x33,
        "Digit4" | "Numpad4" => 0x34,
        "Digit5" | "Numpad5" => 0x35,
        "Digit6" | "Numpad6" => 0x36,
        "Digit7" | "Numpad7" => 0x37,
        "Digit8" | "Numpad8" => 0x38,
        "Digit9" | "Numpad9" => 0x39,
        "Space" => 0x20,
        "Enter" | "NumpadEnter" => 0x0D,
        "Escape" => 0x1B,
        "Tab" => 0x09,
        "Backspace" => 0x08,
        "Delete" => 0x2E,
        "ArrowLeft" => 0x25,
        "ArrowUp" => 0x26,
        "ArrowRight" => 0x27,
        "ArrowDown" => 0x28,
        "Home" => 0x24,
        "End" => 0x23,
        "PageUp" => 0x21,
        "PageDown" => 0x22,
        "ShiftLeft" | "ShiftRight" => 0x10,
        "ControlLeft" | "ControlRight" => 0x11,
        "AltLeft" | "AltRight" => 0x12,
        "MetaLeft" | "MetaRight" | "OSLeft" | "OSRight" => 0x5B,
        "F1" => 0x70,
        "F2" => 0x71,
        "F3" => 0x72,
        "F4" => 0x73,
        "F5" => 0x74,
        "F6" => 0x75,
        "F7" => 0x76,
        "F8" => 0x77,
        "F9" => 0x78,
        "F10" => 0x79,
        "F11" => 0x7A,
        "F12" => 0x7B,
        "Minus" => 0xBD,
        "Equal" => 0xBB,
        "BracketLeft" => 0xDB,
        "BracketRight" => 0xDD,
        "Backslash" => 0xDC,
        "Semicolon" => 0xBA,
        "Quote" => 0xDE,
        "Comma" => 0xBC,
        "Period" => 0xBE,
        "Slash" => 0xBF,
        "Backquote" => 0xC0,
        other => return Err(InputError::UnknownKey(other.into())),
    }))
}

/// Résout un `KeyboardEvent.code` vers keycode macOS (Carbon virtual keycodes).
///
/// # Errors
///
/// Code inconnu.
pub fn mac_keycode(code: &str) -> Result<MacKeyCode, InputError> {
    Ok(MacKeyCode(match code {
        "KeyA" => 0x00,
        "KeyS" => 0x01,
        "KeyD" => 0x02,
        "KeyF" => 0x03,
        "KeyH" => 0x04,
        "KeyG" => 0x05,
        "KeyZ" => 0x06,
        "KeyX" => 0x07,
        "KeyC" => 0x08,
        "KeyV" => 0x09,
        "KeyB" => 0x0B,
        "KeyQ" => 0x0C,
        "KeyW" => 0x0D,
        "KeyE" => 0x0E,
        "KeyR" => 0x0F,
        "KeyY" => 0x10,
        "KeyT" => 0x11,
        "Digit1" => 0x12,
        "Digit2" => 0x13,
        "Digit3" => 0x14,
        "Digit4" => 0x15,
        "Digit6" => 0x16,
        "Digit5" => 0x17,
        "Equal" => 0x18,
        "Digit9" => 0x19,
        "Digit7" => 0x1A,
        "Minus" => 0x1B,
        "Digit8" => 0x1C,
        "Digit0" => 0x1D,
        "BracketRight" => 0x1E,
        "KeyO" => 0x1F,
        "KeyU" => 0x20,
        "BracketLeft" => 0x21,
        "KeyI" => 0x22,
        "KeyP" => 0x23,
        "Enter" | "NumpadEnter" => 0x24,
        "KeyL" => 0x25,
        "KeyJ" => 0x26,
        "Quote" => 0x27,
        "KeyK" => 0x28,
        "Semicolon" => 0x29,
        "Backslash" => 0x2A,
        "Comma" => 0x2B,
        "Slash" => 0x2C,
        "KeyN" => 0x2D,
        "KeyM" => 0x2E,
        "Period" => 0x2F,
        "Tab" => 0x30,
        "Space" => 0x31,
        "Backquote" => 0x32,
        "Backspace" => 0x33,
        "Escape" => 0x35,
        "MetaRight" | "OSRight" => 0x36,
        "MetaLeft" | "OSLeft" => 0x37,
        "ShiftLeft" => 0x38,
        "AltLeft" => 0x3A,
        "ControlLeft" => 0x3B,
        "ShiftRight" => 0x3C,
        "AltRight" => 0x3D,
        "ControlRight" => 0x3E,
        "F17" => 0x40,
        "F18" => 0x4F,
        "F19" => 0x50,
        "F5" => 0x60,
        "F6" => 0x61,
        "F7" => 0x62,
        "F3" => 0x63,
        "F8" => 0x64,
        "F9" => 0x65,
        "F11" => 0x67,
        "F13" => 0x69,
        "F16" => 0x6A,
        "F14" => 0x6B,
        "F10" => 0x6D,
        "F12" => 0x6F,
        "F15" => 0x71,
        "Home" => 0x73,
        "PageUp" => 0x74,
        "Delete" => 0x75,
        "F4" => 0x76,
        "End" => 0x77,
        "F2" => 0x78,
        "PageDown" => 0x79,
        "F1" => 0x7A,
        "ArrowLeft" => 0x7B,
        "ArrowRight" => 0x7C,
        "ArrowDown" => 0x7D,
        "ArrowUp" => 0x7E,
        other => return Err(InputError::UnknownKey(other.into())),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_codes() {
        assert_eq!(win_vk("KeyA").unwrap().0, 0x41);
        assert_eq!(win_vk("Space").unwrap().0, 0x20);
        assert_eq!(mac_keycode("KeyA").unwrap().0, 0x00);
        assert_eq!(mac_keycode("Space").unwrap().0, 0x31);
        assert!(win_vk("Nope").is_err());
    }
}
