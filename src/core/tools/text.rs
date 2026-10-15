//! Text tool and the text edit session (ADR-T29-2).
//!
//! A press with the Text tool re-opens the text block under it (caret at
//! the press) or starts a new block whose top-left corner is the press
//! point, snapped like a shape's start point. The block being edited is a
//! [`TextEdit`] held in [`State`]: it is not part of the document until the
//! editor commits it, so typing never touches the history. The editor
//! routes typed characters and editing keys here and decides when an edit
//! ends.

use super::{Overlay, Phase, Pointer, ToolCtx, ToolView};
use crate::core::document::ShapeId;
use crate::core::geom::{Aabb, Vec2};
use crate::core::shape::{Shape, Style};

/// On-screen font size, in pixels, of a new block at stroke width 0.
pub const TEXT_BASE_PX: f32 = 12.0;

/// On-screen font size added per pixel of stroke width (`[` / `]`).
pub const TEXT_PX_PER_WIDTH: f32 = 2.0;

/// Spaces typed by `Tab`.
pub const TAB_SPACES: usize = 4;

/// On-screen font size in pixels of a new block drawn at stroke width
/// `width_px`.
#[must_use]
pub fn text_size_px(width_px: f32) -> f32 {
    todo!("T29: text size for width {width_px}")
}

/// A text block being edited: its text, caret and placement.
#[derive(Debug, Clone, PartialEq)]
pub struct TextEdit {
    /// Top-left corner of the block, world units.
    pub origin: Vec2,
    /// Font size in world units.
    pub size: f32,
    /// Text colour (`width` unused).
    pub style: Style,
    /// The document block being re-edited, `None` for a new block.
    target: Option<ShapeId>,
    /// The text.
    text: String,
    /// Byte index of the caret, always on a `char` boundary.
    caret: usize,
}

impl TextEdit {
    /// A new, empty block at `origin`.
    #[must_use]
    pub fn new(origin: Vec2, size: f32, style: Style) -> Self {
        todo!("T29: new edit at {origin:?}, {size}, {style:?}")
    }

    /// Re-opens the text block `shape` (document id `id`) with the caret
    /// nearest to world point `at`; `None` if `shape` is not a text block.
    #[must_use]
    pub fn open(id: ShapeId, shape: &Shape, at: Vec2) -> Option<Self> {
        todo!("T29: open {id:?} {shape:?} at {at:?}")
    }

    /// The document block being re-edited, `None` for a new block.
    #[must_use]
    pub fn target(&self) -> Option<ShapeId> {
        self.target
    }

    /// The text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Byte index of the caret; always on a `char` boundary.
    #[must_use]
    pub fn caret(&self) -> usize {
        self.caret
    }

    /// Whether the text is empty or only whitespace (such a block is
    /// discarded on commit).
    #[must_use]
    pub fn is_blank(&self) -> bool {
        todo!("T29: is_blank")
    }

    /// The block as a document shape.
    #[must_use]
    pub fn shape(&self) -> Shape {
        todo!("T29: shape")
    }

    /// World box of the block.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        todo!("T29: bounds")
    }

    /// The caret as a world segment, top to bottom of its line.
    #[must_use]
    pub fn caret_line(&self) -> [Vec2; 2] {
        todo!("T29: caret_line")
    }

    /// Moves the caret to byte `index`, snapped back to a `char` boundary
    /// and clamped to the text.
    pub fn set_caret(&mut self, index: usize) {
        todo!("T29: set_caret {index}")
    }

    /// Moves the caret to the cell boundary nearest to world point `p`.
    pub fn set_caret_at(&mut self, p: Vec2) {
        todo!("T29: set_caret_at {p:?}")
    }

    /// Inserts `ch` at the caret and moves past it. Returns `true`.
    pub fn insert(&mut self, ch: char) -> bool {
        todo!("T29: insert {ch:?}")
    }

    /// Inserts [`TAB_SPACES`] spaces at the caret. Returns `true`.
    pub fn tab(&mut self) -> bool {
        todo!("T29: tab")
    }

    /// Removes the `char` before the caret. Returns whether there was one.
    pub fn backspace(&mut self) -> bool {
        todo!("T29: backspace")
    }

    /// Removes the `char` after the caret. Returns whether there was one.
    pub fn delete(&mut self) -> bool {
        todo!("T29: delete")
    }

    /// Moves the caret one `char` back. Returns whether it moved.
    pub fn left(&mut self) -> bool {
        todo!("T29: left")
    }

    /// Moves the caret one `char` forward. Returns whether it moved.
    pub fn right(&mut self) -> bool {
        todo!("T29: right")
    }

    /// Moves the caret to the previous line, same column (clamped).
    /// Returns whether it moved.
    pub fn up(&mut self) -> bool {
        todo!("T29: up")
    }

    /// Moves the caret to the next line, same column (clamped). Returns
    /// whether it moved.
    pub fn down(&mut self) -> bool {
        todo!("T29: down")
    }

    /// Moves the caret to the start of its line. Returns whether it moved.
    pub fn home(&mut self) -> bool {
        todo!("T29: home")
    }

    /// Moves the caret to the end of its line. Returns whether it moved.
    pub fn end(&mut self) -> bool {
        todo!("T29: end")
    }
}

/// State of the Text tool: the block being edited, if any.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// The edit session; it outlives pointer gestures.
    pub edit: Option<TextEdit>,
}

/// Handles one pointer event of a Text-tool gesture: a `Down` opens the
/// block under the pointer or a new one (the editor has already committed
/// any previous edit). Returns whether a redraw is needed.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    todo!("T29: on_pointer {state:?} {:?} {pointer:?}", ctx.tool)
}

/// What the edit session draws: the edited block, with the original
/// hidden while re-editing.
#[must_use]
pub fn preview(state: &State, view: &ToolView<'_>) -> Overlay {
    todo!("T29: preview {state:?} {:?}", view.tool)
}

/// Ends the gesture. The edit session is not part of the gesture and
/// stays open; returns `false`.
pub fn cancel(state: &mut State) -> bool {
    todo!("T29: cancel {state:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use proptest::prelude::*;

    const STYLE: Style = Style {
        color: ColorId::INK,
        width: 0.0,
    };

    /// An edit of `text` with the caret at byte `caret`.
    fn edit(text: &str, caret: usize) -> TextEdit {
        let mut e = TextEdit::new(Vec2::ZERO, 10.0, STYLE);
        for ch in text.chars() {
            e.insert(ch);
        }
        e.set_caret(caret);
        e
    }

    #[test]
    fn text_size_px_follows_width_ladder() {
        assert!(approx_eq(text_size_px(3.0), 18.0, 1e-6));
        assert!(approx_eq(text_size_px(1.0), 14.0, 1e-6));
        assert!(approx_eq(text_size_px(32.0), 76.0, 1e-6));
    }

    #[test]
    fn edit_new_is_empty_with_caret_at_start() {
        let e = TextEdit::new(Vec2::new(1.0, 2.0), 10.0, STYLE);

        assert_eq!(e.text(), "");
        assert_eq!(e.caret(), 0);
        assert!(e.is_blank());
        assert_eq!(e.target(), None);
    }

    #[test]
    fn edit_insert_places_char_at_caret() {
        // Arrange
        let mut e = edit("ac", 1);

        // Act
        let changed = e.insert('β');

        // Assert
        assert!(changed);
        assert_eq!(e.text(), "aβc");
        assert_eq!(e.caret(), 3);
    }

    #[test]
    fn edit_backspace_removes_multibyte_char() {
        let mut e = edit("aβ∑", "aβ∑".len());

        assert!(e.backspace());
        assert_eq!(e.text(), "aβ");
        assert!(e.backspace());
        assert_eq!(e.text(), "a");
        assert_eq!(e.caret(), 1);
    }

    #[test]
    fn edit_delete_removes_char_after_caret() {
        let mut e = edit("aβc", 1);

        assert!(e.delete());

        assert_eq!(e.text(), "ac");
        assert_eq!(e.caret(), 1);
    }

    #[test]
    fn edit_left_right_step_over_multibyte_chars() {
        let mut e = edit("a∑b", 0);

        assert!(e.right());
        assert_eq!(e.caret(), 1);
        assert!(e.right());
        assert_eq!(e.caret(), 4);
        assert!(e.left());
        assert_eq!(e.caret(), 1);
    }

    #[test]
    fn edit_home_end_go_to_line_bounds() {
        // "ab\ncde": the caret after 'd' is byte 5.
        let mut e = edit("ab\ncde", 5);

        assert!(e.home());
        assert_eq!(e.caret(), 3);
        assert!(e.end());
        assert_eq!(e.caret(), 6);
        assert!(!e.end());
    }

    #[test]
    fn edit_up_down_keep_column_clamped() {
        // Arrange: caret at column 3 of "abcd".
        let mut e = edit("abcd\nx\nwxyz", 3);

        // Act / Assert: down to the short line clamps to its end.
        assert!(e.down());
        assert_eq!(e.caret(), 6);
        assert!(e.down());
        assert_eq!(e.caret(), 8);
        assert!(e.up());
        assert_eq!(e.caret(), 6);
        assert!(e.up());
        assert_eq!(e.caret(), 1);
    }

    #[test]
    fn edit_enter_inserts_newline() {
        let mut e = edit("ab", 1);

        assert!(e.insert('\n'));

        assert_eq!(e.text(), "a\nb");
        assert_eq!(e.caret(), 2);
    }

    #[test]
    fn edit_tab_inserts_spaces() {
        let mut e = edit("ab", 1);

        assert!(e.tab());

        assert_eq!(e.text(), format!("a{}b", " ".repeat(TAB_SPACES)));
        assert_eq!(e.caret(), 1 + TAB_SPACES);
    }

    #[test]
    fn edit_ends_of_text_are_noops() {
        let mut start = edit("ab", 0);
        let mut end = edit("ab", 2);

        assert!(!start.left() && !start.backspace() && !start.up());
        assert!(!end.right() && !end.delete() && !end.down());
        assert_eq!(start.text(), "ab");
        assert_eq!(end.text(), "ab");
    }

    #[test]
    fn edit_set_caret_snaps_into_text() {
        let mut e = edit("aβ", 0);

        // Byte 2 is inside 'β' (bytes 1..3): it snaps back to 1.
        e.set_caret(2);
        assert_eq!(e.caret(), 1);
        e.set_caret(99);
        assert_eq!(e.caret(), 3);
    }

    #[test]
    fn edit_is_blank_for_whitespace_only() {
        assert!(edit(" \n\t ", 0).is_blank());
        assert!(!edit(" x ", 0).is_blank());
    }

    #[test]
    fn edit_shape_is_text_at_origin() {
        // Arrange
        let mut e = TextEdit::new(Vec2::new(5.0, 6.0), 12.0, STYLE);
        e.insert('x');

        // Act
        let shape = e.shape();

        // Assert
        assert_eq!(
            shape,
            Shape::Text {
                origin: Vec2::new(5.0, 6.0),
                text: "x".to_owned(),
                size: 12.0,
                style: STYLE,
            }
        );
    }

    #[test]
    fn edit_open_reads_text_shape_and_places_caret() {
        // Arrange
        let shape = Shape::Text {
            origin: Vec2::ZERO,
            text: "abcd".to_owned(),
            size: 10.0,
            style: STYLE,
        };
        let id = crate::core::document::ShapeId(7);

        // Act: column boundary 2 lies at x = 12.
        let Some(e) = TextEdit::open(id, &shape, Vec2::new(12.5, 4.0)) else {
            panic!("a text shape opens");
        };

        // Assert
        assert_eq!(e.target(), Some(id));
        assert_eq!(e.text(), "abcd");
        assert_eq!(e.caret(), 2);
        assert!(approx_eq(e.size, 10.0, 1e-6));
    }

    #[test]
    fn edit_open_rejects_other_shapes() {
        let line = Shape::Line {
            a: Vec2::ZERO,
            b: Vec2::new(1.0, 1.0),
            style: STYLE,
        };

        assert!(TextEdit::open(crate::core::document::ShapeId(1), &line, Vec2::ZERO).is_none());
    }

    #[test]
    fn edit_caret_line_spans_one_line() {
        let e = edit("ab\nc", 3);

        let [top, bottom] = e.caret_line();

        assert!(top.approx_eq(Vec2::new(0.0, 12.5), 1e-4));
        assert!(bottom.approx_eq(Vec2::new(0.0, 25.0), 1e-4));
    }

    /// One random editing operation.
    #[derive(Debug, Clone)]
    enum Op {
        Insert(char),
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        Home,
        End,
        Tab,
        Caret(usize),
        Click(f32, f32),
    }

    fn arb_op() -> impl Strategy<Value = Op> {
        prop_oneof![
            4 => prop_oneof![Just('a'), Just('\n'), Just('β'), Just('∑'), Just('𝔸'), any::<char>()]
                .prop_map(Op::Insert),
            1 => Just(Op::Backspace),
            1 => Just(Op::Delete),
            1 => Just(Op::Left),
            1 => Just(Op::Right),
            1 => Just(Op::Up),
            1 => Just(Op::Down),
            1 => Just(Op::Home),
            1 => Just(Op::End),
            1 => Just(Op::Tab),
            1 => (0usize..200).prop_map(Op::Caret),
            1 => (-50.0f32..300.0, -50.0f32..300.0).prop_map(|(x, y)| Op::Click(x, y)),
        ]
    }

    proptest! {
        #[test]
        fn edit_random_operations_keep_caret_on_boundary(
            ops in prop::collection::vec(arb_op(), 0..80),
        ) {
            // Arrange
            let mut e = TextEdit::new(Vec2::ZERO, 10.0, STYLE);

            // Act
            for op in ops {
                match op {
                    Op::Insert(ch) => { e.insert(ch); }
                    Op::Backspace => { e.backspace(); }
                    Op::Delete => { e.delete(); }
                    Op::Left => { e.left(); }
                    Op::Right => { e.right(); }
                    Op::Up => { e.up(); }
                    Op::Down => { e.down(); }
                    Op::Home => { e.home(); }
                    Op::End => { e.end(); }
                    Op::Tab => { e.tab(); }
                    Op::Caret(i) => e.set_caret(i),
                    Op::Click(x, y) => e.set_caret_at(Vec2::new(x, y)),
                }

                // Assert
                prop_assert!(e.caret() <= e.text().len());
                prop_assert!(e.text().is_char_boundary(e.caret()));
            }
        }
    }
}
