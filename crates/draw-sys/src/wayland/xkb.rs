//! `libxkbcommon`, loaded with `dlopen` (ADR-T142-1): the keymap the
//! compositor sends, and the key state that turns keycodes into keysyms,
//! characters and modifiers (T169).

#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_void};
use std::rc::Rc;

use crate::loader::{Library, LoadError};
use crate::platform::RawMods;

/// `XKB_KEYMAP_FORMAT_TEXT_V1`.
const FORMAT_TEXT_V1: i32 = 1;
/// `XKB_STATE_MODS_EFFECTIVE`.
const MODS_EFFECTIVE: i32 = 8;
/// The evdev scancode to XKB keycode offset.
const KEYCODE_OFFSET: u32 = 8;

type Ptr = *mut c_void;

/// The functions of `libxkbcommon`, kept with the library (and the context)
/// for as long as any keymap or state made through them lives.
struct Inner {
    context: Ptr,
    context_unref: unsafe extern "C" fn(Ptr), // TEST: fuzz_keymap_decoder_never_panics
    keymap_from_string: unsafe extern "C" fn(Ptr, *const c_char, i32, i32) -> Ptr, // TEST: fuzz_keymap_decoder_never_panics
    keymap_unref: unsafe extern "C" fn(Ptr), // TEST: fuzz_keymap_decoder_never_panics
    keymap_key_repeats: unsafe extern "C" fn(Ptr, u32) -> i32, // TEST: wayland_keyboard_typed_text_arrives
    state_new: unsafe extern "C" fn(Ptr) -> Ptr, // TEST: wayland_keyboard_typed_text_arrives
    state_unref: unsafe extern "C" fn(Ptr),      // TEST: wayland_keyboard_typed_text_arrives
    state_update_mask: unsafe extern "C" fn(Ptr, u32, u32, u32, u32, u32, u32) -> i32, // TEST: wayland_keyboard_modifiers_arrive
    state_key_get_one_sym: unsafe extern "C" fn(Ptr, u32) -> u32, // TEST: wayland_keyboard_typed_text_arrives
    state_key_get_utf8: unsafe extern "C" fn(Ptr, u32, *mut c_char, usize) -> i32, // TEST: wayland_keyboard_typed_text_arrives
    state_mod_active: unsafe extern "C" fn(Ptr, *const c_char, i32) -> i32, // TEST: wayland_keyboard_modifiers_arrive
    _library: Library,
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Inner")
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        // TEST: fuzz_keymap_decoder_never_panics
        // SAFETY: `context` came from `xkb_context_new` and is released once.
        unsafe { (self.context_unref)(self.context) };
    }
}

/// `libxkbcommon`, loaded.
#[derive(Debug, Clone)]
pub struct Xkb {
    inner: Rc<Inner>,
}

/// A parsed keymap.
#[derive(Debug)]
pub struct Keymap {
    inner: Rc<Inner>,
    raw: Ptr,
}

/// The modifier and key state of one keyboard over a [`Keymap`].
#[derive(Debug)]
pub struct KeyState {
    inner: Rc<Inner>,
    raw: Ptr,
}

/// Loads `name` from `library`, typed as the field it initialises.
fn symbol<F: Copy>(library: &Library, name: &str) -> Result<F, LoadError> {
    // TEST: fuzz_keymap_decoder_never_panics
    // SAFETY: every call site assigns the result to a field whose type is
    // the C signature of the named libxkbcommon function.
    unsafe { library.symbol::<F>(name) }.map(|s| *s)
}

impl Xkb {
    /// Loads `libxkbcommon.so.0`.
    ///
    /// # Errors
    /// [`LoadError`] if the library or a function is missing, or no context
    /// can be made (reported as a missing `xkb_context_new` result).
    pub fn load() -> Result<Self, LoadError> {
        let library = Library::open("libxkbcommon.so.0")?;
        // TEST: fuzz_keymap_decoder_never_panics
        let context_new: unsafe extern "C" fn(i32) -> Ptr = symbol(&library, "xkb_context_new")?;
        // TEST: fuzz_keymap_decoder_never_panics
        // SAFETY: `xkb_context_new(0)` takes only flags and returns a context
        // or null.
        let context = unsafe { context_new(0) };
        if context.is_null() {
            return Err(LoadError::SymbolNotFound {
                name: "xkb_context_new".to_owned(),
                reason: "it returned no context".to_owned(),
            });
        }
        let inner = Inner {
            context,
            context_unref: symbol(&library, "xkb_context_unref")?,
            keymap_from_string: symbol(&library, "xkb_keymap_new_from_string")?,
            keymap_unref: symbol(&library, "xkb_keymap_unref")?,
            keymap_key_repeats: symbol(&library, "xkb_keymap_key_repeats")?,
            state_new: symbol(&library, "xkb_state_new")?,
            state_unref: symbol(&library, "xkb_state_unref")?,
            state_update_mask: symbol(&library, "xkb_state_update_mask")?,
            state_key_get_one_sym: symbol(&library, "xkb_state_key_get_one_sym")?,
            state_key_get_utf8: symbol(&library, "xkb_state_key_get_utf8")?,
            state_mod_active: symbol(&library, "xkb_state_mod_name_is_active")?,
            _library: library,
        };
        Ok(Self {
            inner: Rc::new(inner),
        })
    }

    /// Parses keymap text (the format `XKB_V1`); `None` if it is not a
    /// usable keymap. The text is cut at its first NUL, if any.
    #[must_use]
    pub fn keymap_from_text(&self, text: &[u8]) -> Option<Keymap> {
        let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
        let mut owned = Vec::with_capacity(end + 1);
        owned.extend_from_slice(&text[..end]);
        owned.push(0);
        // TEST: fuzz_keymap_decoder_never_panics
        // SAFETY: `context` is live; `owned` is a NUL-terminated buffer that
        // lives through the call; format and flags are valid values.
        let raw = unsafe {
            (self.inner.keymap_from_string)(
                self.inner.context,
                owned.as_ptr().cast(),
                FORMAT_TEXT_V1,
                0,
            )
        };
        (!raw.is_null()).then(|| Keymap {
            inner: Rc::clone(&self.inner),
            raw,
        })
    }
}

impl Keymap {
    /// A fresh key state over this keymap; `None` if libxkbcommon fails.
    #[must_use]
    pub fn state(&self) -> Option<KeyState> {
        // TEST: wayland_keyboard_typed_text_arrives
        // SAFETY: `raw` is the live keymap of this value.
        let raw = unsafe { (self.inner.state_new)(self.raw) };
        (!raw.is_null()).then(|| KeyState {
            inner: Rc::clone(&self.inner),
            raw,
        })
    }

    /// Whether the key with this evdev scancode repeats when held.
    #[must_use]
    pub fn key_repeats(&self, scancode: u32) -> bool {
        // TEST: wayland_keyboard_typed_text_arrives
        // SAFETY: `raw` is the live keymap of this value.
        let repeats = unsafe {
            (self.inner.keymap_key_repeats)(self.raw, scancode.wrapping_add(KEYCODE_OFFSET))
        };
        repeats != 0
    }
}

impl Drop for Keymap {
    fn drop(&mut self) {
        // TEST: fuzz_keymap_decoder_never_panics
        // SAFETY: `raw` is the keymap of this value, released once.
        unsafe { (self.inner.keymap_unref)(self.raw) };
    }
}

impl KeyState {
    /// Applies a `wl_keyboard.modifiers` event.
    pub fn update_mask(&mut self, depressed: u32, latched: u32, locked: u32, group: u32) {
        // TEST: wayland_keyboard_modifiers_arrive
        // SAFETY: `raw` is the live state of this value.
        unsafe {
            (self.inner.state_update_mask)(self.raw, depressed, latched, locked, 0, 0, group)
        };
    }

    /// The keysym of the key with this evdev scancode in the current state.
    #[must_use]
    pub fn keysym(&self, scancode: u32) -> u32 {
        // TEST: wayland_keyboard_typed_text_arrives
        // SAFETY: `raw` is the live state of this value.
        unsafe {
            (self.inner.state_key_get_one_sym)(self.raw, scancode.wrapping_add(KEYCODE_OFFSET))
        }
    }

    /// The text the key types in the current state; empty for none.
    #[must_use]
    pub fn utf8(&self, scancode: u32) -> String {
        let mut buf = [0 as c_char; 32];
        // TEST: wayland_keyboard_typed_text_arrives
        // SAFETY: `raw` is the live state; `buf` is writable for its length.
        let written = unsafe {
            (self.inner.state_key_get_utf8)(
                self.raw,
                scancode.wrapping_add(KEYCODE_OFFSET),
                buf.as_mut_ptr(),
                buf.len(),
            )
        };
        if written <= 0 {
            return String::new();
        }
        // TEST: wayland_keyboard_typed_text_arrives
        // SAFETY: libxkbcommon NUL-terminates the buffer it writes into.
        let text = unsafe { CStr::from_ptr(buf.as_ptr()) };
        text.to_string_lossy().into_owned()
    }

    /// The effective modifiers the shell reads.
    #[must_use]
    pub fn mods(&self) -> RawMods {
        let active = |name: &CStr| {
            // TEST: wayland_keyboard_modifiers_arrive
            // SAFETY: `raw` is the live state; `name` is NUL-terminated.
            unsafe { (self.inner.state_mod_active)(self.raw, name.as_ptr(), MODS_EFFECTIVE) > 0 }
        };
        RawMods {
            shift: active(c"Shift"),
            ctrl: active(c"Control"),
            alt: active(c"Mod1"),
        }
    }
}

impl Drop for KeyState {
    fn drop(&mut self) {
        // TEST: wayland_keyboard_typed_text_arrives
        // SAFETY: `raw` is the state of this value, released once.
        unsafe { (self.inner.state_unref)(self.raw) };
    }
}
