//! The pure decisions of the Wayland seat, keyboard and pointer (T169,
//! ADR-T169-1): what a `wl_seat.capabilities` event does, how a key is
//! repeated, how keysyms, buttons, fixed-point numbers and axis events
//! become [`RawEvent`](crate::platform::RawEvent) parts. No compositor and
//! no `unsafe` here, so every function is tested and fuzzed directly.

use std::time::Duration;

use crate::platform::{Device, DeviceChange, RawButton, RawKey, device_change};

/// `wl_keyboard.keymap_format.xkb_v1`.
const XKB_V1: u32 = 1;

/// Wheel units one notch (120 in `axis_value120`) stands for, like the
/// smooth value of a common wheel.
const UNITS_PER_NOTCH: f32 = 10.0;

/// `wl_seat.capability.pointer`.
pub const CAP_POINTER: u32 = 1;
/// `wl_seat.capability.keyboard`.
pub const CAP_KEYBOARD: u32 = 2;

/// The most repeats one poll sends. A loop that stalled for seconds with a
/// key held drops the surplus instead of replaying it (T45).
pub const MAX_REPEATS_PER_POLL: u32 = 8;

/// The seat devices the backend holds: at most one keyboard and one
/// pointer (ADR-T44-1). Touch is never taken.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SeatDevices {
    keyboard: bool,
    pointer: bool,
}

impl SeatDevices {
    /// Applies a `wl_seat.capabilities` event and returns what to do with
    /// each device that needs something done. A capability announced again
    /// keeps its device.
    pub fn on_capabilities(&mut self, caps: u32) -> Vec<(Device, DeviceChange)> {
        let mut changes = Vec::new();
        let announced = [
            (Device::Keyboard, caps & CAP_KEYBOARD != 0),
            (Device::Pointer, caps & CAP_POINTER != 0),
        ];
        for (device, announced) in announced {
            let slot = match device {
                Device::Keyboard => &mut self.keyboard,
                _ => &mut self.pointer,
            };
            match device_change(*slot, announced) {
                DeviceChange::Create => {
                    *slot = true;
                    changes.push((device, DeviceChange::Create));
                }
                DeviceChange::Release => {
                    *slot = false;
                    changes.push((device, DeviceChange::Release));
                }
                DeviceChange::Keep => {}
            }
        }
        changes
    }

    /// Whether a keyboard is held.
    #[must_use]
    pub fn has_keyboard(&self) -> bool {
        self.keyboard
    }

    /// Whether a pointer is held.
    #[must_use]
    pub fn has_pointer(&self) -> bool {
        self.pointer
    }
}

/// Whether a `wl_keyboard.keymap` event carries a keymap worth parsing
/// (ADR-T79-1): format `XKB_V1` and a size above zero.
#[must_use]
pub fn keymap_usable(format: u32, size: u32) -> bool {
    format == XKB_V1 && size > 0
}

/// How many repeats of a key held for `held_for` are due, given that
/// `emitted` were already sent. `rate` is repeats per second (0 or less
/// turns repeat off), `delay_ms` the wait before the first.
#[must_use]
pub fn repeats_due(rate: i32, delay_ms: i32, held_for: Duration, emitted: u32) -> u32 {
    let Ok(rate) = u128::try_from(rate) else {
        return 0;
    };
    if rate == 0 {
        return 0;
    }
    let delay = Duration::from_millis(u64::try_from(delay_ms.max(0)).unwrap_or(0));
    let Some(after_delay) = held_for.checked_sub(delay) else {
        return 0;
    };
    // `rate` repeats per second: one every 1e9 / rate nanoseconds.
    let total = 1 + after_delay.as_nanos() * rate / 1_000_000_000;
    u32::try_from(total)
        .unwrap_or(u32::MAX)
        .saturating_sub(emitted)
}

/// [`repeats_due`] capped at [`MAX_REPEATS_PER_POLL`].
#[must_use]
pub fn repeats_to_send(rate: i32, delay_ms: i32, held_for: Duration, emitted: u32) -> u32 {
    repeats_due(rate, delay_ms, held_for, emitted).min(MAX_REPEATS_PER_POLL)
}

/// The key a keysym stands for.
#[must_use]
pub fn raw_key_from_keysym(keysym: u32) -> RawKey {
    match keysym {
        0x41..=0x5a | 0x61..=0x7a => {
            let upper = char::from_u32(keysym).map_or('?', |c| c.to_ascii_uppercase());
            RawKey::Letter(upper)
        }
        0x30..=0x39 => RawKey::Digit((keysym - 0x30) as u8),
        0x20 => RawKey::Space,
        0x5b => RawKey::BracketLeft,
        0x5d => RawKey::BracketRight,
        0xff1b => RawKey::Escape,
        0xff09 => RawKey::Tab,
        0xff08 => RawKey::Backspace,
        0xffff => RawKey::Delete,
        0xff0d | 0xff8d => RawKey::Enter,
        0xff50 => RawKey::Home,
        0xff57 => RawKey::End,
        0xff51 => RawKey::ArrowLeft,
        0xff52 => RawKey::ArrowUp,
        0xff53 => RawKey::ArrowRight,
        0xff54 => RawKey::ArrowDown,
        0xffe1 => RawKey::LeftShift,
        0xffe2 => RawKey::RightShift,
        0xffe3 => RawKey::LeftControl,
        0xffe4 => RawKey::RightControl,
        0xffe9 => RawKey::LeftAlt,
        0xffea => RawKey::RightAlt,
        other => RawKey::Other(other),
    }
}

/// The button an evdev button code stands for.
#[must_use]
pub fn raw_button_from_evdev(code: u32) -> RawButton {
    match code {
        0x110 => RawButton::Left,
        0x111 => RawButton::Right,
        0x112 => RawButton::Middle,
        _ => RawButton::Other,
    }
}

/// A `wl_fixed_t` (24.8) as a float.
#[must_use]
pub fn fixed_to_f32(fixed: i32) -> f32 {
    fixed as f32 / 256.0
}

/// A surface position in `wl_fixed_t` as physical pixels at `scale`.
#[must_use]
pub fn pointer_position(x: i32, y: i32, scale: i32) -> (f32, f32) {
    let scale = scale.max(1) as f32;
    (fixed_to_f32(x) * scale, fixed_to_f32(y) * scale)
}

/// The axis events of one `wl_pointer.frame`.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AxisFrame {
    /// Smooth horizontal amount.
    pub dx: f32,
    /// Smooth vertical amount (Wayland: down is positive).
    pub dy: f32,
    /// `axis_value120` horizontal, if sent.
    pub v120_x: Option<i32>,
    /// `axis_value120` vertical, if sent.
    pub v120_y: Option<i32>,
}

/// The wheel movement of a frame, or `None` if it holds none. The vertical
/// sign is flipped; `axis_value120` wins over the smooth value.
#[must_use]
pub fn wheel_of_frame(frame: &AxisFrame) -> Option<(f32, f32)> {
    let finite = |v: f32| if v.is_finite() { v } else { 0.0 };
    let notch = |v120: i32| v120 as f32 / 120.0 * UNITS_PER_NOTCH;
    let dx = frame.v120_x.map_or_else(|| finite(frame.dx), notch);
    let dy = frame.v120_y.map_or_else(|| finite(frame.dy), notch);
    if dx == 0.0 && dy == 0.0 {
        return None;
    }
    Some((dx, -dy))
}
