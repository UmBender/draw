//! The command palette: a prompt that lists the keymap's commands by
//! description, with their keys (ADR-T111-1).
//!
//! The palette is plain state: the editor holds an `Option` of it and sends
//! it the keys while it is open. Nothing here is saved or recorded in the
//! history.

use std::ops::Range;

use crate::core::command::Command;
use crate::core::input::{Key, Modifiers};
use crate::core::keymap::{BINDINGS, KeyChord};

/// Longest query in characters; further typing is dropped.
pub const MAX_QUERY_CHARS: usize = 64;
/// Rows the shell shows at once.
pub const VISIBLE_ROWS: usize = 10;

/// One palette row: a keymap binding.
pub type Row = (KeyChord, Command, &'static str);

/// The indices of `descriptions` that contain the letters of `query` in
/// order (ignoring case), best match first. An empty query keeps every
/// index. Ties keep the order of `descriptions`.
#[must_use]
pub fn rank(query: &str, descriptions: &[&str]) -> Vec<usize> {
    todo!("rank {query} {descriptions:?}")
}

/// How a chord is written in the palette: `Ctrl+Shift+]`, `Alt+←`, `D`.
#[must_use]
pub fn chord_label(chord: KeyChord) -> String {
    todo!("label {chord:?}")
}

/// The rows to show: at most `rows` of `total`, a window that keeps
/// `selected` inside.
#[must_use]
pub fn visible_window(selected: usize, total: usize, rows: usize) -> Range<usize> {
    todo!("window {selected} {total} {rows}")
}

/// The open palette: the query, the matching rows and the highlight.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandPalette {
    /// What was typed.
    query: String,
    /// Indices into [`BINDINGS`] of the matching rows, best first.
    matches: Vec<usize>,
    /// Index into `matches` of the highlighted row.
    selected: usize,
}

impl CommandPalette {
    /// A palette with an empty query: every binding, the first highlighted.
    #[must_use]
    pub fn new() -> Self {
        todo!("new")
    }

    /// What was typed.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Index of the highlighted row among [`CommandPalette::rows`].
    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Number of matching rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.matches.len()
    }

    /// Whether no row matches.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    /// The matching rows, best first.
    pub fn rows(&self) -> impl Iterator<Item = Row> + '_ {
        self.matches.iter().filter_map(|&i| BINDINGS.get(i).copied())
    }

    /// The highlighted row, if any matches.
    #[must_use]
    pub fn highlighted(&self) -> Option<Row> {
        self.matches
            .get(self.selected)
            .and_then(|&i| BINDINGS.get(i).copied())
    }

    /// Types `ch` at the end of the query. Control characters and input
    /// past [`MAX_QUERY_CHARS`] are dropped. Returns whether it changed.
    pub fn insert(&mut self, ch: char) -> bool {
        todo!("insert {ch}")
    }

    /// Deletes the last character of the query. Returns whether it changed.
    pub fn backspace(&mut self) -> bool {
        todo!("backspace")
    }

    /// Highlights the previous row. Returns whether it moved.
    pub fn up(&mut self) -> bool {
        todo!("up")
    }

    /// Highlights the next row. Returns whether it moved.
    pub fn down(&mut self) -> bool {
        todo!("down")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command::Tool;

    #[test]
    fn rank_matches_letters_in_order_ignoring_case() {
        // Arrange
        let rows = ["Rectangle", "Circle / ellipse", "Redo"];

        // Act / Assert
        assert_eq!(rank("RCT", &rows), [0]);
        assert_eq!(rank("cle", &rows), [1]);
        assert_eq!(rank("", &rows), [0, 1, 2]);
        assert_eq!(rank("é", &["Été"]), [0]);
    }

    #[test]
    fn rank_rejects_letters_out_of_order() {
        // Arrange
        let rows = ["Rectangle", "Redo"];

        // Act / Assert
        assert!(rank("tcr", &rows).is_empty());
        assert!(rank("redoo", &rows).is_empty());
        assert!(rank("x", &rows).is_empty());
    }

    #[test]
    fn rank_prefers_contiguous_and_word_start_matches() {
        // Arrange
        let rows = ["a-b-c", "abc"];
        let words = ["Bucket fill", "Fill"];
        let spread = ["Zoom fit", "Grid: one less row"];

        // Act / Assert
        assert_eq!(rank("abc", &rows), [1, 0]);
        assert_eq!(rank("fi", &words), [1, 0]);
        assert_eq!(rank("zf", &spread), [0]);
        assert_eq!(rank("gr", &["Colour grey", "Grid"]), [1, 0]);
    }

    #[test]
    fn rank_ties_keep_keymap_order() {
        // Arrange
        let rows = ["Redo", "Redo", "Redo"];

        // Act / Assert
        assert_eq!(rank("re", &rows), [0, 1, 2]);
    }

    #[test]
    fn chord_label_spells_modifiers_and_keys() {
        // Arrange / Act / Assert
        assert_eq!(chord_label(KeyChord::bare(Key::D)), "D");
        assert_eq!(chord_label(KeyChord::bare(Key::Digit3)), "3");
        assert_eq!(chord_label(KeyChord::ctrl(Key::Z)), "Ctrl+Z");
        assert_eq!(chord_label(KeyChord::ctrl_shift(Key::BracketRight)), "Ctrl+Shift+]");
        assert_eq!(chord_label(KeyChord::alt(Key::ArrowLeft)), "Alt+←");
        assert_eq!(chord_label(KeyChord::shift(Key::Tab)), "Shift+Tab");
        assert_eq!(chord_label(KeyChord::bare(Key::Backspace)), "Backspace");
        assert_eq!(chord_label(KeyChord::bare(Key::Delete)), "Del");
        assert_eq!(chord_label(KeyChord::bare(Key::Escape)), "Esc");
        let all = Modifiers {
            shift: true,
            ctrl: true,
            alt: true,
        };
        assert_eq!(
            chord_label(KeyChord {
                key: Key::P,
                mods: all
            }),
            "Ctrl+Alt+Shift+P"
        );
    }

    #[test]
    fn chord_label_is_never_empty_for_a_binding() {
        for &(chord, _, _) in BINDINGS {
            assert!(!chord_label(chord).is_empty(), "{chord:?}");
        }
    }

    #[test]
    fn rows_are_exactly_the_keymap_bindings() {
        // Arrange / Act
        let palette = CommandPalette::new();
        let rows: Vec<_> = palette.rows().collect();

        // Assert: every binding once, in table order, nothing else.
        assert_eq!(rows, BINDINGS.to_vec());
        assert_eq!(palette.len(), BINDINGS.len());
        assert_eq!(palette.highlighted(), BINDINGS.first().copied());
    }

    #[test]
    fn palette_filters_by_description_and_resets_the_highlight() {
        // Arrange
        let mut palette = CommandPalette::new();
        palette.down();
        palette.down();

        // Act
        assert!(palette.insert('r'));
        assert!(palette.insert('e'));
        assert!(palette.insert('c'));

        // Assert
        assert_eq!(palette.query(), "rec");
        assert_eq!(palette.selected(), 0);
        let found = palette.highlighted();
        assert_eq!(
            found.map(|(_, command, _)| command),
            Some(Command::SetTool(Tool::Rect))
        );
        assert!(palette.rows().all(|(_, _, d)| d.to_lowercase().contains('r')));
    }

    #[test]
    fn palette_query_is_bounded_and_drops_control_characters() {
        // Arrange
        let mut palette = CommandPalette::new();

        // Act
        assert!(!palette.insert('\n'));
        assert!(!palette.insert('\u{7}'));
        for _ in 0..MAX_QUERY_CHARS + 10 {
            palette.insert('a');
        }

        // Assert
        assert_eq!(palette.query().chars().count(), MAX_QUERY_CHARS);
        assert!(!palette.insert('b'));
    }

    #[test]
    fn palette_backspace_on_an_empty_query_does_nothing() {
        let mut palette = CommandPalette::new();
        assert!(!palette.backspace());
        palette.insert('x');
        assert!(palette.backspace());
        assert_eq!(palette.query(), "");
    }

    #[test]
    fn palette_moves_clamp_on_an_empty_list() {
        // Arrange
        let mut palette = CommandPalette::new();
        for ch in "qqqqqq".chars() {
            palette.insert(ch);
        }

        // Act / Assert
        assert_eq!(palette.len(), 0);
        assert!(!palette.down());
        assert!(!palette.up());
        assert_eq!(palette.highlighted(), None);
    }

    #[test]
    fn visible_window_follows_the_highlight() {
        // Arrange / Act / Assert
        assert_eq!(visible_window(0, 5, 10), 0..5);
        assert_eq!(visible_window(0, 40, 10), 0..10);
        assert_eq!(visible_window(9, 40, 10), 0..10);
        assert_eq!(visible_window(10, 40, 10), 1..11);
        assert_eq!(visible_window(39, 40, 10), 30..40);
        assert_eq!(visible_window(0, 0, 10), 0..0);
        assert_eq!(visible_window(7, 3, 10), 0..3);
    }
}
