//! Flight recorder records and the privacy rules for keys (ADR-T47-1).
//!
//! A [`Record`] is small and `Copy`, so recording never allocates. Its
//! text form (one line, at most [`MAX_LINE`] bytes) is made only when a
//! dump or a `trace` log is written. Plain text keys are [`KeyTag::Text`]
//! and characters are only a [`CharClass`]: nothing a user typed can be
//! read back.

use std::fmt;

use macroquad::input::KeyCode;
use macroquad::miniquad::KeyMods;
use macroquad::miniquad::diag::BackendEvent;
pub use macroquad::miniquad::diag::Device;

use crate::core::command::Tool;
use crate::core::input::PointerButton;

/// Longest text form of a [`Record`], in bytes.
pub const MAX_LINE: usize = 96;

/// A key as recorded by the app: named keys and shortcuts keep their code,
/// plain text keys do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyTag {
    /// A letter, digit, punctuation or space key without `Ctrl` or `Alt`.
    Text,
    /// Any other key, or a text key held with `Ctrl` or `Alt`.
    Named(KeyCode),
}

/// A `wl_keyboard.key` code as recorded: the same rule as [`KeyTag`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKey {
    /// A text key without `Ctrl` or `Alt`.
    Text,
    /// The evdev scancode of any other key.
    Code(u16),
}

/// What kind of character a char event carried, without the character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    /// An alphabetic character.
    Letter,
    /// A decimal digit.
    Digit,
    /// A space.
    Space,
    /// ASCII punctuation or a symbol.
    Punct,
    /// Anything else (control characters, other scripts' marks, …).
    Other,
}

/// What a drawn frame re-rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    /// The overlay only.
    Overlay,
    /// New shapes onto the cached layer.
    Append,
    /// The cached layer moved and scaled.
    Moved,
    /// The whole document layer.
    Full,
}

/// Something that should not happen; logged and dumped (ADR-T47-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anomaly {
    /// A key went down again while held, without a repeat flag.
    HeldKeyPressedAgain,
    /// One key press delivered more than one character.
    CharsPerPress,
    /// One wake of the key repeat timer had more than two expirations.
    RepeatBurst,
    /// A second `wl_keyboard` was created in one session.
    KeyboardRecreated,
    /// The backend dropped events before the app took them.
    BackendOverflow,
}

impl Anomaly {
    /// Every anomaly, in [`Anomaly::index`] order.
    pub const ALL: [Self; 5] = [
        Self::HeldKeyPressedAgain,
        Self::CharsPerPress,
        Self::RepeatBurst,
        Self::KeyboardRecreated,
        Self::BackendOverflow,
    ];

    /// Number of anomalies.
    pub const COUNT: usize = Self::ALL.len();

    /// Position in [`Anomaly::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        todo!()
    }

    /// A few words for logs and dumps.
    #[must_use]
    pub const fn describe(self) -> &'static str {
        todo!()
    }

    /// A short name for a dump file.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        todo!()
    }
}

/// One thing the flight recorder keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The recorder started.
    Start,
    /// `wl_seat.capabilities`.
    Capabilities {
        /// The seat has a keyboard.
        keyboard: bool,
        /// The seat has a pointer.
        pointer: bool,
        /// The seat has a touch screen.
        touch: bool,
    },
    /// The backend created a seat device.
    DeviceCreated(Device),
    /// The backend released a seat device.
    DeviceReleased(Device),
    /// The window got keyboard focus.
    KeyboardEnter,
    /// The window lost keyboard focus.
    KeyboardLeave,
    /// `wl_keyboard.key`.
    WlKey {
        /// The key, by the privacy rule.
        key: BackendKey,
        /// 0 released, 1 pressed, 2 repeated by the compositor.
        state: u8,
    },
    /// `wl_keyboard.repeat_info`.
    RepeatInfo {
        /// Repeats per second.
        rate: i32,
        /// Delay before the first repeat.
        delay_ms: i32,
    },
    /// The key repeat timer woke the loop.
    RepeatTimer {
        /// Expirations since the last read, saturated.
        expirations: u32,
    },
    /// A new keymap.
    Keymap,
    /// A tablet tool was added.
    ToolAdded,
    /// A tablet tool was removed.
    ToolRemoved,
    /// A tablet tool came into proximity.
    ProximityIn,
    /// A tablet tool left proximity.
    ProximityOut,
    /// The display connection failed.
    ProtocolError {
        /// `wl_display_get_error`.
        errno: i32,
    },
    /// The backend queue dropped events.
    BackendDropped {
        /// How many, saturated.
        count: u32,
    },
    /// A key went down, as the app saw it.
    KeyDown {
        /// The key, by the privacy rule.
        key: KeyTag,
        /// The platform marked it as a repeat.
        repeat: bool,
    },
    /// A key went up.
    KeyUp {
        /// The key, by the privacy rule.
        key: KeyTag,
    },
    /// A character arrived.
    Char {
        /// Its class only.
        class: CharClass,
        /// The platform marked it as a repeat.
        repeat: bool,
    },
    /// A pointer button went down.
    PointerDown(PointerButton),
    /// A pointer button went up.
    PointerUp(PointerButton),
    /// A wheel notch or scroll.
    Wheel,
    /// Pointer moves in one frame.
    PointerMoves {
        /// How many, saturated.
        count: u32,
    },
    /// A frame re-rendered something.
    Frame {
        /// What it re-rendered.
        kind: FrameKind,
        /// CPU time of the frame, saturated.
        micros: u32,
        /// Shapes in the document.
        shapes: u32,
    },
    /// The pointer tool changed.
    Tool(Tool),
    /// The document revision changed (an edit, undo or redo).
    Revision(u32),
    /// Input arrived after a long gap.
    Idle {
        /// Length of the gap.
        secs: u32,
    },
    /// A detector fired.
    Anomaly(Anomaly),
    /// The user asked for a dump.
    DumpRequested,
}

impl Event {
    /// Whether the event is user input (for the idle detector).
    #[must_use]
    pub const fn is_input(&self) -> bool {
        todo!()
    }
}

/// An [`Event`] and when it happened, in microseconds since the recorder
/// started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    /// Microseconds since the recorder started.
    pub micros: u64,
    /// The event.
    pub event: Event,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        todo!()
    }
}

impl fmt::Display for Record {
    /// `<seconds>.<micros> <event>`, one line.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        todo!()
    }
}

/// The recorded form of a key `code` pressed with `mods`.
#[must_use]
pub fn key_tag(code: KeyCode, mods: KeyMods) -> KeyTag {
    let _ = (code, mods);
    todo!()
}

/// The recorded form of a `wl_keyboard.key` evdev `code`.
#[must_use]
pub fn backend_key(code: u32, ctrl: bool, alt: bool) -> BackendKey {
    let _ = (code, ctrl, alt);
    todo!()
}

/// The class of `ch`.
#[must_use]
pub fn char_class(ch: char) -> CharClass {
    let _ = ch;
    todo!()
}

/// The record of a backend event, with keys by the privacy rule.
#[must_use]
pub fn from_backend(event: BackendEvent) -> Event {
    let _ = event;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command::Tool;
    use crate::core::input::PointerButton;
    use macroquad::input::KeyCode;
    use macroquad::miniquad::KeyMods;
    use macroquad::miniquad::diag::{BackendEvent, Device};

    fn mods(shift: bool, ctrl: bool, alt: bool) -> KeyMods {
        KeyMods {
            shift,
            ctrl,
            alt,
            logo: false,
        }
    }

    const NONE: KeyMods = KeyMods {
        shift: false,
        ctrl: false,
        alt: false,
        logo: false,
    };

    #[test]
    fn record_is_at_most_32_bytes() {
        // Arrange / Act / Assert
        assert!(std::mem::size_of::<Record>() <= 32);
    }

    #[test]
    fn key_tag_plain_letter_is_text() {
        // Arrange
        let codes = [
            KeyCode::A,
            KeyCode::Z,
            KeyCode::Key5,
            KeyCode::Space,
            KeyCode::Comma,
            KeyCode::GraveAccent,
            KeyCode::Kp7,
            KeyCode::KpAdd,
        ];

        for code in codes {
            // Act / Assert
            assert_eq!(key_tag(code, NONE), KeyTag::Text, "{code:?}");
        }
    }

    #[test]
    fn key_tag_shift_letter_is_text() {
        // Arrange / Act / Assert
        assert_eq!(key_tag(KeyCode::A, mods(true, false, false)), KeyTag::Text);
    }

    #[test]
    fn key_tag_ctrl_or_alt_letter_is_named() {
        // Arrange / Act / Assert
        assert_eq!(
            key_tag(KeyCode::Z, mods(false, true, false)),
            KeyTag::Named(KeyCode::Z)
        );
        assert_eq!(
            key_tag(KeyCode::Z, mods(true, false, true)),
            KeyTag::Named(KeyCode::Z)
        );
    }

    #[test]
    fn key_tag_named_key_keeps_code() {
        // Arrange
        let codes = [
            KeyCode::Escape,
            KeyCode::Enter,
            KeyCode::Left,
            KeyCode::Backspace,
            KeyCode::LeftShift,
            KeyCode::F5,
            KeyCode::Tab,
        ];

        for code in codes {
            // Act / Assert
            assert_eq!(key_tag(code, NONE), KeyTag::Named(code));
        }
    }

    #[test]
    fn backend_key_text_codes_are_hidden() {
        // Arrange: evdev codes of digits, letters, punctuation, space and
        // the keypad.
        for code in [2, 13, 16, 27, 30, 41, 44, 53, 57, 71, 83, 86] {
            // Act / Assert
            assert_eq!(backend_key(code, false, false), BackendKey::Text, "{code}");
        }
    }

    #[test]
    fn backend_key_with_ctrl_keeps_code() {
        // Arrange / Act / Assert
        assert_eq!(backend_key(44, true, false), BackendKey::Code(44));
        assert_eq!(backend_key(44, false, true), BackendKey::Code(44));
        // Esc, Enter, Left Ctrl and F1 are not text.
        for code in [1, 28, 29, 59] {
            assert_eq!(backend_key(code, false, false), BackendKey::Code(code as u16));
        }
    }

    #[test]
    fn char_class_hides_character() {
        // Arrange
        let cases = [
            ('a', CharClass::Letter),
            ('Z', CharClass::Letter),
            ('é', CharClass::Letter),
            ('7', CharClass::Digit),
            (' ', CharClass::Space),
            (',', CharClass::Punct),
            ('\u{7}', CharClass::Other),
        ];

        for (ch, class) in cases {
            // Act
            let record = Record {
                micros: 0,
                event: Event::Char {
                    class: char_class(ch),
                    repeat: false,
                },
            };

            // Assert
            assert_eq!(char_class(ch), class, "{ch:?}");
            if ch.is_alphanumeric() {
                assert!(!record.to_string().contains(ch), "{record}");
            }
        }
    }

    #[test]
    fn backend_event_maps_every_kind() {
        // Arrange
        let cases = [
            (
                BackendEvent::Capabilities {
                    keyboard: true,
                    pointer: false,
                    touch: true,
                },
                Event::Capabilities {
                    keyboard: true,
                    pointer: false,
                    touch: true,
                },
            ),
            (
                BackendEvent::DeviceCreated(Device::Keyboard),
                Event::DeviceCreated(Device::Keyboard),
            ),
            (
                BackendEvent::DeviceReleased(Device::Touch),
                Event::DeviceReleased(Device::Touch),
            ),
            (BackendEvent::KeyboardEnter, Event::KeyboardEnter),
            (BackendEvent::KeyboardLeave, Event::KeyboardLeave),
            (
                BackendEvent::Key {
                    code: 30,
                    state: 1,
                    ctrl: false,
                    alt: false,
                },
                Event::WlKey {
                    key: BackendKey::Text,
                    state: 1,
                },
            ),
            (
                BackendEvent::Key {
                    code: 1,
                    state: 0,
                    ctrl: false,
                    alt: false,
                },
                Event::WlKey {
                    key: BackendKey::Code(1),
                    state: 0,
                },
            ),
            (
                BackendEvent::RepeatInfo {
                    rate: 25,
                    delay: 600,
                },
                Event::RepeatInfo {
                    rate: 25,
                    delay_ms: 600,
                },
            ),
            (
                BackendEvent::RepeatTimer {
                    expirations: u64::MAX,
                },
                Event::RepeatTimer {
                    expirations: u32::MAX,
                },
            ),
            (BackendEvent::Keymap, Event::Keymap),
            (BackendEvent::ToolAdded, Event::ToolAdded),
            (BackendEvent::ToolRemoved, Event::ToolRemoved),
            (BackendEvent::ProximityIn, Event::ProximityIn),
            (BackendEvent::ProximityOut, Event::ProximityOut),
            (
                BackendEvent::ProtocolError { errno: 71 },
                Event::ProtocolError { errno: 71 },
            ),
        ];

        for (backend, expected) in cases {
            // Act / Assert
            assert_eq!(from_backend(backend), expected);
        }
    }

    #[test]
    fn every_record_line_fits_96_bytes() {
        // Arrange: every event, with its widest values.
        let events = [
            Event::Start,
            Event::Capabilities {
                keyboard: true,
                pointer: true,
                touch: true,
            },
            Event::DeviceCreated(Device::Keyboard),
            Event::DeviceReleased(Device::Pointer),
            Event::KeyboardEnter,
            Event::KeyboardLeave,
            Event::WlKey {
                key: BackendKey::Code(u16::MAX),
                state: u8::MAX,
            },
            Event::WlKey {
                key: BackendKey::Text,
                state: 2,
            },
            Event::RepeatInfo {
                rate: i32::MIN,
                delay_ms: i32::MIN,
            },
            Event::RepeatTimer {
                expirations: u32::MAX,
            },
            Event::Keymap,
            Event::ToolAdded,
            Event::ToolRemoved,
            Event::ProximityIn,
            Event::ProximityOut,
            Event::ProtocolError { errno: i32::MIN },
            Event::BackendDropped { count: u32::MAX },
            Event::KeyDown {
                key: KeyTag::Named(KeyCode::RightControl),
                repeat: true,
            },
            Event::KeyUp {
                key: KeyTag::Named(KeyCode::PrintScreen),
            },
            Event::Char {
                class: CharClass::Punct,
                repeat: true,
            },
            Event::PointerDown(PointerButton::Middle),
            Event::PointerUp(PointerButton::Right),
            Event::Wheel,
            Event::PointerMoves { count: u32::MAX },
            Event::Frame {
                kind: FrameKind::Moved,
                micros: u32::MAX,
                shapes: u32::MAX,
            },
            Event::Tool(Tool::Bucket),
            Event::Revision(u32::MAX),
            Event::Idle { secs: u32::MAX },
            Event::Anomaly(Anomaly::HeldKeyPressedAgain),
            Event::DumpRequested,
        ];

        for event in events {
            // Act
            let line = Record {
                micros: u64::MAX,
                event,
            }
            .to_string();

            // Assert
            assert!(line.len() <= MAX_LINE, "{} bytes: {line}", line.len());
            assert!(!line.contains('\n'), "{line}");
        }
        for anomaly in Anomaly::ALL {
            assert!(!anomaly.describe().is_empty());
        }
    }
}
