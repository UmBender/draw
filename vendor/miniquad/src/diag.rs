//! Backend diagnostics events (draw patch, ADR-T47-1).
//!
//! The Wayland backend pushes a [`BackendRecord`] for the seat, keyboard
//! and tablet events that explain input bugs: capabilities, devices,
//! focus, keys, key repeat and protocol errors. The application drains
//! them once per frame with [`take_events_into`] and keeps what it needs
//! in its own flight recorder. At most [`MAX_EVENTS`] are queued between
//! two takes; older ones are dropped and counted.

use std::sync::Mutex;
use std::time::Instant;

/// A seat device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Device {
    /// `wl_keyboard`.
    Keyboard,
    /// `wl_pointer`.
    Pointer,
    /// `wl_touch`.
    Touch,
}

/// One backend event worth recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendEvent {
    /// `wl_seat.capabilities`, with the announced devices.
    Capabilities {
        /// The seat has a keyboard.
        keyboard: bool,
        /// The seat has a pointer.
        pointer: bool,
        /// The seat has a touch screen.
        touch: bool,
    },
    /// The backend asked the seat for a device.
    DeviceCreated(Device),
    /// The backend released a device whose capability went away.
    DeviceReleased(Device),
    /// `wl_keyboard.enter`: the window got keyboard focus.
    KeyboardEnter,
    /// `wl_keyboard.leave`: the window lost keyboard focus.
    KeyboardLeave,
    /// `wl_keyboard.key`.
    Key {
        /// evdev scancode (the XKB keycode minus 8).
        code: u32,
        /// 0 released, 1 pressed, 2 repeated by the compositor.
        state: u32,
        /// `Ctrl` was held.
        ctrl: bool,
        /// `Alt` was held.
        alt: bool,
    },
    /// `wl_keyboard.repeat_info`.
    RepeatInfo {
        /// Repeats per second; 0 turns repeat off.
        rate: i32,
        /// Delay before the first repeat, in milliseconds.
        delay: i32,
    },
    /// The key repeat timer woke the loop.
    RepeatTimer {
        /// Timer expirations since the last read.
        expirations: u64,
    },
    /// `wl_keyboard.keymap`.
    Keymap,
    /// `wl_keyboard.keymap` with no usable keymap; the last one stays
    /// (draw patch, ADR-T79-1).
    KeymapIgnored {
        /// `wl_keyboard.keymap_format`; 0 is `no_keymap`.
        format: u32,
        /// Size of the keymap in bytes.
        size: u32,
    },
    /// A tablet tool was added.
    ToolAdded,
    /// A tablet tool was removed.
    ToolRemoved,
    /// A tablet tool came into proximity of the window.
    ProximityIn,
    /// A tablet tool left proximity.
    ProximityOut,
    /// The display connection failed with a protocol error.
    ProtocolError {
        /// `wl_display_get_error`.
        errno: i32,
    },
}

/// A [`BackendEvent`] and when it was pushed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendRecord {
    /// When the backend saw the event.
    pub at: Instant,
    /// The event.
    pub event: BackendEvent,
}

/// Most events queued between two [`take_events_into`] calls.
pub const MAX_EVENTS: usize = 4096;

struct Queue {
    events: Vec<BackendRecord>,
    dropped: u64,
}

static QUEUE: Mutex<Queue> = Mutex::new(Queue {
    events: Vec::new(),
    dropped: 0,
});

/// Moves the queued events, oldest first, into `out` (cleared first) and
/// returns how many were dropped since the last call. The two buffers are
/// swapped, so neither allocates once both have grown.
pub fn take_events_into(out: &mut Vec<BackendRecord>) -> u64 {
    out.clear();
    let mut queue = QUEUE.lock().unwrap_or_else(|e| e.into_inner());
    std::mem::swap(&mut queue.events, out);
    std::mem::take(&mut queue.dropped)
}

/// Queues `event`, stamped now. When the queue is full the older half is
/// dropped and counted.
#[allow(dead_code)] // only the Wayland backend pushes events
pub(crate) fn push(event: BackendEvent) {
    let at = Instant::now();
    let mut queue = QUEUE.lock().unwrap_or_else(|e| e.into_inner());
    if queue.events.len() >= MAX_EVENTS {
        queue.events.drain(..MAX_EVENTS / 2);
        queue.dropped += (MAX_EVENTS / 2) as u64;
    }
    queue.events.push(BackendRecord { at, event });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_keeps_newest_and_counts_dropped() {
        // Arrange
        let pushed = MAX_EVENTS + 10;

        // Act
        for code in 0..pushed {
            push(BackendEvent::Key {
                code: code as u32,
                state: 1,
                ctrl: false,
                alt: false,
            });
        }
        let mut out = Vec::new();
        let dropped = take_events_into(&mut out);
        let mut again = Vec::new();
        let dropped_again = take_events_into(&mut again);

        // Assert
        assert!(out.len() <= MAX_EVENTS);
        assert_eq!(dropped as usize, pushed - out.len());
        let last = out.last().map(|record| record.event);
        assert_eq!(
            last,
            Some(BackendEvent::Key {
                code: (pushed - 1) as u32,
                state: 1,
                ctrl: false,
                alt: false,
            })
        );
        assert!(again.is_empty());
        assert_eq!(dropped_again, 0);
    }
}
