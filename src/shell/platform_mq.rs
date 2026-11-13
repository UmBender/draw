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
