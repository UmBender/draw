//! The shortcut sheet: a paged cheat sheet with one topic per page
//! (ADR-T113-1).
//!
//! The key rows come from [`BINDINGS`], grouped by [`topic_of`], so the sheet
//! cannot drift from the real keys. Gestures and typing rows, which are not
//! key bindings, live in one table here. Nothing here is saved or recorded in
//! the history.

use crate::core::command_palette::chord_label;
use crate::core::keymap::{BINDINGS, Topic, topic_of};

/// Most rows on one page: the header, the topic line, 14 rows of 30 px and
/// the footer fit the default 1280×800 window.
pub const ROWS_PER_PAGE: usize = 14;

/// One line of a page: the keys or gesture, and what they do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetRow {
    /// How the input is written (`Ctrl+Z`, `Wheel`).
    pub keys: String,
    /// What it does.
    pub text: &'static str,
}

/// One page of the sheet: part `part` (from 0) of `parts` of a topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    /// The topic.
    pub topic: Topic,
    /// Which part of the topic this is, from 0.
    pub part: usize,
    /// How many pages the topic has.
    pub parts: usize,
}

impl Page {
    /// The page title: the topic name, with `1/2` when it is split.
    #[must_use]
    pub fn title(&self) -> String {
        if self.parts > 1 {
            format!("{} {}/{}", self.topic.name(), self.part + 1, self.parts)
        } else {
            self.topic.name().to_owned()
        }
    }

    /// The rows of this page, at most [`ROWS_PER_PAGE`].
    #[must_use]
    pub fn rows(&self) -> Vec<SheetRow> {
        let all = topic_rows(self.topic);
        let per = all.len().div_ceil(self.parts.max(1)).max(1);
        all.into_iter().skip(self.part * per).take(per).collect()
    }
}

/// Rows that are not key bindings: `(topic, input, effect)`, in the order
/// shown after the bindings of the topic.
const EXTRA_ROWS: &[(Topic, &str, &str)] = &[
    (Topic::View, "Wheel", "Zoom at the cursor"),
    (Topic::View, "Middle drag, Space+drag", "Pan"),
    (Topic::View, "Ctrl+Shift+P", "Command palette"),
    (Topic::View, "?", "This shortcut sheet"),
    (
        Topic::Gestures,
        "Shift while drawing",
        "Square / circle, 45-degree lines and arrows",
    ),
    (Topic::Gestures, "Alt while drawing", "No snapping"),
    (
        Topic::Gestures,
        "Alt + drag selection",
        "Duplicate while moving",
    ),
    (
        Topic::Gestures,
        "Drag a corner handle",
        "Resize; Shift keeps the proportions",
    ),
    (
        Topic::Gestures,
        "Right drag",
        "Temporary eraser, from any tool",
    ),
    (Topic::TypingText, "Esc, Ctrl+Enter", "Finish the block"),
    (Topic::TypingText, "Ctrl+Z", "Cancel the block"),
    (Topic::TypingText, "Enter", "New line"),
    (Topic::TypingText, "Tab", "Four spaces"),
    (Topic::TypingText, "Arrows, Home, End", "Move the cursor"),
    (Topic::TypingText, "Backspace, Delete", "Delete a character"),
    (
        Topic::TypingText,
        "Ctrl+] / Ctrl+[",
        "Bigger / smaller text",
    ),
    (Topic::TypingText, "Ctrl+V", "Paste text"),
    (
        Topic::TypingShape,
        "T, click in a shape",
        "Type inside a box, circle or grid cell",
    ),
    (
        Topic::TypingShape,
        "Tab / Shift+Tab",
        "Next / previous grid cell",
    ),
    (
        Topic::TypingShape,
        "Enter / Shift+Enter",
        "Cell below / above",
    ),
];

/// Every row of `topic`: its key bindings, then its extra rows.
fn topic_rows(topic: Topic) -> Vec<SheetRow> {
    let bindings = BINDINGS
        .iter()
        .filter(|&&(_, command, _)| topic_of(command) == topic)
        .map(|&(chord, _, text)| SheetRow {
            keys: chord_label(chord),
            text,
        });
    let extra = EXTRA_ROWS
        .iter()
        .filter(|&&(t, _, _)| t == topic)
        .map(|&(_, keys, text)| SheetRow {
            keys: keys.to_owned(),
            text,
        });
    bindings.chain(extra).collect()
}

/// Every page, topic by topic in [`Topic::ALL`] order.
#[must_use]
pub fn pages() -> Vec<Page> {
    Topic::ALL
        .iter()
        .flat_map(|&topic| {
            let parts = topic_rows(topic).len().div_ceil(ROWS_PER_PAGE).max(1);
            (0..parts).map(move |part| Page { topic, part, parts })
        })
        .collect()
}

/// The open sheet: which page it shows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sheet {
    /// Index into [`pages`].
    page: usize,
}

impl Sheet {
    /// A sheet on its first page.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Index of the current page in [`pages`].
    #[must_use]
    pub fn page(&self) -> usize {
        self.page
    }

    /// The current page.
    #[must_use]
    pub fn current(&self) -> Page {
        let all = pages();
        all.get(self.page)
            .or_else(|| all.first())
            .copied()
            .unwrap_or(Page {
                topic: Topic::Tools,
                part: 0,
                parts: 1,
            })
    }

    /// Goes to the next page, wrapping after the last.
    pub fn next(&mut self) {
        self.page = (self.page + 1) % pages().len().max(1);
    }

    /// Goes to the previous page, wrapping before the first.
    pub fn prev(&mut self) {
        let count = pages().len().max(1);
        self.page = (self.page + count - 1) % count;
    }

    /// Goes to the first page of topic number `number` (from 1). Returns
    /// whether there is such a topic.
    pub fn jump(&mut self, number: usize) -> bool {
        let Some(topic) = number.checked_sub(1).and_then(|i| Topic::ALL.get(i)) else {
            return false;
        };
        match pages().iter().position(|p| p.topic == *topic) {
            Some(index) => {
                self.page = index;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The overlays draw with the built-in font, which has no arrows or
    /// accents (T122).
    #[test]
    fn every_sheet_row_is_ascii() {
        for topic in Topic::ALL {
            for row in topic_rows(topic) {
                assert!(row.keys.is_ascii(), "{}", row.keys);
                assert!(row.text.is_ascii(), "{}", row.text);
            }
        }
    }

    /// The topic of each page, in order, without repeats.
    fn page_topics() -> Vec<Topic> {
        let mut topics: Vec<Topic> = Vec::new();
        for page in pages() {
            if topics.last() != Some(&page.topic) {
                topics.push(page.topic);
            }
        }
        topics
    }

    #[test]
    fn topics_follow_the_cheat_sheet_order() {
        // Arrange
        let names: Vec<&str> = Topic::ALL.iter().map(|t| t.name()).collect();

        // Assert
        assert_eq!(
            names,
            [
                "Tools",
                "Style",
                "Edit",
                "View",
                "Helpers",
                "Gestures",
                "Typing in a text block",
                "Typing in a shape"
            ]
        );
        assert_eq!(page_topics(), Topic::ALL);
    }

    #[test]
    fn no_page_has_more_rows_than_fit() {
        for page in pages() {
            assert!(
                page.rows().len() <= ROWS_PER_PAGE,
                "{} has {} rows",
                page.title(),
                page.rows().len()
            );
            assert!(!page.rows().is_empty(), "{} is empty", page.title());
        }
    }

    #[test]
    fn long_topic_is_split_into_numbered_parts() {
        // Arrange
        let edit: Vec<Page> = pages()
            .into_iter()
            .filter(|p| p.topic == Topic::Edit)
            .collect();
        let tools: Vec<Page> = pages()
            .into_iter()
            .filter(|p| p.topic == Topic::Tools)
            .collect();

        // Assert
        assert!(edit.len() >= 2, "Edit needs more than one page");
        let n = edit.len();
        for (i, page) in edit.iter().enumerate() {
            assert_eq!(page.title(), format!("Edit {}/{n}", i + 1));
        }
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].title(), "Tools");
    }

    #[test]
    fn split_parts_keep_every_row_once() {
        // Arrange
        let expected = BINDINGS
            .iter()
            .filter(|&&(_, command, _)| topic_of(command) == Topic::Edit)
            .count();

        // Act
        let shown: usize = pages()
            .iter()
            .filter(|p| p.topic == Topic::Edit)
            .map(|p| p.rows().len())
            .sum();

        // Assert
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_binding_is_on_exactly_one_page() {
        // Arrange
        let rows: Vec<(String, &str)> = pages()
            .iter()
            .flat_map(Page::rows)
            .map(|r| (r.keys, r.text))
            .collect();

        // Act / Assert
        for &(chord, _, description) in BINDINGS {
            let wanted = (chord_label(chord), description);
            let count = rows.iter().filter(|r| **r == wanted).count();
            assert_eq!(count, 1, "{wanted:?} is on {count} pages");
        }
    }

    #[test]
    fn gesture_and_typing_rows_have_text() {
        for page in pages() {
            if matches!(
                page.topic,
                Topic::Gestures | Topic::TypingText | Topic::TypingShape
            ) {
                let rows = page.rows();
                assert!(rows.len() > 1, "{}", page.title());
                for row in rows {
                    assert!(!row.keys.trim().is_empty(), "{}", page.title());
                    assert!(!row.text.trim().is_empty(), "{}", page.title());
                }
            }
        }
    }

    #[test]
    fn sheet_starts_on_the_first_page() {
        assert_eq!(Sheet::new().page(), 0);
        assert_eq!(Sheet::new().current().topic, Topic::Tools);
    }

    #[test]
    fn sheet_next_and_prev_wrap() {
        // Arrange
        let mut sheet = Sheet::new();
        let last = pages().len() - 1;

        // Act / Assert
        sheet.prev();
        assert_eq!(sheet.page(), last);
        sheet.next();
        assert_eq!(sheet.page(), 0);
        sheet.next();
        assert_eq!(sheet.page(), 1);
    }

    #[test]
    fn sheet_jump_goes_to_the_first_page_of_the_topic() {
        // Arrange
        let mut sheet = Sheet::new();

        // Act
        let moved = sheet.jump(3);

        // Assert
        assert!(moved);
        assert_eq!(sheet.current().topic, Topic::Edit);
        assert_eq!(sheet.current().part, 0);
        assert!(sheet.jump(4));
        assert_eq!(sheet.current().topic, Topic::View);
    }

    #[test]
    fn sheet_jump_without_a_topic_is_ignored() {
        // Arrange
        let mut sheet = Sheet::new();
        sheet.next();
        let before = sheet.page();

        // Act / Assert
        assert!(!sheet.jump(0));
        assert!(!sheet.jump(9));
        assert!(!sheet.jump(usize::MAX));
        assert_eq!(sheet.page(), before);
    }
}
