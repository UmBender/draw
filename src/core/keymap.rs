//! Key chords mapped to editor commands.
//!
//! [`BINDINGS`] is the single table of the bindings documented in
//! `docs/architecture/Keymap.md`; the toolbar and docs can list it, and
//! [`resolve`] looks chords up in it. A binding fires only when the held
//! modifiers match the chord's modifiers exactly (ADR-T11-1). `Space` is not
//! bound: the editor consumes it as the pan modifier.

use crate::core::command::{Command, Tool};
use crate::core::input::{Key, Modifiers};
use crate::core::palette::ColorId;

/// A key pressed together with an exact set of modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyChord {
    /// The key.
    pub key: Key,
    /// The modifiers that must be held, no more and no fewer.
    pub mods: Modifiers,
}

impl KeyChord {
    /// `key` with no modifier.
    #[must_use]
    pub const fn bare(key: Key) -> Self {
        Self {
            key,
            mods: Modifiers::NONE,
        }
    }

    /// `Ctrl` + `key`.
    #[must_use]
    pub const fn ctrl(key: Key) -> Self {
        Self {
            key,
            mods: Modifiers {
                shift: false,
                ctrl: true,
                alt: false,
            },
        }
    }

    /// `Ctrl` + `Shift` + `key`.
    #[must_use]
    pub const fn ctrl_shift(key: Key) -> Self {
        Self {
            key,
            mods: Modifiers {
                shift: true,
                ctrl: true,
                alt: false,
            },
        }
    }
}

/// Palette colour for number key `digit` (`1`–`6`), checked at compile time.
const fn color(digit: u8) -> Command {
    match ColorId::from_key_digit(digit) {
        Some(id) => Command::SetColor(id),
        None => panic!("number key has no palette colour"),
    }
}

/// Every binding: chord, command and a short human-readable description.
///
/// Order follows the tables of `docs/architecture/Keymap.md`: tools, style,
/// edit, view.
pub const BINDINGS: &[(KeyChord, Command, &str)] = &[
    // Tools
    (KeyChord::bare(Key::P), Command::SetTool(Tool::Pen), "Pen"),
    (KeyChord::bare(Key::L), Command::SetTool(Tool::Line), "Line"),
    (KeyChord::bare(Key::A), Command::SetTool(Tool::Arrow), "Arrow"),
    (KeyChord::bare(Key::R), Command::SetTool(Tool::Rect), "Rectangle"),
    (
        KeyChord::bare(Key::C),
        Command::SetTool(Tool::Ellipse),
        "Circle / ellipse",
    ),
    (KeyChord::bare(Key::E), Command::SetTool(Tool::Eraser), "Eraser"),
    (KeyChord::bare(Key::B), Command::SetTool(Tool::Bucket), "Bucket fill"),
    (
        KeyChord::bare(Key::V),
        Command::SetTool(Tool::Select),
        "Select / move",
    ),
    (KeyChord::bare(Key::H), Command::SetTool(Tool::Hand), "Hand (pan)"),
    // Style
    (KeyChord::bare(Key::Digit1), color(1), "Colour 1 (ink)"),
    (KeyChord::bare(Key::Digit2), color(2), "Colour 2 (red)"),
    (KeyChord::bare(Key::Digit3), color(3), "Colour 3 (green)"),
    (KeyChord::bare(Key::Digit4), color(4), "Colour 4 (blue)"),
    (KeyChord::bare(Key::Digit5), color(5), "Colour 5 (yellow)"),
    (KeyChord::bare(Key::Digit6), color(6), "Colour 6 (magenta)"),
    (
        KeyChord::bare(Key::BracketLeft),
        Command::WidthDown,
        "Thinner stroke",
    ),
    (
        KeyChord::bare(Key::BracketRight),
        Command::WidthUp,
        "Thicker stroke",
    ),
    (
        KeyChord::bare(Key::S),
        Command::CycleSmoothing,
        "Cycle anti-tremor strength",
    ),
    // Edit
    (KeyChord::ctrl(Key::Z), Command::Undo, "Undo"),
    (KeyChord::ctrl_shift(Key::Z), Command::Redo, "Redo"),
    (KeyChord::ctrl(Key::Y), Command::Redo, "Redo"),
    (KeyChord::ctrl(Key::C), Command::Copy, "Copy"),
    (KeyChord::ctrl(Key::X), Command::Cut, "Cut"),
    (KeyChord::ctrl(Key::V), Command::Paste, "Paste at cursor"),
    (
        KeyChord::ctrl(Key::D),
        Command::Duplicate,
        "Duplicate selection",
    ),
    (KeyChord::ctrl(Key::A), Command::SelectAll, "Select all"),
    (
        KeyChord::bare(Key::Delete),
        Command::DeleteSelection,
        "Delete selection",
    ),
    (
        KeyChord::bare(Key::Backspace),
        Command::DeleteSelection,
        "Delete selection",
    ),
    (
        KeyChord::ctrl(Key::Backspace),
        Command::ClearAll,
        "Clear canvas",
    ),
    (
        KeyChord::bare(Key::Escape),
        Command::Cancel,
        "Cancel gesture / clear selection",
    ),
    // View
    (KeyChord::bare(Key::Digit0), Command::ResetView, "Reset view"),
    (KeyChord::bare(Key::F), Command::FitView, "Fit view to content"),
    (
        KeyChord::bare(Key::Tab),
        Command::ToggleToolbar,
        "Show / hide toolbar",
    ),
];

/// The command bound to `key` pressed with `mods`, if any.
///
/// Modifiers must match exactly: `Shift+R` resolves to nothing rather than to
/// the rectangle tool.
#[must_use]
pub fn resolve(key: Key, mods: Modifiers) -> Option<Command> {
    let chord = KeyChord { key, mods };
    BINDINGS
        .iter()
        .find(|&&(bound, _, _)| bound == chord)
        .map(|&(_, command, _)| command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const NONE: Modifiers = Modifiers::NONE;
    const CTRL: Modifiers = Modifiers {
        shift: false,
        ctrl: true,
        alt: false,
    };
    const CTRL_SHIFT: Modifiers = Modifiers {
        shift: true,
        ctrl: true,
        alt: false,
    };
    const SHIFT: Modifiers = Modifiers {
        shift: true,
        ctrl: false,
        alt: false,
    };
    const ALT: Modifiers = Modifiers {
        shift: false,
        ctrl: false,
        alt: true,
    };

    const ALL_KEYS: [Key; 43] = [
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
        Key::Digit0,
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Digit4,
        Key::Digit5,
        Key::Digit6,
        Key::Digit7,
        Key::Digit8,
        Key::Digit9,
        Key::Delete,
        Key::Backspace,
        Key::Escape,
        Key::Space,
        Key::Tab,
        Key::BracketLeft,
        Key::BracketRight,
    ];

    fn color(digit: u8) -> Command {
        match ColorId::from_key_digit(digit) {
            Some(id) => Command::SetColor(id),
            None => panic!("digit {digit} has no colour"),
        }
    }

    /// The documented table of `docs/architecture/Keymap.md`, written out
    /// independently of `BINDINGS`.
    fn documented() -> Vec<(Key, Modifiers, Command)> {
        vec![
            // Tools
            (Key::P, NONE, Command::SetTool(Tool::Pen)),
            (Key::L, NONE, Command::SetTool(Tool::Line)),
            (Key::A, NONE, Command::SetTool(Tool::Arrow)),
            (Key::R, NONE, Command::SetTool(Tool::Rect)),
            (Key::C, NONE, Command::SetTool(Tool::Ellipse)),
            (Key::E, NONE, Command::SetTool(Tool::Eraser)),
            (Key::B, NONE, Command::SetTool(Tool::Bucket)),
            (Key::V, NONE, Command::SetTool(Tool::Select)),
            (Key::H, NONE, Command::SetTool(Tool::Hand)),
            // Style
            (Key::Digit1, NONE, color(1)),
            (Key::Digit2, NONE, color(2)),
            (Key::Digit3, NONE, color(3)),
            (Key::Digit4, NONE, color(4)),
            (Key::Digit5, NONE, color(5)),
            (Key::Digit6, NONE, color(6)),
            (Key::BracketLeft, NONE, Command::WidthDown),
            (Key::BracketRight, NONE, Command::WidthUp),
            (Key::S, NONE, Command::CycleSmoothing),
            // Edit
            (Key::Z, CTRL, Command::Undo),
            (Key::Z, CTRL_SHIFT, Command::Redo),
            (Key::Y, CTRL, Command::Redo),
            (Key::C, CTRL, Command::Copy),
            (Key::X, CTRL, Command::Cut),
            (Key::V, CTRL, Command::Paste),
            (Key::D, CTRL, Command::Duplicate),
            (Key::A, CTRL, Command::SelectAll),
            (Key::Delete, NONE, Command::DeleteSelection),
            (Key::Backspace, NONE, Command::DeleteSelection),
            (Key::Backspace, CTRL, Command::ClearAll),
            (Key::Escape, NONE, Command::Cancel),
            // View
            (Key::Digit0, NONE, Command::ResetView),
            (Key::F, NONE, Command::FitView),
            (Key::Tab, NONE, Command::ToggleToolbar),
        ]
    }

    fn documented_lookup(key: Key, mods: Modifiers) -> Option<Command> {
        documented()
            .into_iter()
            .find(|&(k, m, _)| k == key && m == mods)
            .map(|(_, _, c)| c)
    }

    fn all_mods() -> impl Iterator<Item = Modifiers> {
        (0u8..8).map(|bits| Modifiers {
            shift: bits & 1 != 0,
            ctrl: bits & 2 != 0,
            alt: bits & 4 != 0,
        })
    }

    #[test]
    fn every_documented_binding() {
        for (key, mods, expected) in documented() {
            // Act
            let got = resolve(key, mods);

            // Assert
            assert_eq!(got, Some(expected), "{key:?} {mods:?}");
        }
    }

    #[test]
    fn modifiers_disambiguate() {
        // Modified bindings differ from the bare key.
        assert_eq!(resolve(Key::C, CTRL), Some(Command::Copy));
        assert_eq!(resolve(Key::C, NONE), Some(Command::SetTool(Tool::Ellipse)));
        assert_eq!(resolve(Key::Backspace, CTRL), Some(Command::ClearAll));
        assert_eq!(resolve(Key::Backspace, NONE), Some(Command::DeleteSelection));
        assert_eq!(resolve(Key::Z, CTRL), Some(Command::Undo));
        assert_eq!(resolve(Key::Z, CTRL_SHIFT), Some(Command::Redo));
        assert_eq!(resolve(Key::V, CTRL), Some(Command::Paste));
        assert_eq!(resolve(Key::A, CTRL), Some(Command::SelectAll));

        // Extra modifiers on bare keys resolve to nothing.
        assert_eq!(resolve(Key::R, SHIFT), None);
        assert_eq!(resolve(Key::Digit1, ALT), None);
        assert_eq!(resolve(Key::Z, NONE), None);
        assert_eq!(resolve(Key::C, CTRL_SHIFT), None);
    }

    #[test]
    fn resolve_matches_documented_table_for_all_chords() {
        for key in ALL_KEYS {
            for mods in all_mods() {
                assert_eq!(
                    resolve(key, mods),
                    documented_lookup(key, mods),
                    "{key:?} {mods:?}"
                );
            }
        }
    }

    #[test]
    fn space_is_not_bound() {
        for mods in all_mods() {
            assert_eq!(resolve(Key::Space, mods), None);
        }
    }

    #[test]
    fn bindings_have_descriptions() {
        for (chord, command, description) in BINDINGS {
            assert!(
                !description.trim().is_empty(),
                "{chord:?} -> {command:?} has no description"
            );
        }
    }

    #[test]
    fn bindings_have_unique_chords() {
        // Arrange
        let chords: std::collections::HashSet<KeyChord> =
            BINDINGS.iter().map(|&(chord, _, _)| chord).collect();

        // Assert
        assert_eq!(chords.len(), BINDINGS.len());
        assert_eq!(BINDINGS.len(), documented().len());
    }

    #[test]
    fn resolve_agrees_with_bindings() {
        for &(chord, command, _) in BINDINGS {
            assert_eq!(resolve(chord.key, chord.mods), Some(command), "{chord:?}");
        }
    }

    proptest! {
        #[test]
        fn unbound_keys_resolve_to_none(
            index in 0..ALL_KEYS.len(),
            shift: bool,
            ctrl: bool,
            alt: bool,
        ) {
            // Arrange
            let key = ALL_KEYS[index];
            let mods = Modifiers { shift, ctrl, alt };
            prop_assume!(documented_lookup(key, mods).is_none());

            // Act / Assert
            prop_assert_eq!(resolve(key, mods), None);
        }
    }
}
