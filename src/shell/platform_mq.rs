//! Translation from miniquad's events to [`RawEvent`] (ADR-T162-1).
//!
//! The only module outside the window setup that names miniquad's input
//! types: the collector, the flight recorder and the detectors see
//! [`RawEvent`], [`RawKey`], [`RawMods`] and [`RawButton`] only.

use std::cell::RefCell;

use draw_sys::platform::{
    BackendEvent, BackendRecord, Device, RawButton, RawEvent, RawKey, RawMods,
};
use macroquad::input::{KeyCode, MouseButton};
use macroquad::miniquad::diag as mq_diag;
use macroquad::miniquad::{EventHandler, KeyMods};

use crate::shell::input_map::Collector;

/// The platform key for a miniquad key code. Keys without a variant keep
/// miniquad's code (an X11 keysym) in [`RawKey::Other`], so the flight
/// recorder's privacy rule still tells text keys from the others.
#[must_use]
#[allow(
    clippy::wildcard_enum_match_arm,
    reason = "KeyCode is miniquad's, about 120 keys; ADR-T131-1"
)]
pub const fn raw_key(code: KeyCode) -> RawKey {
    match code {
        KeyCode::A => RawKey::Letter('A'),
        KeyCode::B => RawKey::Letter('B'),
        KeyCode::C => RawKey::Letter('C'),
        KeyCode::D => RawKey::Letter('D'),
        KeyCode::E => RawKey::Letter('E'),
        KeyCode::F => RawKey::Letter('F'),
        KeyCode::G => RawKey::Letter('G'),
        KeyCode::H => RawKey::Letter('H'),
        KeyCode::I => RawKey::Letter('I'),
        KeyCode::J => RawKey::Letter('J'),
        KeyCode::K => RawKey::Letter('K'),
        KeyCode::L => RawKey::Letter('L'),
        KeyCode::M => RawKey::Letter('M'),
        KeyCode::N => RawKey::Letter('N'),
        KeyCode::O => RawKey::Letter('O'),
        KeyCode::P => RawKey::Letter('P'),
        KeyCode::Q => RawKey::Letter('Q'),
        KeyCode::R => RawKey::Letter('R'),
        KeyCode::S => RawKey::Letter('S'),
        KeyCode::T => RawKey::Letter('T'),
        KeyCode::U => RawKey::Letter('U'),
        KeyCode::V => RawKey::Letter('V'),
        KeyCode::W => RawKey::Letter('W'),
        KeyCode::X => RawKey::Letter('X'),
        KeyCode::Y => RawKey::Letter('Y'),
        KeyCode::Z => RawKey::Letter('Z'),
        KeyCode::Key0 => RawKey::Digit(0),
        KeyCode::Key1 => RawKey::Digit(1),
        KeyCode::Key2 => RawKey::Digit(2),
        KeyCode::Key3 => RawKey::Digit(3),
        KeyCode::Key4 => RawKey::Digit(4),
        KeyCode::Key5 => RawKey::Digit(5),
        KeyCode::Key6 => RawKey::Digit(6),
        KeyCode::Key7 => RawKey::Digit(7),
        KeyCode::Key8 => RawKey::Digit(8),
        KeyCode::Key9 => RawKey::Digit(9),
        KeyCode::Escape => RawKey::Escape,
        KeyCode::Space => RawKey::Space,
        KeyCode::Tab => RawKey::Tab,
        KeyCode::Backspace => RawKey::Backspace,
        KeyCode::Delete => RawKey::Delete,
        KeyCode::Enter | KeyCode::KpEnter => RawKey::Enter,
        KeyCode::Home => RawKey::Home,
        KeyCode::End => RawKey::End,
        KeyCode::Left => RawKey::ArrowLeft,
        KeyCode::Right => RawKey::ArrowRight,
        KeyCode::Up => RawKey::ArrowUp,
        KeyCode::Down => RawKey::ArrowDown,
        KeyCode::LeftBracket => RawKey::BracketLeft,
        KeyCode::RightBracket => RawKey::BracketRight,
        KeyCode::LeftShift => RawKey::LeftShift,
        KeyCode::RightShift => RawKey::RightShift,
        KeyCode::LeftControl => RawKey::LeftControl,
        KeyCode::RightControl => RawKey::RightControl,
        KeyCode::LeftAlt => RawKey::LeftAlt,
        KeyCode::RightAlt => RawKey::RightAlt,
        other => RawKey::Other(other as u32),
    }
}

/// The platform modifiers for miniquad's; the logo key is dropped.
#[must_use]
pub const fn raw_mods(mods: KeyMods) -> RawMods {
    RawMods {
        shift: mods.shift,
        ctrl: mods.ctrl,
        alt: mods.alt,
    }
}

/// The platform button for a miniquad mouse button.
#[must_use]
pub const fn raw_button(button: MouseButton) -> RawButton {
    match button {
        MouseButton::Left => RawButton::Left,
        MouseButton::Middle => RawButton::Middle,
        MouseButton::Right => RawButton::Right,
        MouseButton::Unknown => RawButton::Other,
    }
}

/// Replays miniquad's events into a [`Collector`] as [`RawEvent`]s.
#[derive(Debug)]
pub struct MqInput<'a>(pub &'a mut Collector);

impl EventHandler for MqInput<'_> {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.0.handle(RawEvent::PointerMove { x, y });
    }

    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.0.handle(RawEvent::Wheel { dx: x, dy: y });
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        self.0.handle(RawEvent::Button {
            button: raw_button(button),
            x,
            y,
            down: true,
        });
    }

    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
        self.0.handle(RawEvent::Button {
            button: raw_button(button),
            x,
            y,
            down: false,
        });
    }

    fn key_down_event(&mut self, code: KeyCode, mods: KeyMods, repeat: bool) {
        self.0.handle(RawEvent::Key {
            key: raw_key(code),
            mods: raw_mods(mods),
            down: true,
            repeat,
        });
    }

    fn char_event(&mut self, ch: char, mods: KeyMods, repeat: bool) {
        self.0.handle(RawEvent::Char {
            ch,
            mods: raw_mods(mods),
            repeat,
        });
    }

    fn key_up_event(&mut self, code: KeyCode, mods: KeyMods) {
        self.0.handle(RawEvent::Key {
            key: raw_key(code),
            mods: raw_mods(mods),
            down: false,
            repeat: false,
        });
    }
}

/// The platform device for miniquad's.
const fn device(device: mq_diag::Device) -> Device {
    match device {
        mq_diag::Device::Keyboard => Device::Keyboard,
        mq_diag::Device::Pointer => Device::Pointer,
        mq_diag::Device::Touch => Device::Touch,
    }
}

/// The platform backend event for miniquad's.
const fn backend_event(event: mq_diag::BackendEvent) -> BackendEvent {
    match event {
        mq_diag::BackendEvent::Capabilities {
            keyboard,
            pointer,
            touch,
        } => BackendEvent::Capabilities {
            keyboard,
            pointer,
            touch,
        },
        mq_diag::BackendEvent::DeviceCreated(d) => BackendEvent::DeviceCreated(device(d)),
        mq_diag::BackendEvent::DeviceReleased(d) => BackendEvent::DeviceReleased(device(d)),
        mq_diag::BackendEvent::KeyboardEnter => BackendEvent::KeyboardEnter,
        mq_diag::BackendEvent::KeyboardLeave => BackendEvent::KeyboardLeave,
        mq_diag::BackendEvent::Key {
            code,
            state,
            ctrl,
            alt,
        } => BackendEvent::Key {
            code,
            state,
            ctrl,
            alt,
        },
        mq_diag::BackendEvent::RepeatInfo { rate, delay } => {
            BackendEvent::RepeatInfo { rate, delay }
        }
        mq_diag::BackendEvent::RepeatTimer { expirations } => {
            BackendEvent::RepeatTimer { expirations }
        }
        mq_diag::BackendEvent::Keymap => BackendEvent::Keymap,
        mq_diag::BackendEvent::KeymapIgnored { format, size } => {
            BackendEvent::KeymapIgnored { format, size }
        }
        mq_diag::BackendEvent::ToolAdded => BackendEvent::ToolAdded,
        mq_diag::BackendEvent::ToolRemoved => BackendEvent::ToolRemoved,
        mq_diag::BackendEvent::ProximityIn => BackendEvent::ProximityIn,
        mq_diag::BackendEvent::ProximityOut => BackendEvent::ProximityOut,
        mq_diag::BackendEvent::ProtocolError { errno } => BackendEvent::ProtocolError { errno },
    }
}

thread_local! {
    /// miniquad's records between taking and converting them; reused so a
    /// frame does not allocate.
    static SCRATCH: RefCell<Vec<mq_diag::BackendRecord>> = const { RefCell::new(Vec::new()) };
}

/// Moves miniquad's queued backend records, oldest first, into `out`
/// (cleared first) and returns how many the queue dropped since the last
/// call.
pub fn take_backend_into(out: &mut Vec<BackendRecord>) -> u64 {
    out.clear();
    SCRATCH.with_borrow_mut(|scratch| {
        let dropped = mq_diag::take_events_into(scratch);
        out.extend(scratch.drain(..).map(|r| BackendRecord {
            at: r.at,
            event: backend_event(r.event),
        }));
        dropped
    })
}

#[cfg(test)]
pub(crate) mod keys {
    //! Test helpers: build [`RawEvent`]s with miniquad's key code names.

    use super::*;
    use crate::shell::input_map::Collector;
    use macroquad::input::KeyCode as Mq;

    /// miniquad's key code names as [`RawKey`] constants.
    pub struct KeyCode;

    macro_rules! codes {
        ($($name:ident),* $(,)?) => {
            #[allow(non_upper_case_globals, reason = "the names of miniquad's key codes")]
            impl KeyCode {
                $(pub const $name: RawKey = raw_key(Mq::$name);)*
            }
        };
    }

    codes!(
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Key0, Key1,
        Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Backspace, Comma, Delete, Down, End, Enter,
        Escape, F1, F5, F6, GraveAccent, Home, Kp1, Kp7, KpAdd, KpEnter, Left, LeftAlt,
        LeftBracket, LeftControl, LeftShift, LeftSuper, Minus, PrintScreen, Right, RightAlt,
        RightBracket, RightControl, RightShift, Space, Tab, Unknown, Up,
    );

    /// Feeds a [`Collector`] with the calls miniquad's event handler has.
    pub trait Drive {
        fn mouse_motion_event(&mut self, x: f32, y: f32);
        fn mouse_wheel_event(&mut self, x: f32, y: f32);
        fn mouse_button_down_event(&mut self, button: RawButton, x: f32, y: f32);
        fn mouse_button_up_event(&mut self, button: RawButton, x: f32, y: f32);
        fn key_down_event(&mut self, code: RawKey, mods: RawMods, repeat: bool);
        fn char_event(&mut self, ch: char, mods: RawMods, repeat: bool);
        fn key_up_event(&mut self, code: RawKey, mods: RawMods);
    }

    impl Drive for Collector {
        fn mouse_motion_event(&mut self, x: f32, y: f32) {
            self.handle(RawEvent::PointerMove { x, y });
        }

        fn mouse_wheel_event(&mut self, x: f32, y: f32) {
            self.handle(RawEvent::Wheel { dx: x, dy: y });
        }

        fn mouse_button_down_event(&mut self, button: RawButton, x: f32, y: f32) {
            self.handle(RawEvent::Button {
                button,
                x,
                y,
                down: true,
            });
        }

        fn mouse_button_up_event(&mut self, button: RawButton, x: f32, y: f32) {
            self.handle(RawEvent::Button {
                button,
                x,
                y,
                down: false,
            });
        }

        fn key_down_event(&mut self, code: RawKey, mods: RawMods, repeat: bool) {
            self.handle(RawEvent::Key {
                key: code,
                mods,
                down: true,
                repeat,
            });
        }

        fn char_event(&mut self, ch: char, mods: RawMods, repeat: bool) {
            self.handle(RawEvent::Char { ch, mods, repeat });
        }

        fn key_up_event(&mut self, code: RawKey, mods: RawMods) {
            self.handle(RawEvent::Key {
                key: code,
                mods,
                down: false,
                repeat: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::Vec2;
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use crate::shell::input_map::Collector;
    use macroquad::input::{KeyCode as Mq, MouseButton};
    use macroquad::miniquad::{EventHandler, KeyMods};

    #[test]
    fn raw_key_letters_and_digits_map_to_their_variants() {
        // Act / Assert
        assert_eq!(raw_key(Mq::A), RawKey::Letter('A'));
        assert_eq!(raw_key(Mq::Z), RawKey::Letter('Z'));
        assert_eq!(raw_key(Mq::Key0), RawKey::Digit(0));
        assert_eq!(raw_key(Mq::Key7), RawKey::Digit(7));
        assert_eq!(raw_key(Mq::Escape), RawKey::Escape);
        assert_eq!(raw_key(Mq::LeftBracket), RawKey::BracketLeft);
        assert_eq!(raw_key(Mq::RightControl), RawKey::RightControl);
    }

    #[test]
    fn raw_key_unlisted_key_keeps_its_code() {
        // Act
        let key = raw_key(Mq::F5);

        // Assert
        assert_eq!(key, RawKey::Other(Mq::F5 as u32));
        assert_ne!(key, raw_key(Mq::F6));
    }

    #[test]
    fn raw_key_keypad_enter_is_enter() {
        // Act / Assert
        assert_eq!(raw_key(Mq::KpEnter), RawKey::Enter);
        assert_eq!(raw_key(Mq::Enter), RawKey::Enter);
    }

    #[test]
    fn raw_mods_drop_logo() {
        // Arrange
        let mods = KeyMods {
            shift: true,
            ctrl: false,
            alt: true,
            logo: true,
        };

        // Act
        let raw = raw_mods(mods);

        // Assert
        assert_eq!(
            raw,
            RawMods {
                shift: true,
                ctrl: false,
                alt: true,
            }
        );
    }

    #[test]
    fn raw_button_unknown_is_other() {
        // Act / Assert
        assert_eq!(raw_button(MouseButton::Left), RawButton::Left);
        assert_eq!(raw_button(MouseButton::Middle), RawButton::Middle);
        assert_eq!(raw_button(MouseButton::Right), RawButton::Right);
        assert_eq!(raw_button(MouseButton::Unknown), RawButton::Other);
    }

    #[test]
    fn mq_input_replays_events_in_order() {
        // Arrange
        let mut collector = Collector::new();
        let none = KeyMods {
            shift: false,
            ctrl: false,
            alt: false,
            logo: false,
        };

        // Act
        {
            let mut input = MqInput(&mut collector);
            input.key_down_event(Mq::P, none, false);
            input.mouse_button_down_event(MouseButton::Left, 10.0, 20.0);
            input.mouse_motion_event(11.0, 21.0);
            input.mouse_button_up_event(MouseButton::Left, 11.0, 21.0);
        }
        let events: Vec<InputEvent> = collector.drain().collect();

        // Assert
        let mods = Modifiers::NONE;
        assert_eq!(
            events,
            [
                InputEvent::KeyDown {
                    key: Key::P,
                    mods
                },
                InputEvent::PointerDown {
                    pos: Vec2::new(10.0, 20.0),
                    button: PointerButton::Left,
                    mods,
                },
                InputEvent::PointerMove {
                    pos: Vec2::new(11.0, 21.0),
                    mods,
                },
                InputEvent::PointerUp {
                    pos: Vec2::new(11.0, 21.0),
                    button: PointerButton::Left,
                    mods,
                },
            ]
        );
    }
}
