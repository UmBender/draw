//! Vector shapes with bounds, hit-testing and translation.
//!
//! The document is an ordered list of [`Shape`]s in world coordinates
//! (ADR-0004). Widths are world units (ADR-0013). Every query here is total:
//! non-finite input never panics, a non-finite query point never hits and is
//! never contained.

use crate::core::geom::{Aabb, Vec2};
use crate::core::palette::ColorId;

/// Arrow head length per unit of stroke width.
pub const ARROW_HEAD_LENGTH_PER_WIDTH: f32 = 4.0;

/// Smallest arrow head length in world units, so thin arrows keep a visible head.
pub const ARROW_HEAD_MIN_LENGTH: f32 = 8.0;

/// Half the width of the arrow head base, as a fraction of the head length.
pub const ARROW_HEAD_HALF_WIDTH_RATIO: f32 = 0.5;

/// Outline style shared by every shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Outline colour.
    pub color: ColorId,
    /// Outline width in world units.
    pub width: f32,
}

/// A vector shape in world coordinates.
///
/// For [`Shape::Rect`] and [`Shape::Ellipse`], `a` and `b` are opposite corners
/// of the bounding box, in any order.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// Freehand polyline.
    Stroke {
        /// Polyline vertices in drawing order.
        points: Vec<Vec2>,
        /// Outline style.
        style: Style,
    },
    /// Straight segment from `a` to `b`.
    Line {
        /// Start point.
        a: Vec2,
        /// End point.
        b: Vec2,
        /// Outline style.
        style: Style,
    },
    /// Segment from `a` to `b` with a filled head at `b`.
    Arrow {
        /// Tail.
        a: Vec2,
        /// Tip (where the head is drawn).
        b: Vec2,
        /// Outline style.
        style: Style,
    },
    /// Axis-aligned rectangle.
    Rect {
        /// One corner.
        a: Vec2,
        /// The opposite corner.
        b: Vec2,
        /// Outline style.
        style: Style,
        /// Interior colour, if filled.
        fill: Option<ColorId>,
    },
    /// Axis-aligned ellipse inscribed in the box spanned by `a` and `b`.
    Ellipse {
        /// One corner of the bounding box.
        a: Vec2,
        /// The opposite corner of the bounding box.
        b: Vec2,
        /// Outline style.
        style: Style,
        /// Interior colour, if filled.
        fill: Option<ColorId>,
    },
}

impl Shape {
    /// The outline style.
    #[must_use]
    pub fn style(&self) -> Style {
        todo!()
    }

    /// The fill colour; always `None` for open shapes.
    #[must_use]
    pub fn fill(&self) -> Option<ColorId> {
        todo!()
    }

    /// `true` for shapes with an interior ([`Shape::Rect`], [`Shape::Ellipse`]).
    #[must_use]
    pub fn is_closed(&self) -> bool {
        todo!()
    }

    /// Bounding box including half the outline width (and the arrow head).
    ///
    /// A stroke with no finite point gives the zero-size box at the origin.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        todo!()
    }

    /// `true` if `p` is within `tol + width / 2` of the outline, or inside a
    /// filled closed shape.
    #[must_use]
    pub fn hit(&self, p: Vec2, tol: f32) -> bool {
        let _ = (p, tol);
        todo!()
    }

    /// `true` if `p` lies inside the geometry of a closed shape (boundary
    /// inclusive, outline width and fill ignored). Open shapes contain nothing.
    #[must_use]
    pub fn contains(&self, p: Vec2) -> bool {
        let _ = p;
        todo!()
    }

    /// Moves every point by `delta`.
    pub fn translate(&mut self, delta: Vec2) {
        let _ = delta;
        todo!()
    }

    /// Returns the shape with its fill set to `fill`; open shapes are
    /// returned unchanged.
    #[must_use]
    pub fn with_fill(self, fill: Option<ColorId>) -> Self {
        let _ = fill;
        todo!()
    }

    /// `true` iff every coordinate and the width are finite.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        todo!()
    }
}

/// Arrow head triangle `[tip, left, right]` for an arrow from `a` to `b`.
///
/// The tip is `b`; the base lies `max(ARROW_HEAD_MIN_LENGTH,
/// ARROW_HEAD_LENGTH_PER_WIDTH * width)` back along the shaft and is
/// `2 * ARROW_HEAD_HALF_WIDTH_RATIO` times that length wide. A degenerate
/// arrow (`a` ≈ `b`) gives `[b, b, b]`.
#[must_use]
pub fn arrow_head(a: Vec2, b: Vec2, width: f32) -> [Vec2; 3] {
    let _ = (a, b, width);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Aabb, Vec2, approx_eq};
    use crate::core::palette::ColorId;
    use proptest::prelude::*;

    const EPS: f32 = 1e-4;

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn color(index: u8) -> ColorId {
        ColorId::new(index).unwrap_or(ColorId::INK)
    }

    fn style(width: f32) -> Style {
        Style {
            color: ColorId::INK,
            width,
        }
    }

    fn assert_vec_eq(actual: Vec2, expected: Vec2) {
        assert!(
            actual.approx_eq(expected, EPS),
            "expected {expected:?}, got {actual:?}"
        );
    }

    fn assert_aabb_eq(actual: Aabb, min: Vec2, max: Vec2) {
        assert_vec_eq(actual.min, min);
        assert_vec_eq(actual.max, max);
    }

    fn rect(a: Vec2, b: Vec2, width: f32, fill: Option<ColorId>) -> Shape {
        Shape::Rect {
            a,
            b,
            style: style(width),
            fill,
        }
    }

    fn ellipse(a: Vec2, b: Vec2, width: f32, fill: Option<ColorId>) -> Shape {
        Shape::Ellipse {
            a,
            b,
            style: style(width),
            fill,
        }
    }

    fn stroke(points: &[Vec2], width: f32) -> Shape {
        Shape::Stroke {
            points: points.to_vec(),
            style: style(width),
        }
    }

    // ---- AC-1: construction and accessors ----

    #[test]
    fn construct_stroke_keeps_points_and_style() {
        let s = Shape::Stroke {
            points: vec![v(0.0, 0.0), v(1.0, 2.0)],
            style: Style {
                color: color(2),
                width: 3.0,
            },
        };

        let Shape::Stroke { points, style } = s else {
            panic!("expected Stroke");
        };
        assert_eq!(points.len(), 2);
        assert_vec_eq(points[1], v(1.0, 2.0));
        assert_eq!(style.color, color(2));
        assert!(approx_eq(style.width, 3.0, EPS));
    }

    #[test]
    fn construct_line_and_arrow_keep_endpoints() {
        let line = Shape::Line {
            a: v(1.0, 1.0),
            b: v(4.0, 5.0),
            style: style(2.0),
        };
        let arrow = Shape::Arrow {
            a: v(-1.0, 0.0),
            b: v(0.0, 7.0),
            style: style(2.0),
        };

        let Shape::Line { a, b, .. } = line else {
            panic!("expected Line");
        };
        assert_vec_eq(a, v(1.0, 1.0));
        assert_vec_eq(b, v(4.0, 5.0));
        let Shape::Arrow { a, b, .. } = arrow else {
            panic!("expected Arrow");
        };
        assert_vec_eq(a, v(-1.0, 0.0));
        assert_vec_eq(b, v(0.0, 7.0));
    }

    #[test]
    fn construct_rect_and_ellipse_keep_corners_and_fill() {
        let r = rect(v(0.0, 0.0), v(2.0, 3.0), 1.0, Some(color(3)));
        let e = ellipse(v(5.0, 5.0), v(1.0, 1.0), 1.0, None);

        let Shape::Rect { a, b, fill, .. } = r else {
            panic!("expected Rect");
        };
        assert_vec_eq(a, v(0.0, 0.0));
        assert_vec_eq(b, v(2.0, 3.0));
        assert_eq!(fill, Some(color(3)));
        let Shape::Ellipse { a, b, fill, .. } = e else {
            panic!("expected Ellipse");
        };
        assert_vec_eq(a, v(5.0, 5.0));
        assert_vec_eq(b, v(1.0, 1.0));
        assert_eq!(fill, None);
    }

    #[test]
    fn accessors_report_style_fill_and_closedness() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: Style {
                color: color(1),
                width: 4.0,
            },
        };
        let filled = rect(v(0.0, 0.0), v(1.0, 1.0), 2.0, Some(color(4)));
        let hollow = ellipse(v(0.0, 0.0), v(1.0, 1.0), 2.0, None);

        assert_eq!(line.style().color, color(1));
        assert!(approx_eq(line.style().width, 4.0, EPS));
        assert_eq!(line.fill(), None);
        assert!(!line.is_closed());
        assert!(!stroke(&[v(0.0, 0.0)], 1.0).is_closed());
        assert_eq!(filled.fill(), Some(color(4)));
        assert!(filled.is_closed());
        assert_eq!(hollow.fill(), None);
        assert!(hollow.is_closed());
    }

    // ---- AC-2: bounds ----

    #[test]
    fn bounds_stroke_includes_half_width() {
        let s = stroke(&[v(0.0, 0.0), v(4.0, -2.0), v(1.0, 3.0)], 2.0);

        assert_aabb_eq(s.bounds(), v(-1.0, -3.0), v(5.0, 4.0));
    }

    #[test]
    fn bounds_empty_stroke_is_point_at_origin() {
        let empty = stroke(&[], 2.0);
        let all_nan = stroke(&[v(f32::NAN, 1.0)], 2.0);

        assert_aabb_eq(empty.bounds(), Vec2::ZERO, Vec2::ZERO);
        assert_aabb_eq(all_nan.bounds(), Vec2::ZERO, Vec2::ZERO);
    }

    #[test]
    fn bounds_line_includes_half_width() {
        let line = Shape::Line {
            a: v(3.0, 1.0),
            b: v(-1.0, 2.0),
            style: style(1.0),
        };

        assert_aabb_eq(line.bounds(), v(-1.5, 0.5), v(3.5, 2.5));
    }

    #[test]
    fn bounds_arrow_includes_head() {
        let (a, b, width) = (v(0.0, 0.0), v(100.0, 0.0), 2.0);
        let arrow = Shape::Arrow {
            a,
            b,
            style: style(width),
        };
        let head = arrow_head(a, b, width);

        let bounds = arrow.bounds();

        let half_head = ARROW_HEAD_HALF_WIDTH_RATIO * ARROW_HEAD_MIN_LENGTH;
        assert!(half_head > 1.0, "head must stick out past the half width");
        assert_aabb_eq(bounds, v(-1.0, -half_head - 1.0), v(101.0, half_head + 1.0));
        for p in head {
            assert!(bounds.contains(p), "{p:?} outside {bounds:?}");
        }
    }

    #[test]
    fn bounds_rect_is_corner_order_independent() {
        let r1 = rect(v(0.0, 0.0), v(4.0, 2.0), 2.0, None);
        let r2 = rect(v(4.0, 0.0), v(0.0, 2.0), 2.0, None);

        assert_aabb_eq(r1.bounds(), v(-1.0, -1.0), v(5.0, 3.0));
        assert_aabb_eq(r2.bounds(), v(-1.0, -1.0), v(5.0, 3.0));
    }

    #[test]
    fn bounds_ellipse_matches_bounding_box() {
        let e = ellipse(v(10.0, 4.0), v(2.0, -4.0), 4.0, Some(color(1)));

        assert_aabb_eq(e.bounds(), v(0.0, -6.0), v(12.0, 6.0));
    }

    // ---- AC-3: hit ----

    #[test]
    fn hit_stroke_near_segment_is_true() {
        let s = stroke(&[v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)], 2.0);

        // reach = tol 1 + half width 1 = 2
        assert!(s.hit(v(5.0, 1.9), 1.0));
        assert!(s.hit(v(11.9, 5.0), 1.0));
        assert!(s.hit(v(10.0, 10.0), 0.0));
    }

    #[test]
    fn hit_stroke_far_is_false() {
        let s = stroke(&[v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)], 2.0);

        assert!(!s.hit(v(5.0, 2.1), 1.0));
        assert!(!s.hit(v(5.0, 5.0), 1.0));
        assert!(!s.hit(v(-3.0, 0.0), 1.0));
    }

    #[test]
    fn hit_single_point_stroke_uses_distance_to_point() {
        let dot = stroke(&[v(3.0, 4.0)], 2.0);

        assert!(dot.hit(v(3.0, 4.0), 0.0));
        assert!(dot.hit(v(3.0, 5.9), 1.0));
        assert!(!dot.hit(v(3.0, 6.1), 1.0));
    }

    #[test]
    fn hit_empty_stroke_is_false() {
        let empty = stroke(&[], 2.0);

        assert!(!empty.hit(Vec2::ZERO, 100.0));
    }

    #[test]
    fn hit_line_edge_within_tolerance_plus_half_width() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(10.0, 0.0),
            style: style(4.0),
        };

        // reach = 0.5 + 2 = 2.5
        assert!(line.hit(v(5.0, 2.4), 0.5));
        assert!(line.hit(v(5.0, -2.4), 0.5));
        assert!(!line.hit(v(5.0, 2.6), 0.5));
        assert!(line.hit(v(12.4, 0.0), 0.5));
        assert!(!line.hit(v(12.6, 0.0), 0.5));
    }

    #[test]
    fn hit_arrow_shaft_and_head() {
        let (a, b, width) = (v(0.0, 0.0), v(100.0, 0.0), 1.0);
        let arrow = Shape::Arrow {
            a,
            b,
            style: style(width),
        };
        let [_, left, _] = arrow_head(a, b, width);

        assert!(arrow.hit(v(50.0, 0.4), 0.0), "shaft");
        assert!(!arrow.hit(v(50.0, 2.0), 0.0), "beside shaft");
        assert!(arrow.hit(left, 0.0), "head corner");
        let inside_head = v(100.0 - ARROW_HEAD_MIN_LENGTH * 0.5, left.y * 0.4);
        assert!(arrow.hit(inside_head, 0.0), "inside head");
        assert!(!arrow.hit(v(104.0, 0.0), 0.0), "beyond tip");
    }

    #[test]
    fn hit_rect_outline_inside_outside() {
        let r = rect(v(0.0, 0.0), v(10.0, 10.0), 2.0, None);

        assert!(r.hit(v(0.0, 5.0), 0.0), "on edge");
        assert!(r.hit(v(11.5, 5.0), 0.5), "just outside, within reach");
        assert!(r.hit(v(5.0, 8.5), 0.5), "just inside, within reach");
        assert!(!r.hit(v(5.0, 5.0), 0.5), "hollow interior");
        assert!(!r.hit(v(12.0, 5.0), 0.5), "outside");
    }

    #[test]
    fn hit_filled_rect_interior_is_true() {
        let r = rect(v(0.0, 0.0), v(10.0, 10.0), 2.0, Some(color(2)));

        assert!(r.hit(v(5.0, 5.0), 0.0));
        assert!(!r.hit(v(12.0, 5.0), 0.5));
    }

    #[test]
    fn hit_ellipse_outline_inside_outside() {
        // centre (0, 0), rx = 10, ry = 5
        let e = ellipse(v(-10.0, -5.0), v(10.0, 5.0), 2.0, None);
        let on_outline = v(10.0 * 0.6, 5.0 * 0.8);

        assert!(e.hit(on_outline, 0.0), "on outline");
        assert!(e.hit(v(11.9, 0.0), 1.0), "outside on x axis, within reach");
        assert!(e.hit(v(0.0, 3.1), 1.0), "inside on y axis, within reach");
        assert!(!e.hit(v(12.1, 0.0), 1.0), "outside on x axis");
        assert!(!e.hit(v(0.0, 2.9), 1.0), "inside on y axis");
        assert!(!e.hit(Vec2::ZERO, 1.0), "hollow centre");
        assert!(!e.hit(v(10.0, 5.0), 1.0), "box corner");
    }

    #[test]
    fn hit_filled_ellipse_interior_is_true() {
        let e = ellipse(v(-10.0, -5.0), v(10.0, 5.0), 2.0, Some(color(5)));

        assert!(e.hit(Vec2::ZERO, 0.0));
        assert!(!e.hit(v(10.0, 5.0), 1.0), "box corner is outside");
    }

    #[test]
    fn hit_degenerate_ellipse_is_segment() {
        let flat = ellipse(v(0.0, 2.0), v(10.0, 2.0), 2.0, None);

        assert!(flat.hit(v(5.0, 3.5), 0.5));
        assert!(!flat.hit(v(5.0, 4.0), 0.5));
        assert!(!flat.hit(v(11.6, 2.0), 0.5));
    }

    #[test]
    fn hit_non_finite_point_is_false() {
        let filled = rect(v(0.0, 0.0), v(10.0, 10.0), 2.0, Some(color(1)));
        let s = stroke(&[v(0.0, 0.0), v(1.0, 1.0)], 1.0);

        for p in [
            v(f32::NAN, 5.0),
            v(5.0, f32::INFINITY),
            v(f32::NEG_INFINITY, 0.0),
        ] {
            assert!(!filled.hit(p, 1.0), "{p:?}");
            assert!(!s.hit(p, f32::MAX), "{p:?}");
        }
        assert!(!s.hit(v(0.0, 0.0), f32::NAN));
    }

    // ---- AC-4: contains ----

    #[test]
    fn contains_rect_inside_and_boundary() {
        let r = rect(v(10.0, 10.0), v(0.0, 0.0), 2.0, None);

        assert!(r.contains(v(5.0, 5.0)));
        assert!(r.contains(v(0.0, 0.0)));
        assert!(r.contains(v(10.0, 3.0)));
    }

    #[test]
    fn contains_rect_outside_is_false() {
        let r = rect(v(0.0, 0.0), v(10.0, 10.0), 2.0, None);

        assert!(!r.contains(v(10.5, 5.0)), "within stroke but outside geometry");
        assert!(!r.contains(v(-1.0, -1.0)));
        assert!(!r.contains(v(f32::NAN, 5.0)));
    }

    #[test]
    fn contains_ellipse_inside_and_outside() {
        let e = ellipse(v(-10.0, -5.0), v(10.0, 5.0), 2.0, None);

        assert!(e.contains(Vec2::ZERO));
        assert!(e.contains(v(9.9, 0.0)));
        assert!(e.contains(v(0.0, -4.9)));
        assert!(!e.contains(v(10.1, 0.0)));
        assert!(!e.contains(v(9.0, 4.0)), "inside box, outside ellipse");
        assert!(!e.contains(v(0.0, f32::NAN)));
    }

    #[test]
    fn contains_degenerate_ellipse_is_false() {
        let flat = ellipse(v(0.0, 0.0), v(10.0, 0.0), 2.0, Some(color(1)));
        let point = ellipse(v(3.0, 3.0), v(3.0, 3.0), 2.0, Some(color(1)));

        assert!(!flat.contains(v(5.0, 0.0)));
        assert!(!point.contains(v(3.0, 3.0)));
    }

    #[test]
    fn contains_open_shapes_is_false() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(10.0, 10.0),
            style: style(2.0),
        };
        let arrow = Shape::Arrow {
            a: v(0.0, 0.0),
            b: v(10.0, 10.0),
            style: style(2.0),
        };
        let s = stroke(&[v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0), v(0.0, 0.0)], 2.0);

        assert!(!line.contains(v(5.0, 5.0)));
        assert!(!arrow.contains(v(5.0, 5.0)));
        assert!(!s.contains(v(7.0, 3.0)));
    }

    // ---- AC-5: translate and with_fill ----

    #[test]
    fn translate_moves_every_point() {
        let d = v(3.0, -2.0);
        let mut s = stroke(&[v(0.0, 0.0), v(1.0, 1.0)], 2.0);
        let mut line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: style(2.0),
        };
        let mut arrow = Shape::Arrow {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: style(2.0),
        };
        let mut r = rect(v(0.0, 0.0), v(2.0, 2.0), 2.0, Some(color(2)));
        let mut e = ellipse(v(0.0, 0.0), v(2.0, 2.0), 2.0, None);

        for shape in [&mut s, &mut line, &mut arrow, &mut r, &mut e] {
            shape.translate(d);
        }

        assert_eq!(
            s,
            stroke(&[v(3.0, -2.0), v(4.0, -1.0)], 2.0),
            "stroke points move"
        );
        let Shape::Line { a, b, .. } = line else {
            panic!("expected Line");
        };
        assert_vec_eq(a, v(3.0, -2.0));
        assert_vec_eq(b, v(4.0, -2.0));
        let Shape::Arrow { a, b, .. } = arrow else {
            panic!("expected Arrow");
        };
        assert_vec_eq(a, v(3.0, -2.0));
        assert_vec_eq(b, v(4.0, -2.0));
        assert_eq!(r, rect(v(3.0, -2.0), v(5.0, 0.0), 2.0, Some(color(2))));
        assert_eq!(e, ellipse(v(3.0, -2.0), v(5.0, 0.0), 2.0, None));
    }

    #[test]
    fn with_fill_sets_and_clears_fill_on_closed_shapes() {
        let r = rect(v(0.0, 0.0), v(1.0, 1.0), 1.0, None);
        let e = ellipse(v(0.0, 0.0), v(1.0, 1.0), 1.0, Some(color(1)));

        let filled = r.with_fill(Some(color(3)));
        let cleared = e.with_fill(None);

        assert_eq!(filled.fill(), Some(color(3)));
        assert_eq!(filled, rect(v(0.0, 0.0), v(1.0, 1.0), 1.0, Some(color(3))));
        assert_eq!(cleared.fill(), None);
        assert_eq!(cleared, ellipse(v(0.0, 0.0), v(1.0, 1.0), 1.0, None));
    }

    #[test]
    fn with_fill_on_open_shape_is_noop() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: style(1.0),
        };
        let s = stroke(&[v(0.0, 0.0)], 1.0);

        assert_eq!(line.clone().with_fill(Some(color(2))), line);
        assert_eq!(s.clone().with_fill(Some(color(2))), s);
    }

    // ---- AC-6: arrow_head ----

    #[test]
    fn arrow_head_tip_is_at_b() {
        let [tip, _, _] = arrow_head(v(0.0, 0.0), v(30.0, 40.0), 2.0);

        assert_vec_eq(tip, v(30.0, 40.0));
    }

    #[test]
    fn arrow_head_length_scales_with_width() {
        let width = 10.0;
        let length = ARROW_HEAD_LENGTH_PER_WIDTH * width;
        assert!(length > ARROW_HEAD_MIN_LENGTH, "test needs a wide arrow");

        let [_, left, right] = arrow_head(v(0.0, 0.0), v(100.0, 0.0), width);

        let half = ARROW_HEAD_HALF_WIDTH_RATIO * length;
        assert_vec_eq(left, v(100.0 - length, half));
        assert_vec_eq(right, v(100.0 - length, -half));
    }

    #[test]
    fn arrow_head_has_min_size() {
        let [tip, left, right] = arrow_head(v(0.0, 0.0), v(0.0, 50.0), 0.1);

        let base_mid = left.lerp(right, 0.5);
        assert!(approx_eq(tip.distance(base_mid), ARROW_HEAD_MIN_LENGTH, EPS));
        assert!(approx_eq(
            left.distance(right),
            2.0 * ARROW_HEAD_HALF_WIDTH_RATIO * ARROW_HEAD_MIN_LENGTH,
            EPS
        ));
    }

    #[test]
    fn arrow_head_is_symmetric_about_shaft() {
        let (a, b) = (v(-3.0, 7.0), v(20.0, -11.0));

        let [tip, left, right] = arrow_head(a, b, 3.0);

        assert!(approx_eq(tip.distance(left), tip.distance(right), EPS));
        let base_mid = left.lerp(right, 0.5);
        let shaft = b - a;
        let base = right - left;
        assert!(approx_eq(base.dot(shaft), 0.0, 1e-2), "base ⟂ shaft");
        let to_mid = base_mid - a;
        let cross = shaft.x * to_mid.y - shaft.y * to_mid.x;
        assert!(approx_eq(cross, 0.0, 1e-2), "base midpoint on shaft");
    }

    #[test]
    fn arrow_head_degenerate_collapses_to_tip() {
        let b = v(4.0, 4.0);

        for p in arrow_head(b, b, 2.0) {
            assert_vec_eq(p, b);
        }
    }

    // ---- AC-7: is_finite ----

    #[test]
    fn is_finite_true_for_finite_shapes() {
        assert!(stroke(&[], 1.0).is_finite());
        assert!(stroke(&[v(0.0, 0.0), v(1.0, 1.0)], 1.0).is_finite());
        assert!(rect(v(0.0, 0.0), v(1.0, 1.0), 1.0, Some(color(1))).is_finite());
        assert!(
            Shape::Arrow {
                a: v(0.0, 0.0),
                b: v(-1.0, 1.0),
                style: style(2.0)
            }
            .is_finite()
        );
    }

    #[test]
    fn is_finite_false_for_nan_or_inf_coordinate() {
        assert!(!stroke(&[v(0.0, 0.0), v(f32::NAN, 1.0)], 1.0).is_finite());
        assert!(
            !Shape::Line {
                a: v(0.0, f32::INFINITY),
                b: v(1.0, 1.0),
                style: style(1.0)
            }
            .is_finite()
        );
        assert!(!ellipse(v(0.0, 0.0), v(f32::NEG_INFINITY, 1.0), 1.0, None).is_finite());
    }

    #[test]
    fn is_finite_false_for_non_finite_width() {
        assert!(!stroke(&[v(0.0, 0.0)], f32::NAN).is_finite());
        assert!(!rect(v(0.0, 0.0), v(1.0, 1.0), f32::INFINITY, None).is_finite());
    }

    // ---- properties ----

    fn coord() -> impl Strategy<Value = f32> {
        -1.0e3_f32..1.0e3_f32
    }

    fn point() -> impl Strategy<Value = Vec2> {
        (coord(), coord()).prop_map(|(x, y)| Vec2::new(x, y))
    }

    fn any_shape() -> impl Strategy<Value = Shape> {
        let width = 0.0_f32..20.0;
        prop_oneof![
            (prop::collection::vec(point(), 1..16), width.clone())
                .prop_map(|(points, w)| Shape::Stroke {
                    points,
                    style: style(w)
                }),
            (point(), point(), width.clone()).prop_map(|(a, b, w)| Shape::Line {
                a,
                b,
                style: style(w)
            }),
            (point(), point(), width.clone()).prop_map(|(a, b, w)| Shape::Arrow {
                a,
                b,
                style: style(w)
            }),
            (point(), point(), width.clone()).prop_map(|(a, b, w)| rect(a, b, w, None)),
            (point(), point(), width).prop_map(|(a, b, w)| ellipse(a, b, w, None)),
        ]
    }

    proptest! {
        #[test]
        fn translate_moves_bounds_by_delta(shape in any_shape(), delta in point()) {
            let before = shape.bounds();
            let mut moved = shape;

            moved.translate(delta);

            let expected = before.translate(delta);
            let after = moved.bounds();
            // Coordinates up to 2e3 in f32: rounding stays well below 1e-2.
            prop_assert!(after.min.approx_eq(expected.min, 1e-2), "{after:?} vs {expected:?}");
            prop_assert!(after.max.approx_eq(expected.max, 1e-2), "{after:?} vs {expected:?}");
        }
    }
}
