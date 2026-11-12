//! Key chords mapped to editor commands.
//!
//! [`BINDINGS`] is the single table of the bindings documented in
//! `docs/architecture/Keymap.md`; the toolbar and docs can list it, and
//! [`resolve`] looks chords up in it. A binding fires only when the held
//! modifiers match the chord's modifiers exactly (ADR-T11-1). `Space` is not
//! bound: the editor consumes it as the pan modifier. While a text block is
//! being edited no binding fires (ADR-T29-2).

use crate::core::command::{Axis, Command, Side, Tool};
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

    /// `Shift` + `key`.
    #[must_use]
    pub const fn shift(key: Key) -> Self {
        Self {
            key,
            mods: Modifiers {
                shift: true,
                ctrl: false,
                alt: false,
            },
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

    /// `Alt` + `key`.
    #[must_use]
    pub const fn alt(key: Key) -> Self {
        Self {
            key,
            mods: Modifiers {
                shift: false,
                ctrl: false,
                alt: true,
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
/// edit, view, helpers (ADR-T16-2).
pub const BINDINGS: &[(KeyChord, Command, &str)] = &[
    // Tools
    (KeyChord::bare(Key::D), Command::SetTool(Tool::Pen), "Pen"),
    (KeyChord::bare(Key::W), Command::SetTool(Tool::Line), "Line"),
    (
        KeyChord::bare(Key::A),
        Command::SetTool(Tool::Arrow),
        "Arrow",
    ),
    (
        KeyChord::bare(Key::R),
        Command::SetTool(Tool::Rect),
        "Rectangle",
    ),
    (
        KeyChord::bare(Key::C),
        Command::SetTool(Tool::Ellipse),
        "Circle / ellipse",
    ),
    (
        KeyChord::bare(Key::E),
        Command::SetTool(Tool::Eraser),
        "Eraser",
    ),
    (
        KeyChord::bare(Key::B),
        Command::SetTool(Tool::Bucket),
        "Bucket fill",
    ),
    (
        KeyChord::bare(Key::V),
        Command::SetTool(Tool::Select),
        "Select / move",
    ),
    (
        KeyChord::bare(Key::H),
        Command::SetTool(Tool::Hand),
        "Hand (pan)",
    ),
    (KeyChord::bare(Key::G), Command::SetTool(Tool::Grid), "Grid"),
    (KeyChord::bare(Key::T), Command::SetTool(Tool::Text), "Text"),
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
        KeyChord::ctrl(Key::T),
        Command::GridFromText,
        "Text block to grid",
    ),
    (
        KeyChord::ctrl(Key::L),
        Command::GraphFromText,
        "Text block to graph",
    ),
    (
        KeyChord::alt(Key::BracketRight),
        Command::ScaleUp,
        "Grow selection 10 %",
    ),
    (
        KeyChord::alt(Key::BracketLeft),
        Command::ScaleDown,
        "Shrink selection 10 %",
    ),
    (
        KeyChord::ctrl(Key::P),
        Command::PlotFromText,
        "Text block to plot",
    ),
    (
        KeyChord::ctrl(Key::BracketRight),
        Command::RaiseSelection,
        "Raise selection one step",
    ),
    (
        KeyChord::ctrl(Key::BracketLeft),
        Command::LowerSelection,
        "Lower selection one step",
    ),
    (
        KeyChord::ctrl_shift(Key::BracketRight),
        Command::SelectionToTop,
        "Selection to the top",
    ),
    (
        KeyChord::ctrl_shift(Key::BracketLeft),
        Command::SelectionToBottom,
        "Selection to the bottom",
    ),
    (
        KeyChord::bare(Key::Z),
        Command::RecognizeShape,
        "Last stroke to line / rectangle / ellipse",
    ),
    (
        KeyChord::alt(Key::ArrowLeft),
        Command::Align(Side::Left),
        "Align left edges",
    ),
    (
        KeyChord::alt(Key::ArrowRight),
        Command::Align(Side::Right),
        "Align right edges",
    ),
    (
        KeyChord::alt(Key::ArrowUp),
        Command::Align(Side::Top),
        "Align top edges",
    ),
    (
        KeyChord::alt(Key::ArrowDown),
        Command::Align(Side::Bottom),
        "Align bottom edges",
    ),
    (
        KeyChord::alt(Key::H),
        Command::Distribute(Axis::Horizontal),
        "Lay out as a row, no overlap",
    ),
    (
        KeyChord::alt(Key::V),
        Command::Distribute(Axis::Vertical),
        "Lay out as a column, no overlap",
    ),
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
        KeyChord::ctrl(Key::K),
        Command::LockSelection,
        "Lock selection",
    ),
    (
        KeyChord::ctrl_shift(Key::K),
        Command::UnlockAll,
        "Unlock every shape",
    ),
    (
        KeyChord::bare(Key::Escape),
        Command::Cancel,
        "Cancel gesture / clear selection",
    ),
    // View
    (
        KeyChord::bare(Key::Digit0),
        Command::ResetView,
        "Reset view",
    ),
    (
        KeyChord::bare(Key::F),
        Command::FitView,
        "Fit view to content",
    ),
    (
        KeyChord::bare(Key::Tab),
        Command::ToggleToolbar,
        "Show / hide toolbar",
    ),
    (
        KeyChord::ctrl(Key::E),
        Command::ExportView,
        "Export the view to PNG",
    ),
    (
        KeyChord::ctrl(Key::S),
        Command::SaveFile,
        "Save the drawing to a .draw file",
    ),
    (
        KeyChord::ctrl_shift(Key::E),
        Command::ExportSvg,
        "Export the drawing to SVG",
    ),
    (
        KeyChord::ctrl_shift(Key::C),
        Command::CopyImage,
        "Copy the selection as an image",
    ),
    (KeyChord::alt(Key::Digit1), Command::GoToPage(0), "Page 1"),
    (KeyChord::alt(Key::Digit2), Command::GoToPage(1), "Page 2"),
    (KeyChord::alt(Key::Digit3), Command::GoToPage(2), "Page 3"),
    (KeyChord::alt(Key::Digit4), Command::GoToPage(3), "Page 4"),
    (KeyChord::alt(Key::Digit5), Command::GoToPage(4), "Page 5"),
    (KeyChord::alt(Key::Digit6), Command::GoToPage(5), "Page 6"),
    (KeyChord::alt(Key::Digit7), Command::GoToPage(6), "Page 7"),
    (KeyChord::alt(Key::Digit8), Command::GoToPage(7), "Page 8"),
    (KeyChord::alt(Key::Digit9), Command::GoToPage(8), "Page 9"),
    // Helpers
    (
        KeyChord::bare(Key::X),
        Command::ToggleSmartSnap,
        "Smart snap on / off",
    ),
    (
        KeyChord::shift(Key::G),
        Command::ToggleGridSnap,
        "Grid snap on / off",
    ),
    (
        KeyChord::bare(Key::N),
        Command::ToggleNumbering,
        "Numbering on / off",
    ),
    (
        KeyChord::shift(Key::N),
        Command::ResetNumbering,
        "Restart numbering at the first number",
    ),
    (
        KeyChord::ctrl(Key::N),
        Command::ToggleZeroBased,
        "First number 1 / 0",
    ),
    (
        KeyChord::bare(Key::ArrowRight),
        Command::GridCols(1),
        "Grid: one more column",
    ),
    (
        KeyChord::bare(Key::ArrowLeft),
        Command::GridCols(-1),
        "Grid: one less column",
    ),
    (
        KeyChord::bare(Key::ArrowDown),
        Command::GridRows(1),
        "Grid: one more row",
    ),
    (
        KeyChord::bare(Key::ArrowUp),
        Command::GridRows(-1),
        "Grid: one less row",
    ),
];

/// A page of the shortcut sheet (ADR-T113-1), in the order of
/// `docs/features/Contest cheat sheet.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topic {
    /// Tool keys.
    Tools,
    /// Colours, stroke width, smoothing.
    Style,
    /// Undo, clipboard, selection, arrangement.
    Edit,
    /// Zoom, pan, fit, toolbar, export.
    View,
    /// Snapping, numbering, grid size.
    Helpers,
    /// Mouse gestures and modifiers (no key binding).
    Gestures,
    /// Keys inside a text block (no key binding).
    TypingText,
    /// Keys inside a shape or grid cell (no key binding).
    TypingShape,
}

impl Topic {
    /// Every topic, one sheet page group each.
    pub const ALL: [Self; 8] = [
        Self::Tools,
        Self::Style,
        Self::Edit,
        Self::View,
        Self::Helpers,
        Self::Gestures,
        Self::TypingText,
        Self::TypingShape,
    ];

    /// The name shown on the sheet.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Tools => "Tools",
            Self::Style => "Style",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Helpers => "Helpers",
            Self::Gestures => "Gestures",
            Self::TypingText => "Typing in a text block",
            Self::TypingShape => "Typing in a shape",
        }
    }
}

/// The sheet topic of `command`. An exhaustive `match`: a new command cannot
/// be added without choosing its topic (ADR-T113-1).
#[must_use]
pub fn topic_of(command: Command) -> Topic {
    match command {
        Command::SetTool(_) => Topic::Tools,
        Command::SetColor(_) | Command::WidthUp | Command::WidthDown | Command::CycleSmoothing => {
            Topic::Style
        }
        Command::Undo
        | Command::Redo
        | Command::Copy
        | Command::Cut
        | Command::Paste
        | Command::Duplicate
        | Command::SelectAll
        | Command::GridFromText
        | Command::GraphFromText
        | Command::PlotFromText
        | Command::ScaleUp
        | Command::ScaleDown
        | Command::RaiseSelection
        | Command::LowerSelection
        | Command::SelectionToTop
        | Command::SelectionToBottom
        | Command::Align(_)
        | Command::Distribute(_)
        | Command::RecognizeShape
        | Command::LockSelection
        | Command::UnlockAll
        | Command::DeleteSelection
        | Command::ClearAll
        | Command::Cancel => Topic::Edit,
        Command::ResetView
        | Command::FitView
        | Command::GoToPage(_)
        | Command::ToggleToolbar
        | Command::ExportView
        | Command::ExportSvg
        | Command::CopyImage
        | Command::SaveFile => Topic::View,
        Command::ToggleSmartSnap
        | Command::ToggleGridSnap
        | Command::ToggleNumbering
        | Command::ResetNumbering
        | Command::ToggleZeroBased
        | Command::GridCols(_)
        | Command::GridRows(_) => Topic::Helpers,
    }
}

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
    use crate::core::command::{Axis, Side};
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

    const ALL_KEYS: [Key; 50] = [
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
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::ArrowDown,
        Key::Enter,
        Key::Home,
        Key::End,
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
            // Pages
            (Key::Digit1, ALT, Command::GoToPage(0)),
            (Key::Digit2, ALT, Command::GoToPage(1)),
            (Key::Digit3, ALT, Command::GoToPage(2)),
            (Key::Digit4, ALT, Command::GoToPage(3)),
            (Key::Digit5, ALT, Command::GoToPage(4)),
            (Key::Digit6, ALT, Command::GoToPage(5)),
            (Key::Digit7, ALT, Command::GoToPage(6)),
            (Key::Digit8, ALT, Command::GoToPage(7)),
            (Key::Digit9, ALT, Command::GoToPage(8)),
            // Tools
            (Key::D, NONE, Command::SetTool(Tool::Pen)),
            (Key::W, NONE, Command::SetTool(Tool::Line)),
            (Key::A, NONE, Command::SetTool(Tool::Arrow)),
            (Key::R, NONE, Command::SetTool(Tool::Rect)),
            (Key::C, NONE, Command::SetTool(Tool::Ellipse)),
            (Key::E, NONE, Command::SetTool(Tool::Eraser)),
            (Key::B, NONE, Command::SetTool(Tool::Bucket)),
            (Key::V, NONE, Command::SetTool(Tool::Select)),
            (Key::H, NONE, Command::SetTool(Tool::Hand)),
            (Key::G, NONE, Command::SetTool(Tool::Grid)),
            (Key::T, NONE, Command::SetTool(Tool::Text)),
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
            (Key::T, CTRL, Command::GridFromText),
            (Key::L, CTRL, Command::GraphFromText),
            (Key::BracketRight, ALT, Command::ScaleUp),
            (Key::BracketLeft, ALT, Command::ScaleDown),
            (Key::P, CTRL, Command::PlotFromText),
            (Key::BracketRight, CTRL, Command::RaiseSelection),
            (Key::BracketLeft, CTRL, Command::LowerSelection),
            (Key::BracketRight, CTRL_SHIFT, Command::SelectionToTop),
            (Key::BracketLeft, CTRL_SHIFT, Command::SelectionToBottom),
            (Key::Z, NONE, Command::RecognizeShape),
            (Key::ArrowLeft, ALT, Command::Align(Side::Left)),
            (Key::ArrowRight, ALT, Command::Align(Side::Right)),
            (Key::ArrowUp, ALT, Command::Align(Side::Top)),
            (Key::ArrowDown, ALT, Command::Align(Side::Bottom)),
            (Key::H, ALT, Command::Distribute(Axis::Horizontal)),
            (Key::V, ALT, Command::Distribute(Axis::Vertical)),
            (Key::Delete, NONE, Command::DeleteSelection),
            (Key::Backspace, NONE, Command::DeleteSelection),
            (Key::Backspace, CTRL, Command::ClearAll),
            (Key::K, CTRL, Command::LockSelection),
            (Key::K, CTRL_SHIFT, Command::UnlockAll),
            (Key::Escape, NONE, Command::Cancel),
            // View
            (Key::Digit0, NONE, Command::ResetView),
            (Key::F, NONE, Command::FitView),
            (Key::Tab, NONE, Command::ToggleToolbar),
            (Key::E, CTRL, Command::ExportView),
            (Key::S, CTRL, Command::SaveFile),
            (Key::E, CTRL_SHIFT, Command::ExportSvg),
            (Key::C, CTRL_SHIFT, Command::CopyImage),
            // Helpers
            (Key::X, NONE, Command::ToggleSmartSnap),
            (Key::G, SHIFT, Command::ToggleGridSnap),
            (Key::N, NONE, Command::ToggleNumbering),
            (Key::N, SHIFT, Command::ResetNumbering),
            (Key::N, CTRL, Command::ToggleZeroBased),
            (Key::ArrowRight, NONE, Command::GridCols(1)),
            (Key::ArrowLeft, NONE, Command::GridCols(-1)),
            (Key::ArrowDown, NONE, Command::GridRows(1)),
            (Key::ArrowUp, NONE, Command::GridRows(-1)),
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
        assert_eq!(
            resolve(Key::Backspace, NONE),
            Some(Command::DeleteSelection)
        );
        assert_eq!(resolve(Key::Z, CTRL), Some(Command::Undo));
        assert_eq!(resolve(Key::Z, CTRL_SHIFT), Some(Command::Redo));
        assert_eq!(resolve(Key::V, CTRL), Some(Command::Paste));
        assert_eq!(resolve(Key::A, CTRL), Some(Command::SelectAll));

        // Extra modifiers on bare keys resolve to nothing.
        assert_eq!(resolve(Key::R, SHIFT), None);
        assert_eq!(resolve(Key::Digit1, SHIFT), None);
        assert_eq!(resolve(Key::Z, SHIFT), None);
    }

    #[test]
    fn resolve_ctrl_shift_c_copies_the_selection_as_an_image() {
        assert_eq!(resolve(Key::C, CTRL_SHIFT), Some(Command::CopyImage));
        assert_eq!(resolve(Key::C, CTRL), Some(Command::Copy));
    }

    #[test]
    fn resolve_ctrl_s_saves() {
        assert_eq!(resolve(Key::S, CTRL), Some(Command::SaveFile));
        assert_eq!(resolve(Key::S, NONE), Some(Command::CycleSmoothing));
        assert_eq!(resolve(Key::S, CTRL_SHIFT), None);
    }

    #[test]
    fn resolve_ctrl_e_exports_the_view() {
        assert_eq!(resolve(Key::E, CTRL), Some(Command::ExportView));
        assert_eq!(resolve(Key::E, NONE), Some(Command::SetTool(Tool::Eraser)));
    }

    #[test]
    fn resolve_ctrl_shift_e_exports_svg() {
        assert_eq!(resolve(Key::E, CTRL_SHIFT), Some(Command::ExportSvg));
        assert_eq!(resolve(Key::E, SHIFT), None);
    }

    #[test]
    fn helper_bindings() {
        assert_eq!(resolve(Key::G, NONE), Some(Command::SetTool(Tool::Grid)));
        assert_eq!(resolve(Key::X, NONE), Some(Command::ToggleSmartSnap));
        assert_eq!(resolve(Key::G, SHIFT), Some(Command::ToggleGridSnap));
        assert_eq!(resolve(Key::N, NONE), Some(Command::ToggleNumbering));
        assert_eq!(resolve(Key::N, SHIFT), Some(Command::ResetNumbering));
        assert_eq!(resolve(Key::ArrowRight, NONE), Some(Command::GridCols(1)));
        assert_eq!(resolve(Key::ArrowLeft, NONE), Some(Command::GridCols(-1)));
        assert_eq!(resolve(Key::ArrowDown, NONE), Some(Command::GridRows(1)));
        assert_eq!(resolve(Key::ArrowUp, NONE), Some(Command::GridRows(-1)));
    }

    #[test]
    fn alt_digits_go_to_pages() {
        let digits = [
            Key::Digit1,
            Key::Digit2,
            Key::Digit3,
            Key::Digit4,
            Key::Digit5,
            Key::Digit6,
            Key::Digit7,
            Key::Digit8,
            Key::Digit9,
        ];
        for (page, key) in digits.into_iter().enumerate() {
            assert_eq!(resolve(key, ALT), Some(Command::GoToPage(page)), "{key:?}");
        }
        // Alt+0 and the bare digits keep their meaning.
        assert_eq!(resolve(Key::Digit0, ALT), None);
        assert_eq!(resolve(Key::Digit0, NONE), Some(Command::ResetView));
        assert_eq!(resolve(Key::Digit1, NONE), Some(color(1)));
    }

    #[test]
    fn resolve_ctrl_k_locks_and_ctrl_shift_k_unlocks() {
        assert_eq!(resolve(Key::K, CTRL), Some(Command::LockSelection));
        assert_eq!(resolve(Key::K, CTRL_SHIFT), Some(Command::UnlockAll));
        assert_eq!(resolve(Key::K, NONE), None);
        assert_eq!(resolve(Key::K, SHIFT), None);
        assert_eq!(resolve(Key::K, ALT), None);
    }

    #[test]
    fn ctrl_n_switches_the_first_number() {
        assert_eq!(resolve(Key::N, CTRL), Some(Command::ToggleZeroBased));
        assert_eq!(resolve(Key::N, CTRL_SHIFT), None);
    }

    #[test]
    fn left_hand_keys_select_pen_line_and_smart_snap() {
        assert_eq!(resolve(Key::D, NONE), Some(Command::SetTool(Tool::Pen)));
        assert_eq!(resolve(Key::W, NONE), Some(Command::SetTool(Tool::Line)));
        assert_eq!(resolve(Key::X, NONE), Some(Command::ToggleSmartSnap));
        for key in [Key::D, Key::W, Key::X] {
            for mods in [SHIFT, ALT, CTRL_SHIFT] {
                assert_eq!(resolve(key, mods), None, "{key:?} {mods:?}");
            }
        }
    }

    #[test]
    fn left_hand_keys_keep_their_ctrl_chords() {
        assert_eq!(resolve(Key::D, CTRL), Some(Command::Duplicate));
        assert_eq!(resolve(Key::X, CTRL), Some(Command::Cut));
        assert_eq!(resolve(Key::W, CTRL), None);
    }

    #[test]
    fn p_l_m_are_unbound() {
        // T38: pen, line and smart snap moved to the left hand.
        for key in [Key::P, Key::L, Key::M] {
            for mods in all_mods() {
                if (key, mods) != (Key::L, CTRL) && (key, mods) != (Key::P, CTRL) {
                    assert_eq!(resolve(key, mods), None, "{key:?} {mods:?}");
                }
            }
        }
    }

    #[test]
    fn ctrl_p_builds_a_plot_from_text() {
        assert_eq!(resolve(Key::P, CTRL), Some(Command::PlotFromText));
        assert_eq!(resolve(Key::P, CTRL_SHIFT), None);
        assert_eq!(resolve(Key::P, NONE), None);
    }

    #[test]
    fn ctrl_brackets_restack_the_selection() {
        assert_eq!(
            resolve(Key::BracketRight, CTRL),
            Some(Command::RaiseSelection)
        );
        assert_eq!(
            resolve(Key::BracketLeft, CTRL),
            Some(Command::LowerSelection)
        );
        assert_eq!(
            resolve(Key::BracketRight, CTRL_SHIFT),
            Some(Command::SelectionToTop)
        );
        assert_eq!(
            resolve(Key::BracketLeft, CTRL_SHIFT),
            Some(Command::SelectionToBottom)
        );
        // The other bracket chords keep their meaning.
        assert_eq!(resolve(Key::BracketRight, NONE), Some(Command::WidthUp));
        assert_eq!(resolve(Key::BracketRight, ALT), Some(Command::ScaleUp));
        assert_eq!(resolve(Key::BracketLeft, SHIFT), None);
    }

    #[test]
    fn z_recognizes_the_last_stroke() {
        assert_eq!(resolve(Key::Z, NONE), Some(Command::RecognizeShape));
        // The modified chords keep their meaning.
        assert_eq!(resolve(Key::Z, CTRL), Some(Command::Undo));
        assert_eq!(resolve(Key::Z, CTRL_SHIFT), Some(Command::Redo));
        assert_eq!(resolve(Key::Z, SHIFT), None);
        assert_eq!(resolve(Key::Z, ALT), None);
    }

    #[test]
    fn alt_arrows_align_and_alt_h_v_distribute() {
        assert_eq!(
            resolve(Key::ArrowLeft, ALT),
            Some(Command::Align(Side::Left))
        );
        assert_eq!(
            resolve(Key::ArrowRight, ALT),
            Some(Command::Align(Side::Right))
        );
        assert_eq!(resolve(Key::ArrowUp, ALT), Some(Command::Align(Side::Top)));
        assert_eq!(
            resolve(Key::ArrowDown, ALT),
            Some(Command::Align(Side::Bottom))
        );
        assert_eq!(
            resolve(Key::H, ALT),
            Some(Command::Distribute(Axis::Horizontal))
        );
        assert_eq!(
            resolve(Key::V, ALT),
            Some(Command::Distribute(Axis::Vertical))
        );
        // The other chords of these keys keep their meaning.
        assert_eq!(resolve(Key::ArrowLeft, NONE), Some(Command::GridCols(-1)));
        assert_eq!(resolve(Key::H, NONE), Some(Command::SetTool(Tool::Hand)));
        assert_eq!(resolve(Key::V, CTRL), Some(Command::Paste));
        assert_eq!(resolve(Key::ArrowLeft, CTRL), None);
        assert_eq!(resolve(Key::H, CTRL), None);
    }

    #[test]
    fn ctrl_l_builds_a_graph_from_text() {
        assert_eq!(resolve(Key::L, CTRL), Some(Command::GraphFromText));
        assert_eq!(resolve(Key::L, CTRL_SHIFT), None, "the diagnostics dump");
    }

    #[test]
    fn t_selects_text_tool() {
        assert_eq!(resolve(Key::T, NONE), Some(Command::SetTool(Tool::Text)));
        assert_eq!(resolve(Key::T, SHIFT), None);
    }

    #[test]
    fn ctrl_t_builds_a_grid_from_text() {
        assert_eq!(resolve(Key::T, CTRL), Some(Command::GridFromText));
        assert_eq!(resolve(Key::T, CTRL_SHIFT), None);
    }

    #[test]
    fn ctrl_shift_v_is_left_to_the_shell() {
        // T84: the shell reads the clipboard, the keymap has no text.
        assert_eq!(resolve(Key::V, CTRL_SHIFT), None);
    }

    #[test]
    fn editing_keys_are_unbound() {
        // T29: Enter, Home and End act only inside a text block.
        for key in [Key::Enter, Key::Home, Key::End] {
            for mods in all_mods() {
                assert_eq!(resolve(key, mods), None, "{key:?} {mods:?}");
            }
        }
    }

    #[test]
    fn i_is_unbound() {
        // T22: grid indices follow numbering (`N`), no key of their own.
        for mods in all_mods() {
            assert_eq!(resolve(Key::I, mods), None, "{mods:?}");
        }
    }

    #[test]
    fn shift_chords_are_exact() {
        assert_eq!(KeyChord::shift(Key::G).mods, SHIFT);
        assert_eq!(resolve(Key::G, CTRL_SHIFT), None);
        assert_eq!(resolve(Key::N, ALT), None);
        assert_eq!(resolve(Key::X, SHIFT), None);
        assert_eq!(resolve(Key::ArrowLeft, SHIFT), None);
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
    fn q_is_not_bound() {
        // Held, `Q` is the laser pointer (ADR-T106-2); the shell routes it
        // before the keymap.
        for mods in all_mods() {
            assert_eq!(resolve(Key::Q, mods), None, "{mods:?}");
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
    fn topic_of_matches_the_sections_of_the_keymap() {
        // Arrange: the table is written section by section.
        let order = [
            Topic::Tools,
            Topic::Style,
            Topic::Edit,
            Topic::View,
            Topic::Helpers,
        ];
        let mut section = 0;

        // Act / Assert: topics never go back to an earlier section.
        for &(chord, command, _) in BINDINGS {
            let topic = topic_of(command);
            let at = order
                .iter()
                .position(|&t| t == topic)
                .unwrap_or_else(|| panic!("{chord:?} has the non-key topic {topic:?}"));
            assert!(at >= section, "{chord:?} -> {command:?} is out of order");
            section = at;
        }
        assert_eq!(section, order.len() - 1);
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
