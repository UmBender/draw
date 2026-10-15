//! Monospaced text layout (ADR-T29-1).
//!
//! Every `char` of a text block is one cell of a grid: [`ADVANCE`] em wide
//! and [`LINE_HEIGHT`] em tall, lines split on `\n`. Positions are world
//! units for a block whose top-left corner is `origin` and whose font size
//! (em height) is `size`. A size that is not finite and positive counts as
//! 0, so every function is total. Carets are byte indices into the text;
//! every caret returned lies on a `char` boundary.

use std::rc::Rc;

use crate::core::geom::{Aabb, Vec2};
use crate::core::tex::Layout;

/// Width of one cell, per unit of font size: every glyph of the embedded
/// font is 600/1000 em wide.
pub const ADVANCE: f32 = 0.6;

/// Height of one line, per unit of font size: ascent 0.965 + descent 0.215
/// + line gap 0.07 em.
pub const LINE_HEIGHT: f32 = 1.25;

/// Distance from the top of a line to its baseline, per unit of font size:
/// half the line gap plus the ascent.
pub const BASELINE: f32 = 1.0;

/// One character to draw, with the left end of its baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    /// The character.
    pub ch: char,
    /// Left end of its baseline, in world units.
    pub pos: Vec2,
}

/// The lines of `text`: always at least one, possibly empty.
pub fn lines(text: &str) -> impl Iterator<Item = &str> {
    text.split('\n')
}

/// `(columns, lines)`: the length in `char`s of the longest line and the
/// number of lines (at least 1).
#[must_use]
pub fn extent(text: &str) -> (usize, usize) {
    lines(text).fold((0, 0), |(cols, count), line| {
        (cols.max(line.chars().count()), count + 1)
    })
}

/// The box of a block: `origin` to `origin + (columns × ADVANCE,
/// lines × LINE_HEIGHT) × size`.
#[must_use]
pub fn bounds(origin: Vec2, text: &str, size: f32) -> Aabb {
    let (cols, rows) = extent(text);
    Aabb::from_corners(origin, origin + cell(size, cols, rows))
}

/// The cell box of a block as typed, formulas included as source: what
/// the edit session shows (ADR-T30-1).
#[must_use]
pub fn source_bounds(origin: Vec2, text: &str, size: f32) -> Aabb {
    todo!("{origin:?} {text} {size}")
}

/// The typeset layout of `text` in em (ADR-T30-1), parsed and laid out
/// once per distinct text and then shared.
#[must_use]
pub fn typeset(text: &str) -> Rc<Layout> {
    todo!("{text}")
}

/// The `(line, column)` of `caret`, counted in `char`s. A caret inside a
/// `char` counts as the start of that `char`; one past the end as the end.
#[must_use]
pub fn line_col(text: &str, caret: usize) -> (usize, usize) {
    let caret = floor_boundary(text, caret);
    let before = &text[..caret];
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let line = before.matches('\n').count();
    (line, before[line_start..].chars().count())
}

/// The caret at `(line, col)`: the line is clamped to the last line, the
/// column to the end of its line.
#[must_use]
pub fn caret_index(text: &str, line: usize, col: usize) -> usize {
    let (start, chars) = line_spans(text)
        .take(line.saturating_add(1))
        .last()
        .unwrap_or((0, text));
    start
        + chars
            .char_indices()
            .nth(col)
            .map_or(chars.len(), |(i, _)| i)
}

/// Height of one line of text of font size `size`.
#[must_use]
pub fn line_height(size: f32) -> f32 {
    LINE_HEIGHT * em(size)
}

/// Top of the caret line at `caret`, in world units.
#[must_use]
pub fn caret_point(origin: Vec2, text: &str, size: f32, caret: usize) -> Vec2 {
    let (line, col) = line_col(text, caret);
    origin + cell(size, col, line)
}

/// The caret nearest to world point `p`: the line under it and the nearest
/// cell boundary on that line, clamped into the text. A non-finite `p` or
/// size gives 0.
#[must_use]
pub fn caret_from_point(origin: Vec2, text: &str, size: f32, p: Vec2) -> usize {
    let size = em(size);
    let local = p - origin;
    if size <= 0.0 || !local.is_finite() {
        return 0;
    }
    // Float-to-int casts saturate: negative values become 0.
    let line = (local.y / line_height(size)).floor() as usize;
    let col = (local.x / (ADVANCE * size)).round() as usize;
    caret_index(text, line, col)
}

/// Every character to draw except line breaks, in text order, at its cell.
pub fn glyphs(origin: Vec2, text: &str, size: f32) -> impl Iterator<Item = Glyph> + '_ {
    let baseline = Vec2::new(0.0, BASELINE * em(size));
    lines(text).enumerate().flat_map(move |(line, chars)| {
        chars.chars().enumerate().map(move |(col, ch)| Glyph {
            ch,
            pos: origin + cell(size, col, line) + baseline,
        })
    })
}

/// Every line with the byte index it starts at.
fn line_spans(text: &str) -> impl Iterator<Item = (usize, &str)> {
    lines(text).scan(0, |start, line| {
        let span = (*start, line);
        *start += line.len() + 1;
        Some(span)
    })
}

/// `size` if it is finite and positive, else 0.
fn em(size: f32) -> f32 {
    if size.is_finite() && size > 0.0 {
        size
    } else {
        0.0
    }
}

/// Offset of cell `(col, line)` from the origin.
fn cell(size: f32, col: usize, line: usize) -> Vec2 {
    let size = em(size);
    Vec2::new(
        col as f32 * ADVANCE * size,
        line as f32 * LINE_HEIGHT * size,
    )
}

/// `index` clamped to the text and moved back to a `char` boundary.
pub(crate) fn floor_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use crate::core::snap::Targets;
    use crate::core::tex;
    use proptest::prelude::*;
    use std::rc::Rc;

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

    // ---- T30 formulas --------------------------------------------------------

    const FRAC: &str = "$\\frac{a}{b}$";

    #[test]
    fn layout_bounds_use_the_typeset_size_with_formulas() {
        // Arrange
        let typeset = tex::layout(&tex::parse(FRAC));

        // Act
        let b = bounds(v(10.0, 20.0), FRAC, 10.0);

        // Assert: narrower than its 13-char source, taller than one line.
        assert!(b.min.approx_eq(v(10.0, 20.0), EPS), "{b:?}");
        let size = v(typeset.width * 10.0, typeset.height * 10.0);
        assert!(b.max.approx_eq(v(10.0, 20.0) + size, EPS), "{b:?}");
        assert!(b.width() < 13.0 * 6.0 && b.height() > 12.5, "{b:?}");
    }

    #[test]
    fn layout_source_bounds_ignore_formulas() {
        // Act
        let b = source_bounds(v(0.0, 0.0), FRAC, 10.0);

        // Assert: 13 cells of 6, one line of 12.5.
        assert!(b.max.approx_eq(v(78.0, 12.5), EPS), "{b:?}");
        assert!(
            source_bounds(v(1.0, 2.0), "ab", 10.0)
                .max
                .approx_eq(bounds(v(1.0, 2.0), "ab", 10.0).max, EPS)
        );
    }

    #[test]
    fn layout_text_shape_hits_and_snaps_on_the_typeset_box() {
        // Arrange
        let shape = Shape::Text {
            origin: v(0.0, 0.0),
            text: FRAC.to_owned(),
            size: 10.0,
            style: Style {
                color: ColorId::INK,
                width: 0.0,
            },
        };
        let b = bounds(v(0.0, 0.0), FRAC, 10.0);

        // Act / Assert: bounds, hit-test and snap targets follow the box.
        assert!(shape.bounds().max.approx_eq(b.max, EPS));
        let y = b.center().y;
        assert!(shape.hit(v(b.max.x - 1.0, y), 0.5));
        // Inside the source box, right of the typeset one.
        assert!(!shape.hit(v(b.max.x + 20.0, y), 0.5));
        let targets = Targets::from_shapes([&shape]);
        assert!(targets.xs.iter().any(|a| approx_eq(a.value, b.max.x, EPS)));
        assert!(targets.ys.iter().any(|a| approx_eq(a.value, b.max.y, EPS)));
    }

    #[test]
    fn layout_typeset_is_cached_per_text() {
        // Act
        let first = typeset("$x^2$");
        let again = typeset("$x^2$");
        let other = typeset("$x^3$");

        // Assert: one layout per text, recomputed only for a new text.
        assert!(Rc::ptr_eq(&first, &again));
        assert!(!Rc::ptr_eq(&first, &other));
        assert_eq!(*first, tex::layout(&tex::parse("$x^2$")));
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
            let b = source_bounds(origin, &text, size).expand(1e-2);

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
