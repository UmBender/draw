//! `libxkbcommon`, loaded with `dlopen` (ADR-T142-1): the keymap the
//! compositor sends, and the key state that turns keycodes into keysyms,
//! characters and modifiers (T169).

#![allow(unsafe_code)]

use std::ffi::c_void;

use crate::loader::{Library, LoadError};

/// The functions of `libxkbcommon` the backend uses, with the library kept
/// loaded as long as the table lives.
#[derive(Debug)]
pub struct Xkb {
    _library: Library,
}

/// A parsed keymap.
#[derive(Debug)]
pub struct Keymap {
    _raw: *mut c_void,
}

/// The modifier and key state of one keyboard over a [`Keymap`].
#[derive(Debug)]
pub struct KeyState {
    _raw: *mut c_void,
}

impl Xkb {
    /// Loads `libxkbcommon.so.0`.
    ///
    /// # Errors
    /// [`LoadError`] if the library or a function is missing.
    pub fn load() -> Result<Self, LoadError> {
        todo!()
    }

    /// Parses keymap text (the format `XKB_V1`); `None` if it is not a
    /// usable keymap. The text need not end in a NUL.
    #[must_use]
    pub fn keymap_from_text(&self, text: &[u8]) -> Option<Keymap> {
        let _ = text;
        todo!()
    }
}
