//! Keys, modifiers and buttons of X11 as pure functions (T176 AC-1, AC-2).

use std::collections::HashSet;

use crate::platform::{RawButton, RawKey, RawMods};

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
        todo!("T176 behaviour: {keycode}")
    }

    /// Records the release of `keycode`.
    pub fn release(&mut self, keycode: u32) {
        todo!("T176 behaviour: {keycode}")
    }

    /// Forgets every held key (the window lost the focus).
    pub fn clear(&mut self) {
        todo!("T176 behaviour")
    }
}

/// The [`RawKey`] of a keysym; `keycode` is the X keycode, kept as the
/// evdev code (minus 8) for keys without their own variant.
#[must_use]
pub fn key_from_keysym(sym: u32, keycode: u32) -> RawKey {
    todo!("T176 behaviour: {sym} {keycode}")
}

/// The character a keysym types, `None` for control and function keys.
#[must_use]
pub fn char_from_keysym(sym: u32) -> Option<char> {
    todo!("T176 behaviour: {sym}")
}

/// Shift, control and alt of an X event `state`.
#[must_use]
pub fn mods_from_state(state: u32) -> RawMods {
    todo!("T176 behaviour: {state}")
}

/// The keysym level of a key for an event `state`: shift, caps lock (on
/// `alphabetic` keys only) and AltGr (Mod5).
#[must_use]
pub fn level_for(state: u32, alphabetic: bool) -> u32 {
    todo!("T176 behaviour: {state} {alphabetic}")
}

/// The keyboard group (layout) of an event `state`.
#[must_use]
pub fn group_of(state: u32) -> u32 {
    todo!("T176 behaviour: {state}")
}

/// The pointer button of an X button number (1 left, 2 middle, 3 right).
#[must_use]
pub fn button_from_x(button: u32) -> RawButton {
    todo!("T176 behaviour: {button}")
}

/// The wheel step of X buttons 4 to 7, `None` for any other button.
#[must_use]
pub fn wheel_delta(button: u32) -> Option<(f32, f32)> {
    todo!("T176 behaviour: {button}")
}
