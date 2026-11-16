//! The pure decisions of the Wayland seat, keyboard and pointer (T169,
//! ADR-T169-1): what a `wl_seat.capabilities` event does, how a key is
//! repeated, how keysyms, buttons, fixed-point numbers and axis events
//! become [`RawEvent`](crate::platform::RawEvent) parts. No compositor and
//! no `unsafe` here, so every function is tested and fuzzed directly.

use std::time::Duration;

use crate::platform::{Device, DeviceChange, RawButton, RawKey};

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
        let _ = (caps, &mut *self);
        todo!()
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
    let _ = (format, size);
    todo!()
}

/// How many repeats of a key held for `held_for` are due, given that
/// `emitted` were already sent. `rate` is repeats per second (0 or less
/// turns repeat off), `delay_ms` the wait before the first.
#[must_use]
pub fn repeats_due(rate: i32, delay_ms: i32, held_for: Duration, emitted: u32) -> u32 {
    let _ = (rate, delay_ms, held_for, emitted);
    todo!()
}

/// [`repeats_due`] capped at [`MAX_REPEATS_PER_POLL`].
#[must_use]
pub fn repeats_to_send(rate: i32, delay_ms: i32, held_for: Duration, emitted: u32) -> u32 {
    let _ = (rate, delay_ms, held_for, emitted);
    todo!()
}

/// The key a keysym stands for.
#[must_use]
pub fn raw_key_from_keysym(keysym: u32) -> RawKey {
    let _ = keysym;
    todo!()
}

/// The button an evdev button code stands for.
#[must_use]
pub fn raw_button_from_evdev(code: u32) -> RawButton {
    let _ = code;
    todo!()
}

/// A `wl_fixed_t` (24.8) as a float.
#[must_use]
pub fn fixed_to_f32(fixed: i32) -> f32 {
    let _ = fixed;
    todo!()
}

/// A surface position in `wl_fixed_t` as physical pixels at `scale`.
#[must_use]
pub fn pointer_position(x: i32, y: i32, scale: i32) -> (f32, f32) {
    let _ = (x, y, scale);
    todo!()
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
    let _ = frame;
    todo!()
}
