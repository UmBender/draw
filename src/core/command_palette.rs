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
    let letters = lowered(query);
    let mut scored: Vec<(usize, i32)> = descriptions
        .iter()
        .enumerate()
        .filter_map(|(i, text)| score(&letters, &lowered(text)).map(|s| (i, s)))
        .collect();
    // The sort is stable: equal scores keep the keymap order.
    scored.sort_by_key(|&(_, s)| std::cmp::Reverse(s));
    scored.into_iter().map(|(i, _)| i).collect()
}

/// `text` as lower-case characters, one per character.
fn lowered(text: &str) -> Vec<char> {
    text.chars()
        .map(|c| c.to_lowercase().next().unwrap_or(c))
        .collect()
}

/// Points for one matched letter.
const LETTER: i32 = 1;
/// Extra points when it directly follows the previous matched letter.
const CONTIGUOUS: i32 = 5;
/// Extra points when it starts a word.
const WORD_START: i32 = 3;
/// Extra points when it is the first character of the text.
const TEXT_START: i32 = 2;

/// The best score of matching `query` as a subsequence of `text`, or
/// `None` when it does not match. An empty query scores `0`.
fn score(query: &[char], text: &[char]) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let start_bonus = |i: usize| {
        let word = i == 0 || text.get(i - 1).is_some_and(|c| !c.is_alphanumeric());
        let mut bonus = LETTER;
        if word {
            bonus += WORD_START;
        }
        if i == 0 {
            bonus += TEXT_START;
        }
        bonus
    };
    // `best[i]`: the best score with the letters so far ending at `text[i]`.
    let mut best: Vec<Option<i32>> = text
        .iter()
        .enumerate()
        .map(|(i, &c)| (Some(c) == query.first().copied()).then(|| start_bonus(i)))
        .collect();
    for &letter in query.iter().skip(1) {
        let next: Vec<Option<i32>> = text
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                if c != letter {
                    return None;
                }
                best.iter()
                    .take(i)
                    .enumerate()
                    .filter_map(|(k, prev)| {
                        prev.map(|p| p + start_bonus(i) + if k + 1 == i { CONTIGUOUS } else { 0 })
                    })
                    .max()
            })
            .collect();
        best = next;
    }
    best.into_iter().flatten().max()
}

/// How a chord is written in the palette: `Ctrl+Shift+]`, `Alt+←`, `D`.
#[must_use]
pub fn chord_label(chord: KeyChord) -> String {
    let Modifiers { shift, ctrl, alt } = chord.mods;
    let mut label = String::new();
    for (held, name) in [(ctrl, "Ctrl+"), (alt, "Alt+"), (shift, "Shift+")] {
        if held {
            label.push_str(name);
        }
    }
    label.push_str(key_label(chord.key));
    label
}

/// How a key is written.
fn key_label(key: Key) -> &'static str {
    match key {
        Key::A => "A",
        Key::B => "B",
        Key::C => "C",
        Key::D => "D",
        Key::E => "E",
        Key::F => "F",
        Key::G => "G",
        Key::H => "H",
        Key::I => "I",
        Key::J => "J",
        Key::K => "K",
        Key::L => "L",
        Key::M => "M",
        Key::N => "N",
        Key::O => "O",
        Key::P => "P",
        Key::Q => "Q",
        Key::R => "R",
        Key::S => "S",
        Key::T => "T",
        Key::U => "U",
        Key::V => "V",
        Key::W => "W",
        Key::X => "X",
        Key::Y => "Y",
        Key::Z => "Z",
        Key::Digit0 => "0",
        Key::Digit1 => "1",
        Key::Digit2 => "2",
        Key::Digit3 => "3",
        Key::Digit4 => "4",
        Key::Digit5 => "5",
        Key::Digit6 => "6",
        Key::Digit7 => "7",
        Key::Digit8 => "8",
        Key::Digit9 => "9",
        Key::Delete => "Del",
        Key::Backspace => "Backspace",
        Key::Escape => "Esc",
        Key::Space => "Space",
        Key::Tab => "Tab",
        Key::BracketLeft => "[",
        Key::BracketRight => "]",
        Key::ArrowLeft => "←",
        Key::ArrowRight => "→",
        Key::ArrowUp => "↑",
        Key::ArrowDown => "↓",
        Key::Enter => "Enter",
        Key::Home => "Home",
        Key::End => "End",
    }
}

/// The rows to show: at most `rows` of `total`, a window that keeps
/// `selected` inside.
#[must_use]
pub fn visible_window(selected: usize, total: usize, rows: usize) -> Range<usize> {
    let shown = rows.min(total);
    let start = (selected + 1).saturating_sub(shown).min(total - shown);
    start..start + shown
}

/// The open palette: the query, the matching rows and the highlight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPalette {
    /// What was typed.
    query: String,
    /// Indices into [`BINDINGS`] of the matching rows, best first.
    matches: Vec<usize>,
    /// Index into `matches` of the highlighted row.
    selected: usize,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    /// A palette with an empty query: every binding, the first highlighted.
    #[must_use]
    pub fn new() -> Self {
        let mut palette = Self {
            query: String::new(),
            matches: Vec::new(),
            selected: 0,
        };
        palette.refilter();
        palette
    }

    /// Recomputes the matching rows for the query and highlights the first.
    fn refilter(&mut self) {
        let descriptions: Vec<&str> = BINDINGS.iter().map(|&(_, _, d)| d).collect();
        self.matches = rank(&self.query, &descriptions);
        self.selected = 0;
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
        self.matches
            .iter()
            .filter_map(|&i| BINDINGS.get(i).copied())
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
        if ch.is_control() || self.query.chars().count() >= MAX_QUERY_CHARS {
            return false;
        }
        self.query.push(ch);
        self.refilter();
        true
    }

    /// Deletes the last character of the query. Returns whether it changed.
    pub fn backspace(&mut self) -> bool {
        if self.query.pop().is_none() {
            return false;
        }
        self.refilter();
        true
    }

    /// Highlights the previous row. Returns whether it moved.
    pub fn up(&mut self) -> bool {
        if self.selected == 0 {
            return false;
        }
        self.selected -= 1;
        true
    }

    /// Highlights the next row. Returns whether it moved.
    pub fn down(&mut self) -> bool {
        if self.selected + 1 >= self.matches.len() {
            return false;
        }
        self.selected += 1;
        true
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
        assert_eq!(rank("ell", &rows), [1]);
        assert_eq!(rank("", &rows), [0, 1, 2]);
        assert_eq!(rank("é", &["Été"]), [0]);
    }

    #[test]
    fn rank_rejects_letters_out_of_order() {
        // Arrange
        let rows = ["Rectangle", "Redo"];

        // Act / Assert
        assert_eq!(rank("tcr", &rows).len(), 0);
        assert_eq!(rank("redoo", &rows).len(), 0);
        assert_eq!(rank("x", &rows).len(), 0);
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
        assert_eq!(
            chord_label(KeyChord::ctrl_shift(Key::BracketRight)),
            "Ctrl+Shift+]"
        );
        assert_eq!(chord_label(KeyChord::alt(Key::ArrowLeft)), "Alt+Left");
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

    /// The overlays draw with the built-in font, which has no arrows or
    /// accents (T122).
    #[test]
    fn every_binding_label_and_description_is_ascii() {
        for &(chord, _, description) in BINDINGS {
            assert!(chord_label(chord).is_ascii(), "{chord:?}");
            assert!(description.is_ascii(), "{description}");
        }
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
        assert!(
            palette
                .rows()
                .all(|(_, _, d)| d.to_lowercase().contains('r'))
        );
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
