//! Text tool and the text edit session (ADR-T29-2).

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
