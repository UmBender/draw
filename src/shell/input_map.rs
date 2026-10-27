//! Translates macroquad input into core input events.
//!
//! The shell registers a macroquad input subscriber and replays the raw
//! miniquad events into a [`Collector`] every frame (ADR-T12-1). Unlike
//! macroquad's polled state, this keeps every pointer sample and every click,
//! in arrival order. Raw positions are physical pixels; [`to_logical`] turns
//! them into the logical pixels used by the editor and the renderer.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use macroquad::input::{KeyCode, MouseButton};
use macroquad::miniquad::{EventHandler, KeyMods};

use crate::core::geom::Vec2;
use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};

/// The editor key for a macroquad key code, `None` for keys the editor ignores.
#[must_use]
pub fn map_key(code: KeyCode) -> Option<Key> {
    let key = match code {
        KeyCode::A => Key::A,
        KeyCode::B => Key::B,
        KeyCode::C => Key::C,
        KeyCode::D => Key::D,
        KeyCode::E => Key::E,
        KeyCode::F => Key::F,
        KeyCode::G => Key::G,
        KeyCode::H => Key::H,
        KeyCode::I => Key::I,
        KeyCode::J => Key::J,
        KeyCode::K => Key::K,
        KeyCode::L => Key::L,
        KeyCode::M => Key::M,
        KeyCode::N => Key::N,
        KeyCode::O => Key::O,
        KeyCode::P => Key::P,
        KeyCode::Q => Key::Q,
        KeyCode::R => Key::R,
        KeyCode::S => Key::S,
        KeyCode::T => Key::T,
        KeyCode::U => Key::U,
        KeyCode::V => Key::V,
        KeyCode::W => Key::W,
        KeyCode::X => Key::X,
        KeyCode::Y => Key::Y,
        KeyCode::Z => Key::Z,
        KeyCode::Key0 => Key::Digit0,
        KeyCode::Key1 => Key::Digit1,
        KeyCode::Key2 => Key::Digit2,
        KeyCode::Key3 => Key::Digit3,
        KeyCode::Key4 => Key::Digit4,
        KeyCode::Key5 => Key::Digit5,
        KeyCode::Key6 => Key::Digit6,
        KeyCode::Key7 => Key::Digit7,
        KeyCode::Key8 => Key::Digit8,
        KeyCode::Key9 => Key::Digit9,
        KeyCode::Delete => Key::Delete,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Escape => Key::Escape,
        KeyCode::Space => Key::Space,
        KeyCode::Tab => Key::Tab,
        KeyCode::LeftBracket => Key::BracketLeft,
        KeyCode::RightBracket => Key::BracketRight,
        KeyCode::Left => Key::ArrowLeft,
        KeyCode::Right => Key::ArrowRight,
        KeyCode::Up => Key::ArrowUp,
        KeyCode::Down => Key::ArrowDown,
        KeyCode::Enter | KeyCode::KpEnter => Key::Enter,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        _ => return None,
    };
    Some(key)
}

/// The editor button for a mouse button, `None` for unknown buttons.
#[must_use]
pub fn map_button(button: MouseButton) -> Option<PointerButton> {
    match button {
        MouseButton::Left => Some(PointerButton::Left),
        MouseButton::Middle => Some(PointerButton::Middle),
        MouseButton::Right => Some(PointerButton::Right),
        MouseButton::Unknown => None,
    }
}

/// Editor modifiers for miniquad's; the logo key is dropped.
#[must_use]
pub fn map_mods(mods: KeyMods) -> Modifiers {
    Modifiers {
        shift: mods.shift,
        ctrl: mods.ctrl,
        alt: mods.alt,
    }
}

/// Logical position of physical pixel `(x, y)` at DPI scale `dpi`. A DPI
/// scale that is not finite and positive counts as `1`.
#[must_use]
pub fn to_logical(x: f32, y: f32, dpi: f32) -> Vec2 {
    let scale = if dpi.is_finite() && dpi > 0.0 {
        dpi
    } else {
        1.0
    };
    Vec2::new(x / scale, y / scale)
}

/// Modifier keys currently held, one bit per key so that releasing one side
/// keeps the other held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct HeldMods(u8);

impl HeldMods {
    const LEFT_SHIFT: u8 = 1 << 0;
    const RIGHT_SHIFT: u8 = 1 << 1;
    const LEFT_CTRL: u8 = 1 << 2;
    const RIGHT_CTRL: u8 = 1 << 3;
    const LEFT_ALT: u8 = 1 << 4;
    const RIGHT_ALT: u8 = 1 << 5;
    const SHIFT: u8 = Self::LEFT_SHIFT | Self::RIGHT_SHIFT;
    const CTRL: u8 = Self::LEFT_CTRL | Self::RIGHT_CTRL;
    const ALT: u8 = Self::LEFT_ALT | Self::RIGHT_ALT;

    /// Records a modifier key going down (`true`) or up; other keys are ignored.
    fn set(&mut self, code: KeyCode, down: bool) {
        let bit = match code {
            KeyCode::LeftShift => Self::LEFT_SHIFT,
            KeyCode::RightShift => Self::RIGHT_SHIFT,
            KeyCode::LeftControl => Self::LEFT_CTRL,
            KeyCode::RightControl => Self::RIGHT_CTRL,
            KeyCode::LeftAlt => Self::LEFT_ALT,
            KeyCode::RightAlt => Self::RIGHT_ALT,
            _ => return,
        };
        if down {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }

    /// The held modifiers, either side counting.
    fn modifiers(self) -> Modifiers {
        Modifiers {
            shift: self.0 & Self::SHIFT != 0,
            ctrl: self.0 & Self::CTRL != 0,
            alt: self.0 & Self::ALT != 0,
        }
    }
}

/// Largest distance in physical pixels, per axis, between a pointer event
/// and the pen sample it is paired with.
const PEN_MATCH_PX: f32 = 0.01;

/// A pen sample from the window library: where a pen pointer event
/// happened and how hard the tip pressed (ADR-T35-1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PenSample {
    /// Position in physical pixels.
    pub pos: Vec2,
    /// Tip pressure in `[0, 1]`.
    pub pressure: f32,
}

/// How long a pen's left release is held back in case the tip bounces
/// straight down again (ADR-T42-1).
pub const PEN_BOUNCE_WINDOW: Duration = Duration::from_millis(60);

/// A pen release held back by the bounce window (ADR-T42-1).
#[derive(Debug, Clone, Copy, PartialEq)]
struct HeldRelease {
    /// The `PointerUp` to emit if the tip stays up.
    up: InputEvent,
    /// Frame time of the release.
    at: Instant,
    /// The last pointer move since the release.
    last_move: Option<InputEvent>,
}

/// Collects raw miniquad events as [`InputEvent`]s in arrival order.
///
/// Pointer events carry the modifiers whose keys are held (pointer events
/// have none of their own); key and character events carry the modifiers
/// the platform reports with them.
#[derive(Debug, Clone)]
pub struct Collector {
    events: Vec<InputEvent>,
    held: HeldMods,
    /// Last pointer position in logical pixels (for wheel events).
    pointer: Vec2,
    /// Physical pixels per logical pixel.
    dpi: f32,
    /// Wheel units the platform reports per notch.
    wheel_units_per_notch: f32,
    /// Pen samples of this frame not yet paired, oldest first.
    pen: VecDeque<PenSample>,
    /// Pressure of the last pointer event, as last emitted.
    pressure: Option<f32>,
    /// Pen bounce window; zero holds nothing (ADR-T42-1).
    bounce_window: Duration,
    /// Frame time of the events being replayed.
    now: Instant,
    /// The pen release held back, if any.
    bounce: Option<HeldRelease>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    /// An empty collector at DPI scale `1`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            held: HeldMods::default(),
            pointer: Vec2::ZERO,
            dpi: 1.0,
            wheel_units_per_notch: 1.0,
            pen: VecDeque::new(),
            pressure: None,
            bounce_window: Duration::ZERO,
            now: Instant::now(),
            bounce: None,
        }
    }

    /// Sets how many wheel units make one notch; ignored unless positive
    /// and finite.
    pub fn set_wheel_units_per_notch(&mut self, units: f32) {
        if units.is_finite() && units > 0.0 {
            self.wheel_units_per_notch = units;
        }
    }

    /// Replaces the queued pen samples with this frame's (ADR-T35-1).
    pub fn set_pen_samples(&mut self, samples: impl IntoIterator<Item = PenSample>) {
        self.pen.clear();
        self.pen.extend(samples);
    }

    /// Sets the pen bounce window (ADR-T42-1); zero turns it off.
    pub fn set_bounce_window(&mut self, window: Duration) {
        let _ = window;
        todo!()
    }

    /// Sets the frame time stamped on the following events.
    pub fn set_now(&mut self, now: Instant) {
        let _ = now;
        todo!()
    }

    /// Emits the held pen release, and the last move after it, once the
    /// bounce window has passed at the frame time. Returns whether a
    /// release is still held, so the caller can wake again for it.
    pub fn flush_bounce(&mut self) -> bool {
        todo!()
    }

    /// Sets the DPI scale used for the following pointer events.
    pub fn set_dpi(&mut self, dpi: f32) {
        self.dpi = dpi;
    }

    /// Removes and yields the collected events, oldest first.
    pub fn drain(&mut self) -> impl Iterator<Item = InputEvent> + '_ {
        self.events.drain(..)
    }

    /// Pairs a pointer event at physical `(x, y)` with the oldest queued pen
    /// sample there, dropping older samples, and emits
    /// [`InputEvent::Pressure`] if the pressure changed (ADR-T35-1).
    fn pair_pen(&mut self, x: f32, y: f32) {
        let near = |s: &PenSample| {
            (s.pos.x - x).abs() <= PEN_MATCH_PX && (s.pos.y - y).abs() <= PEN_MATCH_PX
        };
        let pressure = self.pen.iter().position(near).and_then(|i| {
            self.pen.drain(..i);
            self.pen.pop_front().map(|s| s.pressure)
        });
        if pressure.map(f32::to_bits) != self.pressure.map(f32::to_bits) {
            self.pressure = pressure;
            self.events.push(InputEvent::Pressure { pressure });
        }
    }

    /// Converts a physical position and remembers it as the pointer.
    fn track(&mut self, x: f32, y: f32) -> Vec2 {
        self.pointer = to_logical(x, y, self.dpi);
        self.pointer
    }
}

impl EventHandler for Collector {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.pair_pen(x, y);
        let pos = self.track(x, y);
        let mods = self.held.modifiers();
        self.events.push(InputEvent::PointerMove { pos, mods });
    }

    fn mouse_wheel_event(&mut self, _x: f32, y: f32) {
        if y != 0.0 {
            self.events.push(InputEvent::Scroll {
                pos: self.pointer,
                delta: y / self.wheel_units_per_notch,
            });
        }
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        let pos = self.track(x, y);
        if let Some(button) = map_button(button) {
            self.pair_pen(x, y);
            let mods = self.held.modifiers();
            self.events
                .push(InputEvent::PointerDown { pos, button, mods });
        }
    }

    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
        let pos = self.track(x, y);
        if let Some(button) = map_button(button) {
            self.pair_pen(x, y);
            let mods = self.held.modifiers();
            self.events
                .push(InputEvent::PointerUp { pos, button, mods });
        }
    }

    fn key_down_event(&mut self, code: KeyCode, mods: KeyMods, _repeat: bool) {
        self.held.set(code, true);
        if let Some(key) = map_key(code) {
            let mods = map_mods(mods);
            self.events.push(InputEvent::KeyDown { key, mods });
        }
    }

    fn char_event(&mut self, ch: char, mods: KeyMods, _repeat: bool) {
        let mods = map_mods(mods);
        self.events.push(InputEvent::Char { ch, mods });
    }

    fn key_up_event(&mut self, code: KeyCode, mods: KeyMods) {
        self.held.set(code, false);
        if let Some(key) = map_key(code) {
            let mods = map_mods(mods);
            self.events.push(InputEvent::KeyUp { key, mods });
        }
    }
}

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
    fn arrows_map() {
        for (code, key) in [
            (KeyCode::Left, Key::ArrowLeft),
            (KeyCode::Right, Key::ArrowRight),
            (KeyCode::Up, Key::ArrowUp),
            (KeyCode::Down, Key::ArrowDown),
        ] {
            assert_eq!(map_key(code), Some(key), "{code:?}");
        }
    }

    #[test]
    fn map_key_editing_keys() {
        for (code, key) in [
            (KeyCode::Enter, Key::Enter),
            (KeyCode::KpEnter, Key::Enter),
            (KeyCode::Home, Key::Home),
            (KeyCode::End, Key::End),
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
    fn collector_scroll_divides_by_units_per_notch() {
        // Arrange: a Wayland compositor reporting 15 units per notch
        let mut c = Collector::new();
        c.set_wheel_units_per_notch(15.0);
        // Act
        c.mouse_wheel_event(0.0, 15.0);
        c.mouse_wheel_event(0.0, -7.5);
        let events = drain(&mut c);
        // Assert
        let deltas: Vec<f32> = events
            .iter()
            .filter_map(|event| match event {
                InputEvent::Scroll { delta, .. } => Some(*delta),
                _ => None,
            })
            .collect();
        assert_eq!(deltas.len(), 2, "{events:?}");
        assert!(approx_eq(deltas[0], 1.0, EPS));
        assert!(approx_eq(deltas[1], -0.5, EPS));
    }

    #[test]
    fn collector_ignores_non_positive_units_per_notch() {
        // Arrange
        let mut c = Collector::new();
        c.set_wheel_units_per_notch(0.0);
        c.set_wheel_units_per_notch(f32::NAN);
        // Act
        c.mouse_wheel_event(0.0, 1.0);
        let events = drain(&mut c);
        // Assert: still one notch per unit
        let InputEvent::Scroll { delta, .. } = events[0] else {
            panic!("expected scroll, got {:?}", events[0]);
        };
        assert!(approx_eq(delta, 1.0, EPS));
    }

    #[test]
    fn collector_skips_unmapped() {
        // Arrange
        let mut c = Collector::new();
        // Act
        c.key_down_event(KeyCode::F5, NO_MODS, false);
        c.key_up_event(KeyCode::F6, NO_MODS);
        c.mouse_button_down_event(MouseButton::Unknown, 1.0, 1.0);
        c.mouse_button_up_event(MouseButton::Unknown, 1.0, 1.0);
        // Assert
        assert!(drain(&mut c).is_empty());
    }

    #[test]
    fn collector_char_event_becomes_char() {
        // Arrange
        let mut c = Collector::new();
        let ctrl = KeyMods {
            ctrl: true,
            ..NO_MODS
        };

        // Act: miniquad sends the character before its key.
        c.char_event('β', NO_MODS, false);
        c.key_down_event(KeyCode::B, NO_MODS, false);
        c.char_event('z', ctrl, true);
        let events = drain(&mut c);

        // Assert
        assert_eq!(
            events,
            [
                InputEvent::Char {
                    ch: 'β',
                    mods: Modifiers::NONE,
                },
                InputEvent::KeyDown {
                    key: Key::B,
                    mods: Modifiers::NONE,
                },
                InputEvent::Char {
                    ch: 'z',
                    mods: CTRL
                },
            ]
        );
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

    // ---- T35 pen samples ---------------------------------------------------

    fn sample(x: f32, y: f32, pressure: f32) -> PenSample {
        PenSample {
            pos: Vec2::new(x, y),
            pressure,
        }
    }

    fn pressure(p: Option<f32>) -> InputEvent {
        InputEvent::Pressure { pressure: p }
    }

    fn moved(x: f32, y: f32) -> InputEvent {
        InputEvent::PointerMove {
            pos: Vec2::new(x, y),
            mods: Modifiers::NONE,
        }
    }

    #[test]
    fn collector_pairs_pen_samples_with_pointer_events() {
        // Arrange: physical pixels at DPI 2.
        let mut c = Collector::new();
        c.set_dpi(2.0);
        c.set_pen_samples(vec![sample(20.0, 20.0, 0.3), sample(40.0, 20.0, 0.6)]);

        // Act
        c.mouse_motion_event(20.0, 20.0);
        c.mouse_button_down_event(MouseButton::Left, 40.0, 20.0);
        let events = drain(&mut c);

        // Assert
        assert_eq!(
            events,
            [
                pressure(Some(0.3)),
                moved(10.0, 10.0),
                pressure(Some(0.6)),
                InputEvent::PointerDown {
                    pos: Vec2::new(20.0, 10.0),
                    button: PointerButton::Left,
                    mods: Modifiers::NONE,
                },
            ]
        );
    }

    #[test]
    fn collector_emits_pressure_only_when_it_changes() {
        // Arrange
        let mut c = Collector::new();
        c.set_pen_samples(vec![sample(1.0, 1.0, 0.5), sample(2.0, 1.0, 0.5)]);

        // Act
        c.mouse_motion_event(1.0, 1.0);
        c.mouse_motion_event(2.0, 1.0);
        let events = drain(&mut c);

        // Assert
        assert_eq!(
            events,
            [pressure(Some(0.5)), moved(1.0, 1.0), moved(2.0, 1.0)]
        );
    }

    #[test]
    fn collector_mouse_after_pen_clears_pressure() {
        // Arrange
        let mut c = Collector::new();
        c.set_pen_samples(vec![sample(1.0, 1.0, 0.5)]);

        // Act
        c.mouse_motion_event(1.0, 1.0);
        c.mouse_motion_event(50.0, 50.0);
        c.mouse_motion_event(60.0, 50.0);
        let events = drain(&mut c);

        // Assert
        assert_eq!(
            events,
            [
                pressure(Some(0.5)),
                moved(1.0, 1.0),
                pressure(None),
                moved(50.0, 50.0),
                moved(60.0, 50.0),
            ]
        );
    }

    #[test]
    fn collector_drops_samples_older_than_the_match() {
        // Arrange
        let mut c = Collector::new();
        c.set_pen_samples(vec![
            sample(1.0, 1.0, 0.2),
            sample(2.0, 2.0, 0.4),
            sample(3.0, 3.0, 0.6),
        ]);

        // Act: the event at (1, 1) comes after the (2, 2) match.
        c.mouse_motion_event(2.0, 2.0);
        c.mouse_motion_event(1.0, 1.0);
        c.mouse_motion_event(3.0, 3.0);
        let events = drain(&mut c);

        // Assert
        assert_eq!(
            events,
            [
                pressure(Some(0.4)),
                moved(2.0, 2.0),
                pressure(None),
                moved(1.0, 1.0),
                pressure(Some(0.6)),
                moved(3.0, 3.0),
            ]
        );
    }

    // ---- T40 copied wl_pointer input on Wayland ---------------------------

    use macroquad::miniquad::pen::pointer_shadowed_by_pen;

    /// A `wl_pointer.motion` the compositor copied from the pen, as the
    /// Wayland backend forwards it given each tool's proximity.
    fn copied_pointer_motion(c: &mut Collector, in_proximity: &[bool], x: f32, y: f32) {
        if !pointer_shadowed_by_pen(in_proximity.iter().copied()) {
            c.mouse_motion_event(x, y);
        }
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

        // Act
        let dropped_none = pointer_shadowed_by_pen(none);
        let dropped_away = pointer_shadowed_by_pen(away);

        // Assert
        assert!(!dropped_none);
        assert!(!dropped_away);
    }

    #[test]
    fn collector_replayed_wayland_down_with_pen_in_proximity_gets_pressure() {
        // Arrange: Hyprland's tip-down from the T40 log. The tool frame
        // pushed two samples (motion, down) at the copied motion's spot.
        let (x, y) = (226.605_47, 121.929_69);
        let p = 10_539.0 / 65_535.0;
        let mut c = Collector::new();
        c.set_pen_samples(vec![sample(x, y, p), sample(x, y, p)]);

        // Act: wl_pointer copy, tool motion, tool down.
        copied_pointer_motion(&mut c, &[true], x, y);
        c.mouse_motion_event(x, y);
        c.mouse_button_down_event(MouseButton::Left, x, y);
        let events = drain(&mut c);

        // Assert
        assert_eq!(
            events,
            [
                pressure(Some(p)),
                moved(x, y),
                InputEvent::PointerDown {
                    pos: Vec2::new(x, y),
                    button: PointerButton::Left,
                    mods: Modifiers::NONE,
                },
            ]
        );
    }

    #[test]
    fn collector_replayed_wayland_motion_after_proximity_out_has_no_pressure() {
        // Arrange: the pen has left; the mouse moves.
        let mut c = Collector::new();
        c.set_pen_samples(vec![sample(1.0, 1.0, 0.5)]);
        c.mouse_motion_event(1.0, 1.0);

        // Act
        copied_pointer_motion(&mut c, &[false], 30.0, 40.0);
        let events = drain(&mut c);

        // Assert
        assert_eq!(
            events,
            [
                pressure(Some(0.5)),
                moved(1.0, 1.0),
                pressure(None),
                moved(30.0, 40.0),
            ]
        );
    }

    // ---- T42 pen tip bounces -----------------------------------------------

    use std::time::{Duration, Instant};

    const MS: Duration = Duration::from_millis(1);

    /// A collector in pen mode at frame time `t0`.
    fn bouncing(t0: Instant) -> Collector {
        let mut c = Collector::new();
        c.set_bounce_window(PEN_BOUNCE_WINDOW);
        c.set_now(t0);
        c
    }

    /// The pen moves to `(x, y)`.
    fn pen_move(c: &mut Collector, x: f32, y: f32) {
        c.set_pen_samples(vec![sample(x, y, 0.5)]);
        c.mouse_motion_event(x, y);
    }

    /// The pen tip goes down (`down`) or up at `(x, y)`.
    fn pen_tip(c: &mut Collector, down: bool, x: f32, y: f32) {
        c.set_pen_samples(vec![sample(x, y, 0.5)]);
        if down {
            c.mouse_button_down_event(MouseButton::Left, x, y);
        } else {
            c.mouse_button_up_event(MouseButton::Left, x, y);
        }
    }

    /// The events without pressure updates.
    fn actions(c: &mut Collector) -> Vec<InputEvent> {
        drain(c)
            .into_iter()
            .filter(|e| !matches!(e, InputEvent::Pressure { .. }))
            .collect()
    }

    fn button(down: bool, button: PointerButton, x: f32, y: f32) -> InputEvent {
        let pos = Vec2::new(x, y);
        let mods = Modifiers::NONE;
        if down {
            InputEvent::PointerDown { pos, button, mods }
        } else {
            InputEvent::PointerUp { pos, button, mods }
        }
    }

    #[test]
    fn collector_pen_bounce_within_window_joins_the_stroke() {
        // Arrange: a stroke in progress.
        let t0 = Instant::now();
        let mut c = bouncing(t0);
        pen_tip(&mut c, true, 10.0, 10.0);
        pen_move(&mut c, 20.0, 10.0);
        drain(&mut c);

        // Act: the tip bounces 20 ms later, 40 px further along.
        pen_tip(&mut c, false, 20.0, 10.0);
        pen_move(&mut c, 40.0, 10.0);
        c.set_now(t0 + 20 * MS);
        pen_tip(&mut c, true, 60.0, 10.0);
        let still_held = c.flush_bounce();
        pen_move(&mut c, 70.0, 10.0);

        // Assert
        assert!(!still_held);
        assert_eq!(actions(&mut c), [moved(60.0, 10.0), moved(70.0, 10.0)]);
    }

    #[test]
    fn collector_pen_release_is_held_inside_the_window() {
        // Arrange
        let t0 = Instant::now();
        let mut c = bouncing(t0);
        pen_tip(&mut c, true, 10.0, 10.0);
        drain(&mut c);

        // Act
        pen_tip(&mut c, false, 10.0, 10.0);
        pen_move(&mut c, 15.0, 10.0);
        c.set_now(t0 + PEN_BOUNCE_WINDOW - MS);
        let still_held = c.flush_bounce();

        // Assert
        assert!(still_held);
        assert_eq!(actions(&mut c), []);
    }

    #[test]
    fn collector_pen_release_after_the_window_is_handed_on() {
        // Arrange
        let t0 = Instant::now();
        let mut c = bouncing(t0);
        pen_tip(&mut c, true, 10.0, 10.0);
        drain(&mut c);
        pen_tip(&mut c, false, 10.0, 10.0);
        pen_move(&mut c, 15.0, 10.0);
        pen_move(&mut c, 18.0, 10.0);

        // Act
        c.set_now(t0 + PEN_BOUNCE_WINDOW + MS);
        let still_held = c.flush_bounce();
        pen_move(&mut c, 30.0, 10.0);

        // Assert: the release, the last held move, then moves as usual.
        assert!(!still_held);
        assert_eq!(
            actions(&mut c),
            [
                button(false, PointerButton::Left, 10.0, 10.0),
                moved(18.0, 10.0),
                moved(30.0, 10.0),
            ]
        );
    }

    #[test]
    fn collector_pen_press_after_the_window_starts_a_new_stroke() {
        // Arrange
        let t0 = Instant::now();
        let mut c = bouncing(t0);
        pen_tip(&mut c, true, 10.0, 10.0);
        drain(&mut c);
        pen_tip(&mut c, false, 10.0, 10.0);

        // Act: the next press comes 300 ms later, before any flush.
        c.set_now(t0 + 300 * MS);
        pen_tip(&mut c, true, 50.0, 50.0);

        // Assert
        assert_eq!(
            actions(&mut c),
            [
                button(false, PointerButton::Left, 10.0, 10.0),
                button(true, PointerButton::Left, 50.0, 50.0),
            ]
        );
    }

    #[test]
    fn collector_key_flushes_the_held_release_first() {
        // Arrange
        let t0 = Instant::now();
        let mut c = bouncing(t0);
        pen_tip(&mut c, true, 10.0, 10.0);
        drain(&mut c);
        pen_tip(&mut c, false, 10.0, 10.0);
        pen_move(&mut c, 12.0, 10.0);

        // Act
        c.key_down_event(KeyCode::Z, NO_MODS, false);
        c.char_event('z', NO_MODS, false);
        c.mouse_wheel_event(0.0, 1.0);

        // Assert
        let events = actions(&mut c);
        assert_eq!(
            events[..3],
            [
                button(false, PointerButton::Left, 10.0, 10.0),
                moved(12.0, 10.0),
                InputEvent::KeyDown {
                    key: Key::Z,
                    mods: Modifiers::NONE,
                },
            ]
        );
        assert_eq!(events.len(), 5, "{events:?}");
        assert!(!c.flush_bounce());
    }

    #[test]
    fn collector_other_button_flushes_the_held_release_first() {
        // Arrange
        let t0 = Instant::now();
        let mut c = bouncing(t0);
        pen_tip(&mut c, true, 10.0, 10.0);
        drain(&mut c);
        pen_tip(&mut c, false, 10.0, 10.0);

        // Act: a barrel button, 5 ms later.
        c.set_now(t0 + 5 * MS);
        c.set_pen_samples(vec![sample(10.0, 10.0, 0.0)]);
        c.mouse_button_down_event(MouseButton::Middle, 10.0, 10.0);

        // Assert
        assert_eq!(
            actions(&mut c),
            [
                button(false, PointerButton::Left, 10.0, 10.0),
                button(true, PointerButton::Middle, 10.0, 10.0),
            ]
        );
    }

    #[test]
    fn collector_mouse_release_is_never_held() {
        // Arrange: no pen samples, so these are mouse events.
        let mut c = bouncing(Instant::now());
        c.mouse_button_down_event(MouseButton::Left, 10.0, 10.0);
        drain(&mut c);

        // Act
        c.mouse_button_up_event(MouseButton::Left, 10.0, 10.0);
        let still_held = c.flush_bounce();

        // Assert
        assert!(!still_held);
        assert_eq!(
            actions(&mut c),
            [button(false, PointerButton::Left, 10.0, 10.0)]
        );
    }

    #[test]
    fn collector_without_a_window_passes_pen_releases() {
        // Arrange: no `set_bounce_window`.
        let mut c = Collector::new();
        c.set_now(Instant::now());
        pen_tip(&mut c, true, 10.0, 10.0);
        drain(&mut c);

        // Act
        pen_tip(&mut c, false, 10.0, 10.0);
        pen_tip(&mut c, true, 20.0, 10.0);

        // Assert
        assert_eq!(
            actions(&mut c),
            [
                button(false, PointerButton::Left, 10.0, 10.0),
                button(true, PointerButton::Left, 20.0, 10.0),
            ]
        );
    }
}
