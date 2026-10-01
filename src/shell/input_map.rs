//! Translates macroquad input into core input events.
//!
//! Owned by T12; filled in by that task.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use macroquad::input::{KeyCode, MouseButton};
    use macroquad::miniquad::{EventHandler, KeyMods};

    const EPS: f32 = 1e-5;

    const NO_MODS: KeyMods = KeyMods {
        shift: false,
        ctrl: false,
        alt: false,
        logo: false,
    };

    const CTRL: Modifiers = Modifiers {
        shift: false,
        ctrl: true,
        alt: false,
    };

    fn pos_eq(a: Vec2, b: Vec2) -> bool {
        a.approx_eq(b, EPS)
    }

    fn drain(collector: &mut Collector) -> Vec<InputEvent> {
        collector.drain().collect()
    }

    // AC-1

    #[test]
    fn map_key_letters_and_digits() {
        let letters = [
            (KeyCode::A, Key::A),
            (KeyCode::B, Key::B),
            (KeyCode::C, Key::C),
            (KeyCode::D, Key::D),
            (KeyCode::E, Key::E),
            (KeyCode::F, Key::F),
            (KeyCode::G, Key::G),
            (KeyCode::H, Key::H),
            (KeyCode::I, Key::I),
            (KeyCode::J, Key::J),
            (KeyCode::K, Key::K),
            (KeyCode::L, Key::L),
            (KeyCode::M, Key::M),
            (KeyCode::N, Key::N),
            (KeyCode::O, Key::O),
            (KeyCode::P, Key::P),
            (KeyCode::Q, Key::Q),
            (KeyCode::R, Key::R),
            (KeyCode::S, Key::S),
            (KeyCode::T, Key::T),
            (KeyCode::U, Key::U),
            (KeyCode::V, Key::V),
            (KeyCode::W, Key::W),
            (KeyCode::X, Key::X),
            (KeyCode::Y, Key::Y),
            (KeyCode::Z, Key::Z),
        ];
        let digits = [
            (KeyCode::Key0, Key::Digit0),
            (KeyCode::Key1, Key::Digit1),
            (KeyCode::Key2, Key::Digit2),
            (KeyCode::Key3, Key::Digit3),
            (KeyCode::Key4, Key::Digit4),
            (KeyCode::Key5, Key::Digit5),
            (KeyCode::Key6, Key::Digit6),
            (KeyCode::Key7, Key::Digit7),
            (KeyCode::Key8, Key::Digit8),
            (KeyCode::Key9, Key::Digit9),
        ];
        for (code, key) in letters.into_iter().chain(digits) {
            assert_eq!(map_key(code), Some(key), "{code:?}");
        }
    }

    #[test]
    fn map_key_named_keys() {
        for (code, key) in [
            (KeyCode::Delete, Key::Delete),
            (KeyCode::Backspace, Key::Backspace),
            (KeyCode::Escape, Key::Escape),
            (KeyCode::Space, Key::Space),
            (KeyCode::Tab, Key::Tab),
            (KeyCode::LeftBracket, Key::BracketLeft),
            (KeyCode::RightBracket, Key::BracketRight),
        ] {
            assert_eq!(map_key(code), Some(key), "{code:?}");
        }
    }

    #[test]
    fn map_key_unbound_is_none() {
        for code in [
            KeyCode::LeftShift,
            KeyCode::RightControl,
            KeyCode::LeftAlt,
            KeyCode::LeftSuper,
            KeyCode::Enter,
            KeyCode::F1,
            KeyCode::Kp1,
            KeyCode::Minus,
            KeyCode::Unknown,
        ] {
            assert_eq!(map_key(code), None, "{code:?}");
        }
    }

    #[test]
    fn map_button_all() {
        assert_eq!(map_button(MouseButton::Left), Some(PointerButton::Left));
        assert_eq!(map_button(MouseButton::Middle), Some(PointerButton::Middle));
        assert_eq!(map_button(MouseButton::Right), Some(PointerButton::Right));
        assert_eq!(map_button(MouseButton::Unknown), None);
    }

    #[test]
    fn map_mods_drops_logo() {
        // Arrange
        let mods = KeyMods {
            shift: true,
            ctrl: false,
            alt: true,
            logo: true,
        };
        // Act
        let mapped = map_mods(mods);
        // Assert
        assert_eq!(
            mapped,
            Modifiers {
                shift: true,
                ctrl: false,
                alt: true,
            }
        );
    }

    #[test]
    fn to_logical_divides_by_dpi() {
        let p = to_logical(300.0, 150.0, 2.0);
        assert!(pos_eq(p, Vec2::new(150.0, 75.0)));
    }

    #[test]
    fn to_logical_bad_dpi_is_identity() {
        for dpi in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            let p = to_logical(30.0, 40.0, dpi);
            assert!(pos_eq(p, Vec2::new(30.0, 40.0)), "dpi {dpi}");
        }
    }

    // AC-1b

    #[test]
    fn collector_translates_in_order() {
        // Arrange
        let mut c = Collector::new();
        // Act
        c.key_down_event(KeyCode::P, NO_MODS, false);
        c.mouse_button_down_event(MouseButton::Left, 10.0, 20.0);
        c.mouse_motion_event(11.0, 21.0);
        c.mouse_motion_event(12.0, 22.0);
        c.mouse_button_up_event(MouseButton::Left, 12.0, 22.0);
        c.key_down_event(KeyCode::P, NO_MODS, true);
        c.key_up_event(KeyCode::P, NO_MODS);
        let events = drain(&mut c);
        // Assert
        let none = Modifiers::NONE;
        let expected = [
            InputEvent::KeyDown {
                key: Key::P,
                mods: none,
            },
            InputEvent::PointerDown {
                pos: Vec2::new(10.0, 20.0),
                button: PointerButton::Left,
                mods: none,
            },
            InputEvent::PointerMove {
                pos: Vec2::new(11.0, 21.0),
                mods: none,
            },
            InputEvent::PointerMove {
                pos: Vec2::new(12.0, 22.0),
                mods: none,
            },
            InputEvent::PointerUp {
                pos: Vec2::new(12.0, 22.0),
                button: PointerButton::Left,
                mods: none,
            },
            InputEvent::KeyDown {
                key: Key::P,
                mods: none,
            },
            InputEvent::KeyUp {
                key: Key::P,
                mods: none,
            },
        ];
        assert_eq!(events, expected);
        assert!(drain(&mut c).is_empty(), "drain empties the queue");
    }

    #[test]
    fn collector_scroll_uses_last_pointer() {
        // Arrange
        let mut c = Collector::new();
        // Act
        c.mouse_motion_event(40.0, 50.0);
        c.mouse_wheel_event(0.0, -1.0);
        c.mouse_wheel_event(3.0, 0.0);
        let events = drain(&mut c);
        // Assert
        assert_eq!(events.len(), 2, "{events:?}");
        let InputEvent::Scroll { pos, delta } = events[1] else {
            panic!("expected scroll, got {:?}", events[1]);
        };
        assert!(pos_eq(pos, Vec2::new(40.0, 50.0)));
        assert!(approx_eq(delta, -1.0, EPS));
    }

    #[test]
    fn collector_skips_unmapped() {
        // Arrange
        let mut c = Collector::new();
        // Act
        c.key_down_event(KeyCode::F5, NO_MODS, false);
        c.key_up_event(KeyCode::Enter, NO_MODS);
        c.mouse_button_down_event(MouseButton::Unknown, 1.0, 1.0);
        c.mouse_button_up_event(MouseButton::Unknown, 1.0, 1.0);
        // Assert
        assert!(drain(&mut c).is_empty());
    }

    #[test]
    fn collector_pointer_mods_follow_modifier_keys() {
        // Arrange
        let mut c = Collector::new();
        let ctrl_mods = KeyMods {
            ctrl: true,
            ..NO_MODS
        };
        // Act
        c.key_down_event(KeyCode::LeftControl, NO_MODS, false);
        c.key_down_event(KeyCode::RightControl, ctrl_mods, false);
        c.mouse_button_down_event(MouseButton::Left, 1.0, 1.0);
        c.key_up_event(KeyCode::LeftControl, ctrl_mods);
        c.mouse_motion_event(2.0, 2.0);
        c.key_up_event(KeyCode::RightControl, ctrl_mods);
        c.mouse_motion_event(3.0, 3.0);
        c.key_down_event(KeyCode::LeftShift, NO_MODS, false);
        c.mouse_button_up_event(MouseButton::Left, 3.0, 3.0);
        let events = drain(&mut c);
        // Assert
        let shift = Modifiers {
            shift: true,
            ..Modifiers::NONE
        };
        let mods: Vec<Modifiers> = events
            .iter()
            .filter_map(|e| match *e {
                InputEvent::PointerDown { mods, .. }
                | InputEvent::PointerMove { mods, .. }
                | InputEvent::PointerUp { mods, .. } => Some(mods),
                _ => None,
            })
            .collect();
        assert_eq!(mods, [CTRL, CTRL, Modifiers::NONE, shift]);
    }

    #[test]
    fn collector_scales_by_dpi() {
        // Arrange
        let mut c = Collector::new();
        c.set_dpi(2.0);
        // Act
        c.mouse_button_down_event(MouseButton::Right, 100.0, 60.0);
        c.mouse_wheel_event(0.0, 1.0);
        let events = drain(&mut c);
        // Assert
        let expected = Vec2::new(50.0, 30.0);
        match events[..] {
            [
                InputEvent::PointerDown { pos: down, .. },
                InputEvent::Scroll { pos: wheel, .. },
            ] => {
                assert!(pos_eq(down, expected));
                assert!(pos_eq(wheel, expected));
            }
            _ => panic!("unexpected events {events:?}"),
        }
    }

    #[test]
    fn collector_key_events_carry_event_mods() {
        // Arrange
        let mut c = Collector::new();
        let mods = KeyMods {
            ctrl: true,
            shift: true,
            ..NO_MODS
        };
        // Act
        c.key_down_event(KeyCode::Z, mods, false);
        let events = drain(&mut c);
        // Assert
        assert_eq!(
            events,
            [InputEvent::KeyDown {
                key: Key::Z,
                mods: Modifiers {
                    shift: true,
                    ctrl: true,
                    alt: false,
                },
            }]
        );
    }
}
