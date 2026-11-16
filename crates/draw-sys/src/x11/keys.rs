//! Keys, modifiers and buttons of X11 as pure functions (T176 AC-1, AC-2).

use std::collections::HashSet;

use crate::platform::{RawButton, RawKey, RawMods};

const SHIFT_MASK: u32 = 1 << 0;
const LOCK_MASK: u32 = 1 << 1;
const CONTROL_MASK: u32 = 1 << 2;
const MOD1_MASK: u32 = 1 << 3;
const MOD5_MASK: u32 = 1 << 7;

/// The keys that are held down, to tell an autorepeat from a press.
///
/// Detectable autorepeat is on, so a held key sends presses with no
/// release between them: a press of a key already down is a repeat.
#[derive(Debug, Default)]
pub struct KeyTracker {
    down: HashSet<u32>,
}

impl KeyTracker {
    /// Records a press of `keycode`; `true` if the key was already down.
    pub fn press(&mut self, keycode: u32) -> bool {
        !self.down.insert(keycode)
    }

    /// Records the release of `keycode`.
    pub fn release(&mut self, keycode: u32) {
        self.down.remove(&keycode);
    }

    /// Forgets every held key (the window lost the focus).
    pub fn clear(&mut self) {
        self.down.clear();
    }
}

/// The [`RawKey`] of a keysym; `keycode` is the X keycode, kept as the
/// evdev code (minus 8) for keys without their own variant.
#[must_use]
pub fn key_from_keysym(sym: u32, keycode: u32) -> RawKey {
    match sym {
        0x41..=0x5a | 0x61..=0x7a => {
            RawKey::Letter(char::from(u8::try_from(sym).unwrap_or(b'A')).to_ascii_uppercase())
        }
        0x30..=0x39 => RawKey::Digit(u8::try_from(sym - 0x30).unwrap_or(0)),
        0xff1b => RawKey::Escape,
        0x20 => RawKey::Space,
        0xff09 | 0xfe20 => RawKey::Tab,
        0xff08 => RawKey::Backspace,
        0xffff => RawKey::Delete,
        0xff0d | 0xff8d => RawKey::Enter,
        0xff50 => RawKey::Home,
        0xff57 => RawKey::End,
        0xff51 => RawKey::ArrowLeft,
        0xff52 => RawKey::ArrowUp,
        0xff53 => RawKey::ArrowRight,
        0xff54 => RawKey::ArrowDown,
        0x5b => RawKey::BracketLeft,
        0x5d => RawKey::BracketRight,
        0xffe1 => RawKey::LeftShift,
        0xffe2 => RawKey::RightShift,
        0xffe3 => RawKey::LeftControl,
        0xffe4 => RawKey::RightControl,
        0xffe9 => RawKey::LeftAlt,
        0xffea => RawKey::RightAlt,
        _ => RawKey::Other(keycode.saturating_sub(8)),
    }
}

/// The character a keysym types, `None` for control and function keys.
#[must_use]
pub fn char_from_keysym(sym: u32) -> Option<char> {
    let code = match sym {
        0x20..=0x7e | 0xa0..=0xff => sym,
        0xffb0..=0xffb9 => sym - 0xffb0 + 0x30,
        0x0100_0000..=0x0110_ffff => sym - 0x0100_0000,
        _ => return None,
    };
    char::from_u32(code).filter(|c| !c.is_control())
}

/// Shift, control and alt of an X event `state`.
#[must_use]
pub fn mods_from_state(state: u32) -> RawMods {
    RawMods {
        shift: state & SHIFT_MASK != 0,
        ctrl: state & CONTROL_MASK != 0,
        alt: state & MOD1_MASK != 0,
    }
}

/// The keysym level of a key for an event `state`: shift, caps lock (on
/// `alphabetic` keys only) and `AltGr` (`Mod5`).
#[must_use]
pub fn level_for(state: u32, alphabetic: bool) -> u32 {
    let shifted = (state & SHIFT_MASK != 0) ^ (alphabetic && state & LOCK_MASK != 0);
    u32::from(shifted) | if state & MOD5_MASK != 0 { 2 } else { 0 }
}

/// The keyboard group (layout) of an event `state`.
#[must_use]
pub fn group_of(state: u32) -> u32 {
    (state >> 13) & 3
}

/// The pointer button of an X button number (1 left, 2 middle, 3 right).
#[must_use]
pub fn button_from_x(button: u32) -> RawButton {
    match button {
        1 => RawButton::Left,
        2 => RawButton::Middle,
        3 => RawButton::Right,
        _ => RawButton::Other,
    }
}

/// The wheel step of X buttons 4 to 7, `None` for any other button.
#[must_use]
pub fn wheel_delta(button: u32) -> Option<(f32, f32)> {
    match button {
        4 => Some((0.0, 1.0)),
        5 => Some((0.0, -1.0)),
        6 => Some((1.0, 0.0)),
        7 => Some((-1.0, 0.0)),
        _ => None,
    }
}
