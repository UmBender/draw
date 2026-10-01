//! Key chords mapped to editor commands.
//!
//! Owned by T11; the signature is fixed by T08, the body is a stub until T11
//! implements the bindings in `docs/architecture/Keymap.md`.

use crate::core::command::Command;
use crate::core::input::{Key, Modifiers};

/// The command bound to `key` pressed with `mods`, if any.
#[must_use]
pub fn resolve(key: Key, mods: Modifiers) -> Option<Command> {
    let _ = (key, mods);
    None
}
