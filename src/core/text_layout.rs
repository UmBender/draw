//! Monospaced text layout (ADR-T29-1).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Vec2, approx_eq};
    use proptest::prelude::*;

    const EPS: f32 = 1e-4;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    #[test]
    fn layout_metrics_match_the_embedded_font() {
        assert!(approx_eq(ADVANCE, 0.6, EPS));
        assert!(approx_eq(LINE_HEIGHT, 1.25, EPS));
        assert!(approx_eq(BASELINE, 1.0, EPS));
    }

    #[test]
    fn layout_bounds_empty_text_is_one_line_tall() {
        // Act
        let b = bounds(v(10.0, 20.0), "", 10.0);

        // Assert
        assert!(b.min.approx_eq(v(10.0, 20.0), EPS));
        assert!(b.max.approx_eq(v(10.0, 32.5), EPS));
    }

    #[test]
    fn layout_bounds_multiline_uses_longest_line() {
        // Act
        let b = bounds(v(0.0, 0.0), "ab\nabcd\n", 10.0);

        // Assert: 4 columns of 6, 3 lines of 12.5.
        assert!(b.max.approx_eq(v(24.0, 37.5), EPS), "{b:?}");
        assert_eq!(extent("ab\nabcd\n"), (4, 3));
    }

    #[test]
    fn layout_bounds_counts_chars_not_bytes() {
        // "αβγ≤" is 4 chars in 9 bytes.
        let b = bounds(v(0.0, 0.0), "αβγ≤", 10.0);

        assert!(approx_eq(b.width(), 24.0, EPS), "{b:?}");
    }

    #[test]
    fn layout_non_finite_size_counts_as_zero() {
        for size in [f32::NAN, f32::INFINITY, -3.0] {
            let b = bounds(v(1.0, 2.0), "abc", size);

            assert!(b.min.approx_eq(v(1.0, 2.0), EPS), "{size}");
            assert!(b.max.approx_eq(v(1.0, 2.0), EPS), "{size}");
        }
    }

    #[test]
    fn layout_line_col_and_caret_index_round_trip() {
        let text = "aβ\ncd";

        assert_eq!(line_col(text, 0), (0, 0));
        assert_eq!(line_col(text, 3), (0, 2));
        assert_eq!(line_col(text, 4), (1, 0));
        assert_eq!(line_col(text, text.len()), (1, 2));
        assert_eq!(caret_index(text, 0, 2), 3);
        assert_eq!(caret_index(text, 1, 1), 5);
        // Columns past the end clamp to the line end, lines to the last.
        assert_eq!(caret_index(text, 0, 9), 3);
        assert_eq!(caret_index(text, 7, 0), 4);
    }

    #[test]
    fn layout_caret_point_after_newline_starts_next_line() {
        // Act
        let p = caret_point(v(5.0, 5.0), "ab\ncd", 10.0, 3);

        // Assert
        assert!(p.approx_eq(v(5.0, 17.5), EPS), "{p:?}");
        assert!(caret_point(v(5.0, 5.0), "ab\ncd", 10.0, 2).approx_eq(v(17.0, 5.0), EPS));
    }

    #[test]
    fn layout_caret_from_point_rounds_to_nearest_column() {
        let text = "abcd\nef";
        let origin = v(0.0, 0.0);

        // Column boundaries at 0, 6, 12 …; 8 is nearer 6, 10 nearer 12.
        assert_eq!(caret_from_point(origin, text, 10.0, v(8.0, 3.0)), 1);
        assert_eq!(caret_from_point(origin, text, 10.0, v(10.0, 3.0)), 2);
        assert_eq!(caret_from_point(origin, text, 10.0, v(1.0, 14.0)), 5);
    }

    #[test]
    fn layout_caret_from_point_clamps_outside_text() {
        let text = "abcd\nef";
        let origin = v(0.0, 0.0);

        assert_eq!(caret_from_point(origin, text, 10.0, v(-50.0, -50.0)), 0);
        assert_eq!(caret_from_point(origin, text, 10.0, v(500.0, 2.0)), 4);
        assert_eq!(
            caret_from_point(origin, text, 10.0, v(500.0, 500.0)),
            text.len()
        );
        assert_eq!(caret_from_point(origin, text, 10.0, v(f32::NAN, 2.0)), 0);
        assert_eq!(caret_from_point(origin, text, f32::NAN, v(5.0, 2.0)), 0);
    }

    #[test]
    fn layout_glyphs_skip_newlines_and_advance_by_cells() {
        // Act
        let glyphs: Vec<Glyph> = glyphs(v(0.0, 0.0), "aβ\nc", 10.0).collect();

        // Assert: baseline-left positions of each cell.
        assert_eq!(glyphs.len(), 3);
        assert_eq!(glyphs[0].ch, 'a');
        assert!(glyphs[0].pos.approx_eq(v(0.0, 10.0), EPS));
        assert_eq!(glyphs[1].ch, 'β');
        assert!(glyphs[1].pos.approx_eq(v(6.0, 10.0), EPS));
        assert_eq!(glyphs[2].ch, 'c');
        assert!(glyphs[2].pos.approx_eq(v(0.0, 22.5), EPS));
    }

    fn arb_text() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                Just('a'),
                Just('\n'),
                Just('é'),
                Just('∑'),
                Just('𝔸'),
                any::<char>(),
            ],
            0..40,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    proptest! {
        #[test]
        fn layout_caret_from_point_is_char_boundary(
            text in arb_text(),
            x in -100.0f32..400.0,
            y in -100.0f32..400.0,
            size in prop_oneof![0.1f32..50.0, Just(f32::NAN), Just(0.0f32)],
        ) {
            // Act
            let caret = caret_from_point(v(0.0, 0.0), &text, size, v(x, y));

            // Assert
            prop_assert!(caret <= text.len());
            prop_assert!(text.is_char_boundary(caret));
        }

        #[test]
        fn layout_bounds_contain_every_glyph_cell(
            text in arb_text(),
            ox in -1000.0f32..1000.0,
            oy in -1000.0f32..1000.0,
            size in 0.1f32..50.0,
        ) {
            // Arrange
            let origin = v(ox, oy);
            let b = bounds(origin, &text, size).expand(1e-2);

            // Act / Assert: each glyph cell and each caret point lies inside.
            for g in glyphs(origin, &text, size) {
                let top = v(g.pos.x, g.pos.y - BASELINE * size);
                let cell = v(top.x + ADVANCE * size, top.y + LINE_HEIGHT * size);
                prop_assert!(b.contains(top) && b.contains(cell), "{g:?} {b:?}");
            }
            for (caret, _) in text.char_indices().chain([(text.len(), ' ')]) {
                let p = caret_point(origin, &text, size, caret);
                prop_assert!(b.contains(p), "caret {caret} at {p:?} {b:?}");
                let (line, col) = line_col(&text, caret);
                prop_assert_eq!(caret_index(&text, line, col), caret);
            }
        }
    }
}
