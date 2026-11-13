//! Translation from miniquad's events to [`RawEvent`] (ADR-T162-1).
//!
//! The only module outside the window setup that names miniquad's input
//! types: the collector, the flight recorder and the detectors see
//! [`RawEvent`], [`RawKey`], [`RawMods`] and [`RawButton`] only.

use std::cell::RefCell;
use std::future::Future;

use draw_sys::platform::{
    BackendEvent, BackendRecord, Device, Platform, RawButton, RawEvent, RawKey, RawMods,
    WindowMetrics,
};
use std::time::Instant;

use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::input::{KeyCode, MouseButton};
use macroquad::miniquad::diag as mq_diag;
use macroquad::miniquad::pen;
use macroquad::miniquad::window::{
    clipboard_get, clipboard_set, clipboard_set_png, schedule_update,
};
use macroquad::miniquad::{EventHandler, KeyMods};
use macroquad::window::{next_frame, screen_dpi_scale, screen_height, screen_width};

use crate::shell::diag::{self, event::Event};

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

/// Collects miniquad's events as [`RawEvent`]s, oldest first.
#[derive(Debug)]
pub struct MqInput<'a>(pub &'a mut Vec<RawEvent>);

impl EventHandler for MqInput<'_> {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.0.push(RawEvent::PointerMove { x, y });
    }

    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.0.push(RawEvent::Wheel { dx: x, dy: y });
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        self.0.push(RawEvent::Button {
            button: raw_button(button),
            x,
            y,
            down: true,
        });
    }

    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
        self.0.push(RawEvent::Button {
            button: raw_button(button),
            x,
            y,
            down: false,
        });
    }

    fn key_down_event(&mut self, code: KeyCode, mods: KeyMods, repeat: bool) {
        self.0.push(RawEvent::Key {
            key: raw_key(code),
            mods: raw_mods(mods),
            down: true,
            repeat,
        });
    }

    fn char_event(&mut self, ch: char, mods: KeyMods, repeat: bool) {
        self.0.push(RawEvent::Char {
            ch,
            mods: raw_mods(mods),
            repeat,
        });
    }

    fn key_up_event(&mut self, code: KeyCode, mods: KeyMods) {
        self.0.push(RawEvent::Key {
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

/// Appends the pen samples miniquad queued since the last call to `out`
/// as [`RawEvent::Tablet`] (ADR-T35-1).
pub fn push_tablet_events(out: &mut Vec<RawEvent>) {
    out.extend(
        pen::take_samples()
            .into_iter()
            .map(|s| RawEvent::Tablet {
                x: s.x,
                y: s.y,
                pressure: s.pressure,
            }),
    );
}

/// The miniquad and macroquad window as a [`Platform`] (ADR-T140-1): the
/// window metrics, the clipboard, the wake-up and the frame loop, and the
/// input, pen samples and backend records as [`RawEvent`]s.
#[derive(Debug)]
pub struct MqPlatform {
    /// macroquad's input subscriber for this window.
    subscriber: usize,
    /// Backend records between taking and converting them.
    backend: Vec<BackendRecord>,
}

impl MqPlatform {
    /// A platform that reads the input of `subscriber`
    /// (`register_input_subscriber`).
    #[must_use]
    pub fn new(subscriber: usize) -> Self {
        Self {
            subscriber,
            backend: Vec::with_capacity(256),
        }
    }

    /// A platform on the running window: registers an input subscriber, so
    /// call it once the window exists.
    #[must_use]
    pub fn open() -> Self {
        Self::new(register_input_subscriber())
    }
}

impl Platform for MqPlatform {
    fn poll(&mut self, out: &mut Vec<RawEvent>) {
        let dropped = take_backend_into(&mut self.backend);
        if dropped > 0 {
            // `RawEvent` has no variant for the count: log it here.
            let count = u32::try_from(dropped).unwrap_or(u32::MAX);
            let mut record = vec![(Instant::now(), Event::BackendDropped { count })];
            diag::record_all(&mut record);
        }
        out.extend(self.backend.drain(..).map(RawEvent::Backend));
        push_tablet_events(out);
        repeat_all_miniquad_input(&mut MqInput(out), self.subscriber);
    }

    fn window(&mut self) -> WindowMetrics {
        WindowMetrics {
            width: screen_width(),
            height: screen_height(),
            scale: screen_dpi_scale(),
        }
    }

    fn clipboard_text(&mut self) -> Option<String> {
        clipboard_get()
    }

    fn set_clipboard_text(&mut self, text: &str) {
        clipboard_set(text);
    }

    fn set_clipboard_png(&mut self, png: &[u8]) {
        clipboard_set_png(png);
    }

    fn schedule_update(&mut self) {
        schedule_update();
    }

    fn next_frame(&mut self) -> impl Future<Output = ()> {
        next_frame()
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
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        M,
        N,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,
        Key0,
        Key1,
        Key2,
        Key3,
        Key4,
        Key5,
        Key6,
        Key7,
        Key8,
        Key9,
        Backspace,
        Comma,
        Delete,
        Down,
        End,
        Enter,
        Escape,
        F1,
        F5,
        F6,
        GraveAccent,
        Home,
        Kp1,
        Kp7,
        KpAdd,
        KpEnter,
        Left,
        LeftAlt,
        LeftBracket,
        LeftControl,
        LeftShift,
        LeftSuper,
        Minus,
        PrintScreen,
        Right,
        RightBracket,
        RightControl,
        Space,
        Tab,
        Unknown,
        Up,
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
        let mut raw = Vec::new();
        {
            let mut input = MqInput(&mut raw);
            input.key_down_event(Mq::P, none, false);
            input.mouse_button_down_event(MouseButton::Left, 10.0, 20.0);
            input.mouse_motion_event(11.0, 21.0);
            input.mouse_button_up_event(MouseButton::Left, 11.0, 21.0);
        }
        for event in raw {
            collector.handle(event);
        }
        let events: Vec<InputEvent> = collector.drain().collect();

        // Assert
        let mods = Modifiers::NONE;
        assert_eq!(
            events,
            [
                InputEvent::KeyDown { key: Key::P, mods },
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

    fn takes_platform<P: draw_sys::platform::Platform>(_: &P) {}

    #[test]
    fn mq_platform_is_a_platform() {
        // Arrange
        let platform = MqPlatform::new(0);

        // Act / Assert: compiles only if the trait is implemented.
        takes_platform(&platform);
    }

    #[test]
    fn mq_events_collect_raw_events_in_arrival_order() {
        // Arrange
        let mut raw = Vec::new();
        let mods = KeyMods {
            shift: false,
            ctrl: true,
            alt: false,
            logo: false,
        };

        // Act
        {
            let mut input = MqInput(&mut raw);
            input.key_down_event(Mq::R, mods, false);
            input.mouse_motion_event(3.0, 4.0);
            input.mouse_wheel_event(0.0, 1.0);
        }

        // Assert
        let ctrl = RawMods {
            shift: false,
            ctrl: true,
            alt: false,
        };
        assert_eq!(
            raw,
            [
                RawEvent::Key {
                    key: RawKey::Letter('R'),
                    mods: ctrl,
                    down: true,
                    repeat: false,
                },
                RawEvent::PointerMove { x: 3.0, y: 4.0 },
                RawEvent::Wheel { dx: 0.0, dy: 1.0 },
            ]
        );
    }

    #[test]
    fn tablet_events_are_empty_without_a_window() {
        // Arrange / Act: no backend has run, so nothing was pushed.
        let mut out = Vec::new();
        push_tablet_events(&mut out);

        // Assert
        assert_eq!(out, []);
    }
}
