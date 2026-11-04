//! Editor snapshots (ADR-T74-1): the whole headless [`Editor`] state as a
//! small, canonical, line-based text, and back.
//!
//! ```text
//! draw-snapshot 1
//! [editor]
//! tool pen
//! …
//! [document]
//! next-id 3
//! shape 1 line 0 3.0 10.0 10.0 50.0 40.0
//! …
//! ```
//!
//! [`Editor::snapshot`] captures a state, [`Snapshot::to_text`] writes it,
//! [`Snapshot::parse`] reads and validates text, and
//! [`Editor::from_snapshot`] turns a snapshot back into an editor. The
//! text is canonical: one state has exactly one text. Private tool states
//! write and read their own records through [`Line`] and [`Tokens`].

use std::fmt;

use crate::core::editor::Editor;

/// First token of a snapshot's first line.
pub const HEADER: &str = "draw-snapshot";

/// Version of the format this build writes and reads. Bumped, with an ADR,
/// whenever the text changes (ADR-T74-1).
pub const VERSION: u32 = 1;

/// Why a text is not a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotError {
    /// 1-based line of the problem.
    pub line: usize,
    /// What is wrong, for people.
    pub message: String,
}

impl SnapshotError {
    /// An error at `line` (1-based).
    #[must_use]
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for SnapshotError {}

/// A validated editor state: what [`Editor::snapshot`] captures and
/// [`Snapshot::parse`] accepts. Loading it cannot fail.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot(pub(crate) Editor);

impl Snapshot {
    /// The canonical text of this state.
    #[must_use]
    pub fn to_text(&self) -> String {
        todo!()
    }

    /// Reads a snapshot from `text`.
    ///
    /// # Errors
    ///
    /// A [`SnapshotError`] naming the first bad line: wrong header or
    /// version, unknown section or key, a malformed or non-finite number,
    /// or a state that breaks an editor invariant.
    pub fn parse(text: &str) -> Result<Self, SnapshotError> {
        let _ = text;
        todo!()
    }
}

/// Builds snapshot text section by section.
#[derive(Debug, Default)]
pub struct Writer {
    out: String,
}

/// The tokens of one record being written.
#[derive(Debug)]
pub struct Line<'a> {
    out: &'a mut String,
}

/// Reads snapshot text line by line.
#[derive(Debug)]
pub struct Reader<'a> {
    lines: Vec<&'a str>,
    next: usize,
}

/// One token of a record.
#[derive(Debug, Clone, PartialEq)]
enum Token<'a> {
    /// A bare word: a key, number, name or `-`.
    Word(&'a str),
    /// A quoted string, unescaped.
    Str(String),
}

/// The tokens of one record being read.
#[derive(Debug)]
pub struct Tokens<'a> {
    line: usize,
    items: Vec<Token<'a>>,
    next: usize,
}

/// A value with a token form inside a record.
pub trait Snap: Sized {
    /// Appends the value's tokens.
    fn write(&self, line: &mut Line<'_>);

    /// Reads the value's tokens.
    ///
    /// # Errors
    ///
    /// When the tokens are missing, malformed or out of range.
    fn read(tokens: &mut Tokens<'_>) -> Result<Self, SnapshotError>;
}

/// Golden files: compare a state's text with a committed one (AC-5). The
/// file system and `DRAW_UPDATE_SNAPSHOTS` stay with the caller.
pub mod golden {
    /// The outcome of [`check`].
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Golden {
        /// The texts are equal.
        Match,
        /// The caller should write the actual text (update requested).
        Write,
        /// The texts differ: a readable line diff.
        Mismatch(String),
        /// There is no golden file and no update was requested.
        Missing,
    }

    /// Compares `actual` with the golden `expected` text.
    #[must_use]
    pub fn check(actual: &str, expected: Option<&str>, update: bool) -> Golden {
        let _ = (actual, expected, update);
        todo!()
    }

    /// A short line diff from `expected` to `actual`.
    #[must_use]
    pub fn diff(expected: &str, actual: &str) -> String {
        let _ = (expected, actual);
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command::{Command, Tool};
    use crate::core::editor::Editor;
    use crate::core::geom::Vec2;
    use crate::core::input::{InputEvent, Key, Modifiers, PointerButton};
    use std::path::Path;

    const NONE: Modifiers = Modifiers::NONE;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn editor() -> Editor {
        let mut ed = Editor::new();
        ed.handle(InputEvent::Resize {
            size: v(800.0, 600.0),
        });
        ed
    }

    fn down(ed: &mut Editor, pos: Vec2) {
        ed.handle(InputEvent::PointerMove { pos, mods: NONE });
        ed.handle(InputEvent::PointerDown {
            pos,
            button: PointerButton::Left,
            mods: NONE,
        });
    }

    fn move_to(ed: &mut Editor, pos: Vec2) {
        ed.handle(InputEvent::PointerMove { pos, mods: NONE });
    }

    fn up(ed: &mut Editor, pos: Vec2) {
        ed.handle(InputEvent::PointerUp {
            pos,
            button: PointerButton::Left,
            mods: NONE,
        });
    }

    fn drag(ed: &mut Editor, a: Vec2, b: Vec2) {
        down(ed, a);
        for i in 1..=8 {
            let t = i as f32 / 8.0;
            move_to(ed, v(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t + t * 3.0));
        }
        up(ed, b);
    }

    fn type_text(ed: &mut Editor, text: &str) {
        for ch in text.chars() {
            ed.handle(InputEvent::Char { ch, mods: NONE });
        }
    }

    fn key(ed: &mut Editor, key: Key) {
        ed.handle(InputEvent::KeyDown { key, mods: NONE });
        ed.handle(InputEvent::KeyUp { key, mods: NONE });
    }

    /// `ed` through text and back.
    fn round_trip(ed: &Editor) -> Editor {
        let text = ed.snapshot().to_text();
        let snapshot = Snapshot::parse(&text).unwrap_or_else(|err| panic!("{err}\n{text}"));
        Editor::from_snapshot(snapshot)
    }

    /// A few shapes, an undo step and a redo step, a selection and a
    /// clipboard.
    fn busy_editor() -> Editor {
        let mut ed = editor();
        drag(&mut ed, v(10.0, 10.0), v(200.0, 120.0));
        ed.apply(Command::SetTool(Tool::Rect));
        drag(&mut ed, v(300.0, 50.0), v(400.0, 150.0));
        ed.apply(Command::SetTool(Tool::Grid));
        drag(&mut ed, v(50.0, 300.0), v(250.0, 450.0));
        ed.apply(Command::SetTool(Tool::Arrow));
        drag(&mut ed, v(500.0, 300.0), v(700.0, 320.0));
        ed.apply(Command::SetTool(Tool::Text));
        down(&mut ed, v(450.0, 450.0));
        up(&mut ed, v(450.0, 450.0));
        type_text(&mut ed, "x = \"a\\b\"\n$\\sum_{i} i^2$");
        key(&mut ed, Key::Escape);
        ed.apply(Command::Undo);
        ed.apply(Command::SelectAll);
        ed.apply(Command::Copy);
        ed.apply(Command::SetColor(crate::core::palette::ColorId::INK));
        ed
    }

    fn fixtures() -> Vec<std::path::PathBuf> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");
        let mut found = Vec::new();
        for dir in [root.clone(), root.join("regressions")] {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "snap") {
                    found.push(path);
                }
            }
        }
        found.sort();
        found
    }

    /// The text of `ed` with `line` (0-based) replaced by `with`.
    fn edited(ed: &Editor, line: usize, with: &str) -> String {
        let text = ed.snapshot().to_text();
        let mut lines: Vec<&str> = text.lines().collect();
        lines[line] = with;
        lines.join("\n") + "\n"
    }

    /// 0-based index of the first line starting with `prefix`.
    fn line_of(text: &str, prefix: &str) -> usize {
        text.lines()
            .position(|l| l.starts_with(prefix))
            .unwrap_or_else(|| panic!("no line {prefix:?} in\n{text}"))
    }

    // AC-1

    #[test]
    fn round_trip_new_editor_is_equal() {
        // Arrange
        let ed = Editor::new();

        // Act
        let loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded, ed);
    }

    #[test]
    fn round_trip_busy_editor_is_equal_and_undo_redo_agree() {
        // Arrange
        let mut ed = busy_editor();

        // Act
        let mut loaded = round_trip(&ed);

        // Assert
        assert_eq!(loaded, ed);
        for command in [Command::Redo, Command::Undo, Command::Undo, Command::Paste] {
            ed.apply(command);
            loaded.apply(command);
            assert_eq!(loaded, ed, "after {command:?}");
        }
    }

    #[test]
    fn round_trip_mid_gesture_states() {
        // Arrange: one editor per in-progress gesture.
        let mut pen = editor();
        down(&mut pen, v(10.0, 10.0));
        move_to(&mut pen, v(40.0, 30.0));
        move_to(&mut pen, v(80.0, 35.0));

        let mut shape = editor();
        shape.apply(Command::SetTool(Tool::Ellipse));
        down(&mut shape, v(10.0, 10.0));
        move_to(&mut shape, v(90.0, 70.0));

        let mut grid = editor();
        grid.apply(Command::SetTool(Tool::Grid));
        down(&mut grid, v(10.0, 10.0));
        move_to(&mut grid, v(90.0, 70.0));

        let mut select = busy_editor();
        select.apply(Command::SetTool(Tool::Select));
        down(&mut select, v(100.0, 60.0));
        move_to(&mut select, v(130.0, 90.0));

        let mut marquee = busy_editor();
        marquee.apply(Command::SetTool(Tool::Select));
        down(&mut marquee, v(780.0, 590.0));
        move_to(&mut marquee, v(600.0, 400.0));

        let mut eraser = busy_editor();
        eraser.apply(Command::SetTool(Tool::Eraser));
        down(&mut eraser, v(5.0, 5.0));
        move_to(&mut eraser, v(200.0, 120.0));

        let mut text = editor();
        text.apply(Command::SetTool(Tool::Text));
        down(&mut text, v(100.0, 100.0));
        up(&mut text, v(100.0, 100.0));
        type_text(&mut text, "ab $x^2$ cd");
        key(&mut text, Key::ArrowLeft);
        key(&mut text, Key::ArrowLeft);
        key(&mut text, Key::ArrowLeft);
        key(&mut text, Key::ArrowLeft);

        let mut pan = editor();
        pan.handle(InputEvent::PointerDown {
            pos: v(50.0, 50.0),
            button: PointerButton::Middle,
            mods: NONE,
        });
        move_to(&mut pan, v(70.0, 90.0));

        let mut tap = Editor::new().with_tap_slop(16.0);
        down(&mut tap, v(10.0, 10.0));
        move_to(&mut tap, v(12.0, 11.0));
        tap.handle(InputEvent::Pressure {
            pressure: Some(0.25),
        });

        for (name, ed) in [
            ("pen", pen),
            ("shape", shape),
            ("grid", grid),
            ("select", select),
            ("marquee", marquee),
            ("eraser", eraser),
            ("text", text),
            ("pan", pan),
            ("tap", tap),
        ] {
            // Act
            let mut loaded = round_trip(&ed);
            let mut ed = ed;

            // Assert: equal now and after the gesture ends the same way.
            assert_eq!(loaded, ed, "{name}");
            for event in [
                InputEvent::PointerMove {
                    pos: v(150.0, 140.0),
                    mods: NONE,
                },
                InputEvent::Char { ch: 'z', mods: NONE },
                InputEvent::PointerUp {
                    pos: v(150.0, 140.0),
                    button: PointerButton::Left,
                    mods: NONE,
                },
                InputEvent::PointerUp {
                    pos: v(150.0, 140.0),
                    button: PointerButton::Middle,
                    mods: NONE,
                },
                InputEvent::KeyDown {
                    key: Key::Escape,
                    mods: NONE,
                },
            ] {
                ed.handle(event);
                loaded.handle(event);
                assert_eq!(loaded, ed, "{name} after {event:?}");
            }
        }
    }

    #[test]
    fn snapshot_ignores_document_revisions() {
        // Arrange
        let ed = busy_editor();
        assert!(ed.document_revision() > 0);

        // Act
        let loaded = round_trip(&ed);

        // Assert: a load is a fresh start for the shell (ADR-T74-1).
        assert_eq!(loaded.document_revision(), 0);
        assert_eq!(loaded.document_base_revision(), 0);
        assert_eq!(loaded, ed);
    }

    // AC-2

    #[test]
    fn fixtures_are_canonical() {
        // Arrange
        let paths = fixtures();
        assert!(!paths.is_empty(), "no fixtures in tests/snapshots");

        for path in paths {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("{}: {err}", path.display()));

            // Act
            let snapshot =
                Snapshot::parse(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));

            // Assert
            assert_eq!(snapshot.to_text(), text, "{} is not canonical", path.display());
        }
    }

    #[test]
    fn text_is_canonical_for_busy_editor() {
        // Arrange
        let text = busy_editor().snapshot().to_text();

        // Act
        let again = Snapshot::parse(&text).map(|s| s.to_text());

        // Assert
        assert_eq!(again.as_deref(), Ok(text.as_str()));
        assert!(text.starts_with(&format!("{HEADER} {VERSION}\n")));
        assert!(text.ends_with('\n'));
    }

    #[test]
    fn strings_round_trip_with_escapes() {
        // Arrange
        let mut ed = editor();
        ed.apply(Command::SetTool(Tool::Text));
        down(&mut ed, v(100.0, 100.0));
        up(&mut ed, v(100.0, 100.0));
        type_text(&mut ed, "q\"uo\\te\ttab\nline é ∑ \u{7f} end ");

        // Act
        let text = ed.snapshot().to_text();
        let loaded = round_trip(&ed);

        // Assert: one record per line, and the text comes back.
        assert_eq!(loaded, ed);
        assert_eq!(loaded.text_edit().map(|e| e.text().to_owned()), ed.text_edit().map(|e| e.text().to_owned()));
        assert!(!text.contains('\t'), "tabs are escaped");
        assert!(!text.contains('\u{7f}'), "control characters are escaped");
    }

    #[test]
    fn floats_keep_their_bits() {
        // Arrange: awkward zoom, offsets and widths.
        let mut ed = editor();
        for delta in [0.37, -1.3, 2.9] {
            ed.handle(InputEvent::Scroll {
                pos: v(123.456, 78.9),
                delta,
            });
        }
        drag(&mut ed, v(0.1, 0.2), v(333.333, 1e-3));

        // Act
        let loaded = round_trip(&ed);

        // Assert
        assert_eq!(
            loaded.camera().zoom().to_bits(),
            ed.camera().zoom().to_bits()
        );
        assert_eq!(
            loaded.camera().offset().x.to_bits(),
            ed.camera().offset().x.to_bits()
        );
        assert_eq!(loaded, ed);
    }

    // AC-3

    #[test]
    fn parse_error_names_line() {
        // Arrange
        let ed = busy_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "camera ");

        // Act
        let err = Snapshot::parse(&edited(&ed, line, "camera 1.0 oops 1.0"));

        // Assert
        let err = err.err().unwrap_or_else(|| panic!("bad camera accepted"));
        assert_eq!(err.line, line + 1);
        assert!(err.to_string().starts_with(&format!("line {}: ", line + 1)), "{err}");
    }

    #[test]
    fn parse_rejects_non_finite_float() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "camera ");

        for bad in ["camera NaN 0.0 1.0", "camera 0.0 inf 1.0", "camera 0.0 0.0 -inf"] {
            // Act
            let result = Snapshot::parse(&edited(&ed, line, bad));

            // Assert
            assert_eq!(result.err().map(|e| e.line), Some(line + 1), "{bad}");
        }
    }

    #[test]
    fn parse_rejects_zoom_out_of_range() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "camera ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "camera 0.0 0.0 1000000.0"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn parse_rejects_duplicate_id() {
        // Arrange
        let ed = busy_editor();
        let text = ed.snapshot().to_text();
        let first = line_of(&text, "shape ");
        let mut lines: Vec<&str> = text.lines().collect();
        let copy = lines[first];
        lines.insert(first + 1, copy);
        let text = lines.join("\n") + "\n";

        // Act
        let result = Snapshot::parse(&text);

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(first + 2));
    }

    #[test]
    fn parse_rejects_id_at_or_above_next_id() {
        // Arrange
        let ed = busy_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "next-id ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "next-id 1"));

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_selection_outside_document() {
        // Arrange
        let ed = busy_editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "selection");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "selection 999"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn parse_rejects_unknown_color() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "style ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "style 200 3.0"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn parse_rejects_caret_off_char_boundary() {
        // Arrange: an open block holding "é", caret after it (2 bytes).
        let mut ed = editor();
        ed.apply(Command::SetTool(Tool::Text));
        down(&mut ed, v(100.0, 100.0));
        up(&mut ed, v(100.0, 100.0));
        type_text(&mut ed, "é");
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "text ");
        let bad = text.lines().nth(line).map(|l| l.replace("\"é\" 2 ", "\"é\" 1 "));
        let bad = bad.unwrap_or_default();
        assert_ne!(bad, text.lines().nth(line).unwrap_or_default(), "caret token not found");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, &bad));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn parse_rejects_truncated_text() {
        // Arrange
        let text = busy_editor().snapshot().to_text();

        for end in (0..text.len()).filter(|&i| text.is_char_boundary(i)).step_by(7) {
            // Act
            let result = Snapshot::parse(&text[..end]);

            // Assert: a prefix that drops whole records is still a valid
            // snapshot only if every section is there; never a panic.
            if let Err(err) = result {
                assert!(err.line >= 1, "{err}");
            }
        }
    }

    #[test]
    fn parse_rejects_empty_and_garbage() {
        for text in ["", "\n", "draw-snapshot", "hello\nworld\n", "\u{0}\u{0}"] {
            // Act
            let result = Snapshot::parse(text);

            // Assert
            assert_eq!(result.err().map(|e| e.line), Some(1), "{text:?}");
        }
    }

    // AC-4

    #[test]
    fn rejects_unknown_version() {
        // Arrange
        let ed = editor();
        let next = format!("{HEADER} {}", VERSION + 1);

        // Act
        let result = Snapshot::parse(&edited(&ed, 0, &next));

        // Assert
        let err = result.err().unwrap_or_else(|| panic!("version {} accepted", VERSION + 1));
        assert_eq!(err.line, 1);
        assert!(err.message.contains("version"), "{err}");
    }

    #[test]
    fn rejects_unknown_section() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "[clipboard]");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "[pasteboard]"));

        // Assert
        let err = result.err().unwrap_or_else(|| panic!("unknown section accepted"));
        assert_eq!(err.line, line + 1);
        assert!(err.message.contains("section"), "{err}");
    }

    #[test]
    fn rejects_unknown_key() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "toolbar ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "sidebar 1"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }

    #[test]
    fn rejects_trailing_tokens() {
        // Arrange
        let ed = editor();
        let text = ed.snapshot().to_text();
        let line = line_of(&text, "toolbar ");

        // Act
        let result = Snapshot::parse(&edited(&ed, line, "toolbar 1 1"));

        // Assert
        assert_eq!(result.err().map(|e| e.line), Some(line + 1));
    }
}

#[cfg(test)]
mod golden_tests {
    use super::golden::{Golden, check, diff};

    // AC-5

    #[test]
    fn check_matches_equal_text() {
        // Arrange / Act / Assert
        assert_eq!(check("a\nb\n", Some("a\nb\n"), false), Golden::Match);
        assert_eq!(check("a\nb\n", Some("a\nb\n"), true), Golden::Match);
    }

    #[test]
    fn check_mismatch_has_line_diff() {
        // Arrange
        let expected = "head\nkeep\nold one\nold two\ntail\n";
        let actual = "head\nkeep\nnew one\ntail\n";

        // Act
        let result = check(actual, Some(expected), false);

        // Assert
        let Golden::Mismatch(text) = result else {
            panic!("expected a mismatch, got {result:?}");
        };
        assert_eq!(text, diff(expected, actual));
        assert!(text.contains("line 3"), "{text}");
        assert!(text.contains("- old one\n- old two\n+ new one\n"), "{text}");
        assert!(!text.contains("keep") && !text.contains("tail"), "{text}");
    }

    #[test]
    fn check_writes_only_when_updating() {
        // Arrange / Act / Assert
        assert_eq!(check("new\n", Some("old\n"), true), Golden::Write);
        assert_eq!(check("new\n", None, true), Golden::Write);
        assert!(matches!(check("new\n", Some("old\n"), false), Golden::Mismatch(_)));
    }

    #[test]
    fn check_missing_without_update() {
        // Arrange / Act / Assert
        assert_eq!(check("new\n", None, false), Golden::Missing);
    }

    #[test]
    fn diff_caps_long_changes() {
        // Arrange
        let expected: String = (0..1000).map(|i| format!("a{i}\n")).collect();
        let actual: String = (0..1000).map(|i| format!("b{i}\n")).collect();

        // Act
        let text = diff(&expected, &actual);

        // Assert: readable, not the whole file.
        assert!(text.lines().count() < 100, "{} lines", text.lines().count());
        assert!(text.contains("more"), "{text}");
    }
}
