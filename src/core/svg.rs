//! The drawing as an SVG file (ADR-T114-2).
//!
//! [`to_svg`] is pure: it takes the shapes bottom to top and a source of
//! glyph outlines, and returns the text of the file. The world is the SVG's
//! user space (one world unit is one user unit), text and formulas go
//! through the same layout the renderer uses and are written as outlines,
//! each glyph defined once and reused.

use crate::core::geom::Vec2;
use crate::core::palette::Rgba;
use crate::core::shape::Shape;
use crate::core::tex::Face;

/// The outline of one character, as an SVG path.
#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
    /// Path data in font units, y down, so ink above the baseline has a
    /// negative y.
    pub path: String,
    /// Font units per em.
    pub units_per_em: f32,
    /// Advance width in font units.
    pub advance: f32,
    /// Height of the ink above the baseline in font units.
    pub ink_top: f32,
}

/// Where the glyph outlines come from: the embedded fonts, in the shell.
pub trait Outlines {
    /// The outline of `ch` in `face`; `None` for a character without ink
    /// (a space) or one the font lacks.
    fn outline(&self, face: Face, ch: char) -> Option<Outline>;
}

/// `shapes`, bottom to top, as an SVG document on a `background` canvas.
/// Non-finite shapes are skipped, so every number in it is finite.
#[must_use]
pub fn to_svg<'a>(
    shapes: impl IntoIterator<Item = &'a Shape>,
    background: Rgba,
    outlines: &dyn Outlines,
) -> String {
    let _ = (shapes, background, outlines);
    todo!("write the document")
}

/// `value` with at most 2 decimals and no trailing zeros; 0 when it is not
/// finite or rounds to zero.
#[must_use]
pub fn number(value: f32) -> String {
    let _ = value;
    todo!("format a number")
}

/// Whether `xml` is one well-formed element tree: balanced and matching
/// tags, quoted and unique attributes. A small check for tests; it knows
/// no entities or comments beyond `<?…?>` declarations.
#[must_use]
pub fn is_well_formed(xml: &str) -> bool {
    let _ = xml;
    todo!("check the markup")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::palette::ColorId;
    use crate::core::shape::{CellFill, ShapeText, Style};
    use crate::theme;

    /// A square glyph, 1000 units per em.
    struct Squares;

    impl Outlines for Squares {
        fn outline(&self, _face: Face, ch: char) -> Option<Outline> {
            (!ch.is_whitespace()).then(|| Outline {
                path: "M0 0L500 0L500 -700L0 -700Z".to_owned(),
                units_per_em: 1000.0,
                advance: 600.0,
                ink_top: 700.0,
            })
        }
    }

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn style(width: f32) -> Style {
        Style {
            color: ColorId::INK,
            width,
        }
    }

    fn render(shapes: &[Shape]) -> String {
        to_svg(shapes, Rgba::from_hex(theme::CANVAS), &Squares)
    }

    fn count(svg: &str, pattern: &str) -> usize {
        svg.matches(pattern).count()
    }

    fn text(origin: Vec2, content: &str) -> Shape {
        Shape::Text {
            origin,
            text: content.to_owned(),
            size: 20.0,
            style: style(0.0),
        }
    }

    fn rect(label: Option<u32>, fill: Option<ColorId>) -> Shape {
        Shape::Rect {
            a: v(0.0, 0.0),
            b: v(100.0, 60.0),
            style: style(2.0),
            fill,
            label,
            text: None,
        }
    }

    // ---- AC-4: the file ------------------------------------------------

    #[test]
    fn empty_document_is_valid_svg() {
        let svg = render(&[]);

        assert!(svg.starts_with("<svg"), "{svg}");
        assert!(svg.trim_end().ends_with("</svg>"), "{svg}");
        assert!(is_well_formed(&svg), "{svg}");
        assert!(!svg.contains("NaN") && !svg.contains("inf"), "{svg}");
    }

    #[test]
    fn to_svg_has_canvas_background_and_viewbox_of_all_shapes() {
        let shapes = [
            Shape::Line {
                a: v(0.0, 0.0),
                b: v(100.0, 50.0),
                style: style(2.0),
            },
            Shape::Line {
                a: v(-200.0, 10.0),
                b: v(10.0, 400.0),
                style: style(2.0),
            },
        ];

        let svg = render(&shapes);

        assert!(svg.contains("#181616"), "background colour: {svg}");
        let view = svg
            .split("viewBox=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or_default();
        let n: Vec<f32> = view.split(' ').filter_map(|s| s.parse().ok()).collect();
        assert_eq!(n.len(), 4, "viewBox {view}");
        assert!(n[0] <= -200.0 && n[1] <= 0.0, "origin {view}");
        assert!(n[0] + n[2] >= 100.0 && n[1] + n[3] >= 400.0, "extent {view}");
    }

    #[test]
    fn non_finite_shapes_are_skipped() {
        let shapes = [
            Shape::Line {
                a: v(f32::NAN, 0.0),
                b: v(1.0, f32::INFINITY),
                style: style(2.0),
            },
            Shape::Line {
                a: v(0.0, 0.0),
                b: v(10.0, 10.0),
                style: style(2.0),
            },
        ];

        let svg = render(&shapes);

        assert_eq!(count(&svg, "<line"), 1, "{svg}");
        assert!(!svg.contains("NaN") && !svg.contains("inf"), "{svg}");
        assert!(is_well_formed(&svg));
    }

    #[test]
    fn numbers_have_two_decimals_and_no_trailing_zeros() {
        assert_eq!(number(1.0), "1");
        assert_eq!(number(0.5), "0.5");
        assert_eq!(number(1.234), "1.23");
        assert_eq!(number(-2.5), "-2.5");
        assert_eq!(number(-0.001), "0");
        assert_eq!(number(f32::NAN), "0");
    }

    #[test]
    fn is_well_formed_accepts_and_rejects() {
        assert!(is_well_formed("<a b=\"1\"><c/></a>"));
        assert!(is_well_formed("<?xml version=\"1.0\"?>\n<a>\n</a>\n"));
        assert!(!is_well_formed("<a><b></a></b>"));
        assert!(!is_well_formed("<a>"));
        assert!(!is_well_formed("<a b=1/>"));
        assert!(!is_well_formed("<a/><b/>"));
        assert!(!is_well_formed("<a x=\"1\" x=\"2\"/>"));
        assert!(!is_well_formed(""));
    }

    // ---- AC-2: shapes --------------------------------------------------

    #[test]
    fn line_has_colour_width_and_round_caps() {
        let svg = render(&[Shape::Line {
            a: v(0.0, 0.0),
            b: v(100.0, 50.0),
            style: style(2.0),
        }]);

        assert!(svg.contains("<line x1=\"0\" y1=\"0\" x2=\"100\" y2=\"50\""), "{svg}");
        assert!(svg.contains("stroke=\"#c5c9c5\""), "{svg}");
        assert!(svg.contains("stroke-width=\"2\""), "{svg}");
        assert!(svg.contains("stroke-linecap=\"round\""), "{svg}");
    }

    #[test]
    fn arrow_has_a_shaft_and_a_filled_head() {
        let svg = render(&[Shape::Arrow {
            a: v(0.0, 0.0),
            b: v(100.0, 0.0),
            style: style(2.0),
        }]);

        assert_eq!(count(&svg, "<line"), 1, "{svg}");
        assert_eq!(count(&svg, "<polygon"), 1, "{svg}");
        assert!(is_well_formed(&svg));
    }

    #[test]
    fn rect_has_outline_and_fill() {
        let red = ColorId::new(1);
        let svg = render(&[rect(None, red)]);

        assert!(svg.contains("<rect x=\"0\" y=\"0\" width=\"100\" height=\"60\""), "{svg}");
        assert!(svg.contains("fill=\"#d16961\""), "{svg}");
        assert!(svg.contains("stroke=\"#c5c9c5\""), "{svg}");
    }

    #[test]
    fn rect_without_fill_has_fill_none() {
        let svg = render(&[rect(None, None)]);

        assert!(svg.contains("fill=\"none\""), "{svg}");
    }

    #[test]
    fn ellipse_has_centre_and_radii() {
        let svg = render(&[Shape::Ellipse {
            a: v(0.0, 0.0),
            b: v(100.0, 60.0),
            style: style(2.0),
            fill: None,
            label: None,
            text: None,
        }]);

        assert!(
            svg.contains("<ellipse cx=\"50\" cy=\"30\" rx=\"50\" ry=\"30\""),
            "{svg}"
        );
    }

    #[test]
    fn label_on_a_rect_is_drawn_as_glyphs() {
        let svg = render(&[rect(Some(12), None)]);

        assert_eq!(count(&svg, "<use"), 2, "two digits: {svg}");
    }

    #[test]
    fn grid_has_lines_cell_fills_and_index_bands() {
        let grid = Shape::Grid {
            a: v(0.0, 0.0),
            b: v(300.0, 200.0),
            cols: 3,
            rows: 2,
            style: style(2.0),
            axes: true,
            fills: vec![CellFill {
                col: 1,
                row: 0,
                color: ColorId::INK,
            }],
            texts: Vec::new(),
        };

        let svg = render(&[grid]);

        assert_eq!(count(&svg, "<line"), (3 + 1) + (2 + 1), "{svg}");
        assert_eq!(count(&svg, "<rect"), 2, "background and one filled cell: {svg}");
        // 0, 1, 2 over the columns and 0, 1 beside the rows.
        assert_eq!(count(&svg, "<use"), 5, "{svg}");
    }

    #[test]
    fn grid_cell_text_is_drawn() {
        let content = ShapeText::new("ab".to_owned(), 20.0);
        let grid = Shape::Grid {
            a: v(0.0, 0.0),
            b: v(200.0, 100.0),
            cols: 2,
            rows: 1,
            style: style(2.0),
            axes: false,
            fills: Vec::new(),
            texts: content
                .map(|content| crate::core::shape::CellText {
                    col: 1,
                    row: 0,
                    content,
                })
                .into_iter()
                .collect(),
        };

        let svg = render(&[grid]);

        assert_eq!(count(&svg, "<use"), 2, "{svg}");
    }

    #[test]
    fn stroke_is_a_round_polyline() {
        let svg = render(&[Shape::Stroke {
            points: vec![v(0.0, 0.0), v(10.0, 5.0), v(20.0, 0.0)],
            widths: None,
            style: style(3.0),
        }]);

        assert!(svg.contains("<polyline points=\"0,0 10,5 20,0\""), "{svg}");
        assert!(svg.contains("stroke-linejoin=\"round\""), "{svg}");
        assert!(svg.contains("fill=\"none\""), "{svg}");
    }

    #[test]
    fn single_point_stroke_is_a_dot() {
        let svg = render(&[Shape::Stroke {
            points: vec![v(4.0, 5.0)],
            widths: None,
            style: style(3.0),
        }]);

        assert!(svg.contains("<circle cx=\"4\" cy=\"5\" r=\"1.5\""), "{svg}");
    }

    #[test]
    fn pressure_stroke_is_one_line_per_segment() {
        let svg = render(&[Shape::Stroke {
            points: vec![v(0.0, 0.0), v(10.0, 0.0), v(20.0, 0.0)],
            widths: Some(vec![0.5, 1.0, 1.5]),
            style: style(2.0),
        }]);

        assert_eq!(count(&svg, "<line"), 2, "{svg}");
        assert!(svg.contains("stroke-width=\"1.5\""), "mean of 0.5 and 1.0: {svg}");
    }

    // ---- AC-3: glyph outlines ------------------------------------------

    #[test]
    fn text_glyphs_are_defined_once_and_reused() {
        let svg = render(&[text(v(0.0, 0.0), "aaa")]);

        assert_eq!(count(&svg, "<path id="), 1, "{svg}");
        assert_eq!(count(&svg, "<use"), 3, "{svg}");
        assert!(svg.contains("<defs>"), "{svg}");
        assert!(is_well_formed(&svg));
    }

    #[test]
    fn text_with_different_characters_defines_each_once() {
        let svg = render(&[text(v(0.0, 0.0), "ab\nba")]);

        assert_eq!(count(&svg, "<path id="), 2, "{svg}");
        assert_eq!(count(&svg, "<use"), 4, "{svg}");
    }

    #[test]
    fn text_spaces_draw_nothing() {
        let svg = render(&[text(v(0.0, 0.0), "a b")]);

        assert_eq!(count(&svg, "<use"), 2, "{svg}");
    }

    #[test]
    fn text_glyph_is_placed_at_its_baseline_and_scaled_to_the_size() {
        // Size 20, 1000 units per em: scale 0.02; baseline at 1.0 em.
        let svg = render(&[text(v(10.0, 100.0), "a")]);

        assert!(
            svg.contains("transform=\"translate(10 120) scale(0.02)\""),
            "{svg}"
        );
    }

    #[test]
    fn formula_fraction_bar_is_a_line() {
        let svg = render(&[text(v(0.0, 0.0), "$\\frac{a}{b}$")]);

        assert!(count(&svg, "<line") >= 1, "{svg}");
        assert!(count(&svg, "<use") >= 2, "{svg}");
        assert!(is_well_formed(&svg));
    }

    #[test]
    fn shape_text_is_drawn_inside_the_shape() {
        let shape = Shape::Rect {
            a: v(0.0, 0.0),
            b: v(200.0, 100.0),
            style: style(2.0),
            fill: None,
            label: None,
            text: ShapeText::new("hi".to_owned(), 30.0),
        };

        let svg = render(&[shape]);

        assert_eq!(count(&svg, "<use"), 2, "{svg}");
    }

    #[test]
    fn text_on_a_filled_shape_takes_the_background_colour() {
        let shape = Shape::Rect {
            a: v(0.0, 0.0),
            b: v(200.0, 100.0),
            style: style(2.0),
            fill: ColorId::new(1),
            label: None,
            text: ShapeText::new("hi".to_owned(), 30.0),
        };

        let svg = render(&[shape]);

        assert!(svg.contains("fill=\"#181616\""), "{svg}");
    }

    #[test]
    fn shapes_keep_their_stacking_order() {
        let svg = render(&[
            Shape::Line {
                a: v(0.0, 0.0),
                b: v(1.0, 1.0),
                style: style(2.0),
            },
            rect(None, None),
        ]);

        let line = svg.find("<line");
        let rect = svg.rfind("<rect");
        assert!(line < rect, "{svg}");
    }
}
