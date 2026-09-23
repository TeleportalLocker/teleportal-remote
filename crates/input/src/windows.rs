//! Injection Windows via SendInput.

use teleportal_protocol::{KeyModifiers, Message, MouseButton};
use tracing::debug;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, VIRTUAL_KEY, WHEEL_DELTA,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

use crate::error::InputError;
use crate::mapping::win_vk;
use crate::traits::InputInjector;
use crate::types::InjectConfig;

/// Injecteur Windows (`SendInput`).
pub struct WinInjector {
    config: InjectConfig,
}

impl InputInjector for WinInjector {
    fn start(mut config: InjectConfig) -> Result<Self, InputError> {
        // Préférer la taille système si la config par défaut est utilisée.
        let sx = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let sy = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        if sx > 0 && sy > 0 {
            config.display_width = sx as u32;
            config.display_height = sy as u32;
        }
        config.validate()?;
        debug!(
            width = config.display_width,
            height = config.display_height,
            "win injector started"
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
                self.mouse_button(*button, *pressed)
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
        debug!("win injector stopped");
        Ok(())
    }
}

impl WinInjector {
    fn send(&self, inputs: &mut [INPUT]) -> Result<(), InputError> {
        let sent = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
        if sent as usize != inputs.len() {
            return Err(InputError::Native(format!(
                "SendInput sent {sent}/{}",
                inputs.len()
            )));
        }
        Ok(())
    }

    fn abs_coords(&self, x: f32, y: f32) -> (i32, i32) {
        // SendInput absolute : 0..65535 sur l'écran virtuel primaire.
        let nx = x.clamp(0.0, 1.0);
        let ny = y.clamp(0.0, 1.0);
        let ax = (nx * 65535.0).round() as i32;
        let ay = (ny * 65535.0).round() as i32;
        (ax, ay)
    }

    fn mouse_move(&self, x: f32, y: f32) -> Result<(), InputError> {
        let (ax, ay) = self.abs_coords(x, y);
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: ax,
                    dy: ay,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        self.send(std::slice::from_mut(&mut input))
    }

    fn mouse_button(&self, button: MouseButton, pressed: bool) -> Result<(), InputError> {
        let flags = match (button, pressed) {
            (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
            (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
            (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
            (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
            (MouseButton::Middle, true) => MOUSEEVENTF_MIDDLEDOWN,
            (MouseButton::Middle, false) => MOUSEEVENTF_MIDDLEUP,
        };
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        self.send(std::slice::from_mut(&mut input))
    }

    fn mouse_scroll(&self, dx: f32, dy: f32) -> Result<(), InputError> {
        if dy.abs() > f32::EPSILON {
            let delta = (dy * f32::from(WHEEL_DELTA)).round() as i32;
            let mut input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: delta as u32,
                        dwFlags: MOUSEEVENTF_WHEEL,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            self.send(std::slice::from_mut(&mut input))?;
        }
        if dx.abs() > f32::EPSILON {
            let delta = (dx * f32::from(WHEEL_DELTA)).round() as i32;
            let mut input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: delta as u32,
                        dwFlags: MOUSEEVENTF_HWHEEL,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            self.send(std::slice::from_mut(&mut input))?;
        }
        Ok(())
    }

    fn key_event(
        &self,
        key: &str,
        pressed: bool,
        _modifiers: &KeyModifiers,
    ) -> Result<(), InputError> {
        let vk = win_vk(key)?;
        let flags = if pressed {
            Default::default()
        } else {
            KEYEVENTF_KEYUP
        };
        let mut input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk.0),
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        self.send(std::slice::from_mut(&mut input))
    }
}
