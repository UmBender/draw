//! Flight recorder records and the privacy rules for keys (ADR-T47-1).

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
