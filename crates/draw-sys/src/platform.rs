//! The `Platform` trait and the raw events it delivers (ADR-T140-1).
//!
//! `draw-sys` cannot depend on `draw-core` (`tests/deps.rs`), so the events
//! are raw: the shell translates them into editor input.

use std::future::Future;
use std::time::Instant;

/// A seat device of a Wayland seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Device {
    /// `wl_keyboard`.
    Keyboard,
    /// `wl_pointer`.
    Pointer,
    /// `wl_touch`.
    Touch,
}

/// One backend event worth recording (ADR-T47-1): the seat, keyboard and
/// tablet events that explain input bugs.
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
    /// (ADR-T79-1).
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

/// A [`BackendEvent`] and when the backend saw it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendRecord {
    /// When the backend saw the event.
    pub at: Instant,
    /// The event.
    pub event: BackendEvent,
}

/// What a `wl_seat.capabilities` event does to one seat device (ADR-T44-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceChange {
    /// Ask the seat for the device and add its listener.
    Create,
    /// Release the device: its capability is gone.
    Release,
    /// Leave the device as it is.
    Keep,
}

/// What to do with a seat device that `exists` (or not) when its
/// capability is `announced` (or not) by a `capabilities` event. A
/// capability announced again keeps the device it already has.
#[must_use]
pub fn device_change(exists: bool, announced: bool) -> DeviceChange {
    match (exists, announced) {
        (false, true) => DeviceChange::Create,
        (true, false) => DeviceChange::Release,
        _ => DeviceChange::Keep,
    }
}

/// Whether seat devices of a `wl_seat` bound at `version` have the
/// `release` request (version 3 and later).
#[must_use]
pub fn has_release_request(version: u32) -> bool {
    version >= 3
}

/// Whether a plain pointer motion or button event is dropped because a
/// tablet tool is in proximity of the window (ADR-T40-1). `in_proximity`
/// holds, for every known tool, whether it is in proximity.
#[must_use]
pub fn pointer_shadowed_by_pen(in_proximity: impl IntoIterator<Item = bool>) -> bool {
    in_proximity.into_iter().any(|focused| focused)
}

/// A key as the platform reports it. Only the keys the shell reads have
/// their own variant; the rest are [`RawKey::Other`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawKey {
    /// A letter key, `'A'..='Z'`.
    Letter(char),
    /// A digit key on the main row, `0..=9`.
    Digit(u8),
    /// Escape.
    Escape,
    /// Space bar.
    Space,
    /// Tab.
    Tab,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Enter, either the main or the keypad one.
    Enter,
    /// Home.
    Home,
    /// End.
    End,
    /// Left arrow.
    ArrowLeft,
    /// Right arrow.
    ArrowRight,
    /// Up arrow.
    ArrowUp,
    /// Down arrow.
    ArrowDown,
    /// `[`.
    BracketLeft,
    /// `]`.
    BracketRight,
    /// Left shift.
    LeftShift,
    /// Right shift.
    RightShift,
    /// Left control.
    LeftControl,
    /// Right control.
    RightControl,
    /// Left alt.
    LeftAlt,
    /// Right alt.
    RightAlt,
    /// Any other key, by the platform's code.
    Other(u32),
}

/// Modifiers the platform reports with a key or character (the logo key
/// is not reported: the shell ignores it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RawMods {
    /// Shift held.
    pub shift: bool,
    /// Control held.
    pub ctrl: bool,
    /// Alt held.
    pub alt: bool,
}

/// A pointer button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawButton {
    /// Primary.
    Left,
    /// Wheel button.
    Middle,
    /// Secondary.
    Right,
    /// Any other button.
    Other,
}

/// A seat (input device group) change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatChange {
    /// A seat with this id appeared.
    Added(u32),
    /// The seat with this id went away.
    Removed(u32),
}

/// One input or window event, positions in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RawEvent {
    /// A key went down (`down`) or up.
    Key {
        /// The key.
        key: RawKey,
        /// Modifiers reported with it.
        mods: RawMods,
        /// Pressed or released.
        down: bool,
        /// An auto-repeat of a held key.
        repeat: bool,
    },
    /// A character was typed.
    Char {
        /// The character.
        ch: char,
        /// Modifiers reported with it.
        mods: RawMods,
        /// An auto-repeat of a held key.
        repeat: bool,
    },
    /// The pointer moved.
    PointerMove {
        /// Physical x.
        x: f32,
        /// Physical y.
        y: f32,
    },
    /// A pointer button went down (`down`) or up.
    Button {
        /// The button.
        button: RawButton,
        /// Physical x.
        x: f32,
        /// Physical y.
        y: f32,
        /// Pressed or released.
        down: bool,
    },
    /// The wheel turned, in platform units.
    Wheel {
        /// Horizontal amount.
        dx: f32,
        /// Vertical amount.
        dy: f32,
    },
    /// A pen sample.
    Tablet {
        /// Physical x.
        x: f32,
        /// Physical y.
        y: f32,
        /// Tip pressure in `0..=1`.
        pressure: f32,
    },
    /// The window changed size or scale.
    Resize(WindowMetrics),
    /// Keyboard focus came (`true`) or went.
    Focus(bool),
    /// A seat came or went.
    Seat(SeatChange),
    /// A backend diagnostics record, in order with the input
    /// (ADR-T162-1).
    Backend(BackendRecord),
}

/// Window size in logical pixels and physical pixels per logical pixel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowMetrics {
    /// Width in logical pixels.
    pub width: f32,
    /// Height in logical pixels.
    pub height: f32,
    /// Physical pixels per logical pixel.
    pub scale: f32,
}

/// A window and its input (ADR-T140-1). The shell is generic over it: no
/// trait objects.
pub trait Platform {
    /// Appends the events that arrived since the last poll to `out`, oldest
    /// first.
    fn poll(&mut self, out: &mut Vec<RawEvent>);

    /// The window's current size and scale.
    fn window(&mut self) -> WindowMetrics;

    /// The clipboard's text, `None` when empty or unavailable.
    fn clipboard_text(&mut self) -> Option<String>;

    /// Puts `text` on the clipboard.
    fn set_clipboard_text(&mut self, text: &str);

    /// Puts a PNG image on the clipboard.
    fn set_clipboard_png(&mut self, png: &[u8]);

    /// Asks for one more frame even without input.
    fn schedule_update(&mut self);

    /// Presents the frame and waits until the next one is due.
    fn next_frame(&mut self) -> impl Future<Output = ()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn device_change_announced_without_device_creates() {
        // Act
        let change = device_change(false, true);

        // Assert
        assert_eq!(change, DeviceChange::Create);
    }

    #[test]
    fn device_change_announced_with_device_keeps() {
        // Act
        let change = device_change(true, true);

        // Assert
        assert_eq!(change, DeviceChange::Keep);
    }

    #[test]
    fn device_change_withdrawn_with_device_releases() {
        // Act
        let change = device_change(true, false);

        // Assert
        assert_eq!(change, DeviceChange::Release);
    }

    #[test]
    fn device_change_withdrawn_without_device_keeps() {
        // Act
        let change = device_change(false, false);

        // Assert
        assert_eq!(change, DeviceChange::Keep);
    }

    #[test]
    fn has_release_request_needs_version_three() {
        // Act / Assert
        assert!(!has_release_request(2));
        assert!(has_release_request(3));
        assert!(has_release_request(9));
    }

    #[test]
    fn pointer_shadowed_by_pen_with_a_tool_in_proximity_is_true() {
        // Arrange: a pen in proximity, its eraser end not.
        let tools = [false, true];

        // Act
        let dropped = pointer_shadowed_by_pen(tools);

        // Assert
        assert!(dropped);
    }

    #[test]
    fn pointer_shadowed_by_pen_without_tools_in_proximity_is_false() {
        // Arrange
        let none: [bool; 0] = [];
        let away = [false, false];

        // Act / Assert
        assert!(!pointer_shadowed_by_pen(none));
        assert!(!pointer_shadowed_by_pen(away));
    }

    #[test]
    fn raw_event_backend_keeps_the_record() {
        // Arrange
        let record = BackendRecord {
            at: Instant::now(),
            event: BackendEvent::Key {
                code: 44,
                state: 1,
                ctrl: true,
                alt: false,
            },
        };

        // Act
        let event = RawEvent::Backend(record);

        // Assert
        assert_eq!(event, RawEvent::Backend(record));
    }
}
