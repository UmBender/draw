//! Vector shapes with bounds, hit-testing and translation.
//!
//! The document is an ordered list of [`Shape`]s in world coordinates
//! (ADR-0004, amended by ADR-T16-1 with grids and labels). Widths are world units (ADR-0013). Every query here is total:
//! non-finite input never panics, a non-finite query point never hits and is
//! never contained.

use crate::core::geom::{Aabb, Vec2, distance_to_segment};
use crate::core::palette::ColorId;

/// Arrow head length per unit of stroke width.
pub const ARROW_HEAD_LENGTH_PER_WIDTH: f32 = 4.0;

/// Smallest arrow head length in world units, so thin arrows keep a visible head.
pub const ARROW_HEAD_MIN_LENGTH: f32 = 8.0;

/// Half the width of the arrow head base, as a fraction of the head length.
pub const ARROW_HEAD_HALF_WIDTH_RATIO: f32 = 0.5;

/// Most columns or rows a [`Shape::Grid`] can have.
pub const GRID_MAX_CELLS: u32 = 64;

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
/// For [`Shape::Rect`], [`Shape::Ellipse`] and [`Shape::Grid`], `a` and `b`
/// are opposite corners of the bounding box, in any order.
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
        /// Number drawn centred inside, if any.
        label: Option<u32>,
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
        /// Number drawn centred inside, if any.
        label: Option<u32>,
    },
    /// `cols × rows` uniform cells in the box spanned by `a` and `b`.
    ///
    /// An open shape: it has no interior and is never filled.
    Grid {
        /// One corner.
        a: Vec2,
        /// The opposite corner.
        b: Vec2,
        /// Number of columns, in `1..=GRID_MAX_CELLS`.
        cols: u32,
        /// Number of rows, in `1..=GRID_MAX_CELLS`.
        rows: u32,
        /// Outline style.
        style: Style,
        /// Draws 0-based column and row indices outside the grid, starting
        /// at corner `a` (ADR-T18-3).
        axes: bool,
    },
}

/// One axis index of a grid with [`Shape::Grid::axes`]: the 0-based column
/// or row `index` and the world box it is drawn centred in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisLabel {
    /// 0-based column or row index.
    pub index: u32,
    /// World box outside the grid, one cell in size.
    pub rect: Aabb,
}

impl Shape {
    /// The outline style.
    #[must_use]
    pub fn style(&self) -> Style {
        match self {
            Self::Stroke { style, .. }
            | Self::Line { style, .. }
            | Self::Arrow { style, .. }
            | Self::Rect { style, .. }
            | Self::Ellipse { style, .. }
            | Self::Grid { style, .. } => *style,
        }
    }

    /// The fill colour; always `None` for open shapes.
    #[must_use]
    pub fn fill(&self) -> Option<ColorId> {
        match self {
            Self::Rect { fill, .. } | Self::Ellipse { fill, .. } => *fill,
            Self::Stroke { .. } | Self::Line { .. } | Self::Arrow { .. } | Self::Grid { .. } => {
                None
            }
        }
    }

    /// The label; always `None` for shapes other than [`Shape::Rect`] and
    /// [`Shape::Ellipse`].
    #[must_use]
    pub fn label(&self) -> Option<u32> {
        match self {
            Self::Rect { label, .. } | Self::Ellipse { label, .. } => *label,
            Self::Stroke { .. } | Self::Line { .. } | Self::Arrow { .. } | Self::Grid { .. } => {
                None
            }
        }
    }

    /// `true` for shapes with an interior ([`Shape::Rect`], [`Shape::Ellipse`]).
    #[must_use]
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Rect { .. } | Self::Ellipse { .. })
    }

    /// Bounding box including half the outline width (and the arrow head).
    ///
    /// A stroke with no finite point gives the zero-size box at the origin.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        let geometry = match self {
            Self::Stroke { points, .. } => match Aabb::from_points(points) {
                Some(b) => b,
                None => return Aabb::from_corners(Vec2::ZERO, Vec2::ZERO),
            },
            Self::Arrow { a, b, style } => {
                let [tip, left, right] = arrow_head(*a, *b, style.width);
                Aabb::from_corners(*a, *b)
                    .union(&Aabb::from_corners(left, right))
                    .union(&Aabb::from_corners(tip, tip))
            }
            Self::Line { a, b, .. }
            | Self::Rect { a, b, .. }
            | Self::Ellipse { a, b, .. }
            | Self::Grid { a, b, .. } => Aabb::from_corners(*a, *b),
        };
        geometry.expand(half_width(self.style().width))
    }

    /// `true` if `p` is within `tol + width / 2` of the outline, or inside a
    /// filled closed shape.
    ///
    /// Ellipse outlines use the gradient-normalised distance estimate, exact on
    /// the axes and on the outline itself.
    #[must_use]
    pub fn hit(&self, p: Vec2, tol: f32) -> bool {
        let reach = tol + half_width(self.style().width);
        if !p.is_finite() || !reach.is_finite() || !self.bounds().expand(tol).contains(p) {
            return false;
        }
        if self.fill().is_some() && self.contains(p) {
            return true;
        }
        match self {
            Self::Stroke { points, .. } => match points.as_slice() {
                [] => false,
                [only] => p.distance(*only) <= reach,
                _ => points
                    .windows(2)
                    .any(|w| distance_to_segment(p, w[0], w[1]) <= reach),
            },
            Self::Line { a, b, .. } => distance_to_segment(p, *a, *b) <= reach,
            Self::Arrow { a, b, style } => {
                let head = arrow_head(*a, *b, style.width);
                distance_to_segment(p, *a, *b) <= reach
                    || point_in_triangle(p, head)
                    || polygon_edge_distance(p, &head) <= reach
            }
            Self::Rect { a, b, .. } => {
                let r = Aabb::from_corners(*a, *b);
                let corners = [
                    r.min,
                    Vec2::new(r.max.x, r.min.y),
                    r.max,
                    Vec2::new(r.min.x, r.max.y),
                ];
                polygon_edge_distance(p, &corners) <= reach
            }
            Self::Ellipse { a, b, .. } => ellipse_outline_distance(p, *a, *b) <= reach,
            Self::Grid {
                a, b, cols, rows, ..
            } => grid_lines(*a, *b, *cols, *rows)
                .any(|[start, end]| distance_to_segment(p, start, end) <= reach),
        }
    }

    /// `true` if `p` lies inside the geometry of a closed shape (boundary
    /// inclusive, outline width and fill ignored). Open shapes contain nothing.
    #[must_use]
    pub fn contains(&self, p: Vec2) -> bool {
        if !p.is_finite() {
            return false;
        }
        match self {
            Self::Rect { a, b, .. } => Aabb::from_corners(*a, *b).contains(p),
            Self::Ellipse { a, b, .. } => {
                let (center, radii) = ellipse_frame(*a, *b);
                if radii.x <= 0.0 || radii.y <= 0.0 {
                    return false;
                }
                let q = p - center;
                let (u, w) = (q.x / radii.x, q.y / radii.y);
                u * u + w * w <= 1.0
            }
            Self::Stroke { .. } | Self::Line { .. } | Self::Arrow { .. } | Self::Grid { .. } => {
                false
            }
        }
    }

    /// Moves every point by `delta`.
    pub fn translate(&mut self, delta: Vec2) {
        match self {
            Self::Stroke { points, .. } => {
                for point in points {
                    *point += delta;
                }
            }
            Self::Line { a, b, .. }
            | Self::Arrow { a, b, .. }
            | Self::Rect { a, b, .. }
            | Self::Ellipse { a, b, .. }
            | Self::Grid { a, b, .. } => {
                *a += delta;
                *b += delta;
            }
        }
    }

    /// Returns the shape with its fill set to `fill`; open shapes are
    /// returned unchanged.
    #[must_use]
    pub fn with_fill(mut self, fill: Option<ColorId>) -> Self {
        if let Self::Rect { fill: f, .. } | Self::Ellipse { fill: f, .. } = &mut self {
            *f = fill;
        }
        self
    }

    /// Returns the shape with its label set to `label`; shapes other than
    /// [`Shape::Rect`] and [`Shape::Ellipse`] are returned unchanged.
    #[must_use]
    pub fn with_label(mut self, label: Option<u32>) -> Self {
        if let Self::Rect { label: l, .. } | Self::Ellipse { label: l, .. } = &mut self {
            *l = label;
        }
        self
    }

    /// The axis indices to draw: those of a grid with `axes` on (see
    /// [`grid_axis_labels`]), empty for every other shape.
    #[must_use]
    pub fn axis_labels(&self) -> Vec<AxisLabel> {
        todo!()
    }

    /// `true` iff every coordinate and the width are finite.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.style().width.is_finite()
            && match self {
                Self::Stroke { points, .. } => points.iter().all(|p| p.is_finite()),
                Self::Line { a, b, .. }
                | Self::Arrow { a, b, .. }
                | Self::Rect { a, b, .. }
                | Self::Ellipse { a, b, .. }
                | Self::Grid { a, b, .. } => a.is_finite() && b.is_finite(),
            }
    }
}

/// The lines of a `cols × rows` grid in the box `a`–`b`: `cols + 1`
/// vertical lines left to right, then `rows + 1` horizontal lines top to
/// bottom, each as `[start, end]`. Dimensions are clamped to
/// `1..=GRID_MAX_CELLS`, so at most `2 * (GRID_MAX_CELLS + 1)` lines.
pub fn grid_lines(a: Vec2, b: Vec2, cols: u32, rows: u32) -> impl Iterator<Item = [Vec2; 2]> {
    let r = Aabb::from_corners(a, b);
    let (cols, rows) = (clamp_cells(cols), clamp_cells(rows));
    let (dx, dy) = (r.width() / cols as f32, r.height() / rows as f32);
    // The last line sits exactly on the far edge, free of rounding drift.
    let at = |min: f32, max: f32, step: f32, n: u32, i: u32| {
        if i == n { max } else { min + step * i as f32 }
    };
    let vertical = (0..=cols).map(move |i| {
        let x = at(r.min.x, r.max.x, dx, cols, i);
        [Vec2::new(x, r.min.y), Vec2::new(x, r.max.y)]
    });
    let horizontal = (0..=rows).map(move |j| {
        let y = at(r.min.y, r.max.y, dy, rows, j);
        [Vec2::new(r.min.x, y), Vec2::new(r.max.x, y)]
    });
    vertical.chain(horizontal)
}

/// The axis indices of a `cols × rows` grid dragged from `a` to `b`:
/// columns `0..cols` counted from `a.x` toward `b.x`, in boxes one cell tall
/// just outside the edge `y = a.y`, then rows `0..rows` counted from `a.y`
/// toward `b.y`, in boxes one cell wide just outside `x = a.x`
/// (ADR-T18-3). Dimensions are clamped like [`grid_lines`].
pub fn grid_axis_labels(a: Vec2, b: Vec2, cols: u32, rows: u32) -> impl Iterator<Item = AxisLabel> {
    let _ = (a, b, cols, rows);
    std::iter::empty::<AxisLabel>().chain(std::iter::from_fn(|| todo!()))
}

/// `n` clamped to the valid grid dimension range `1..=GRID_MAX_CELLS`.
fn clamp_cells(n: u32) -> u32 {
    n.clamp(1, GRID_MAX_CELLS)
}

/// Arrow head triangle `[tip, left, right]` for an arrow from `a` to `b`.
///
/// The tip is `b`; the base lies `max(ARROW_HEAD_MIN_LENGTH,
/// ARROW_HEAD_LENGTH_PER_WIDTH * width)` back along the shaft and is
/// `2 * ARROW_HEAD_HALF_WIDTH_RATIO` times that length wide. "Left" is the
/// side reached by turning the shaft direction a quarter turn towards +y.
/// A degenerate arrow (`a` ≈ `b`) gives `[b, b, b]`.
#[must_use]
pub fn arrow_head(a: Vec2, b: Vec2, width: f32) -> [Vec2; 3] {
    let shaft = b - a;
    let len = shaft.length();
    if len.is_nan() || len <= f32::EPSILON {
        return [b, b, b];
    }
    let dir = shaft / len;
    let head_len = (ARROW_HEAD_LENGTH_PER_WIDTH * width).max(ARROW_HEAD_MIN_LENGTH);
    let base = b - dir * head_len;
    let side = Vec2::new(-dir.y, dir.x) * (ARROW_HEAD_HALF_WIDTH_RATIO * head_len);
    [b, base + side, base - side]
}

/// Half of `width`, treating negative and NaN widths as 0.
fn half_width(width: f32) -> f32 {
    width.max(0.0) * 0.5
}

/// Centre and (non-negative) radii of the ellipse inscribed in the box `a`–`b`.
fn ellipse_frame(a: Vec2, b: Vec2) -> (Vec2, Vec2) {
    let r = Aabb::from_corners(a, b);
    (r.center(), Vec2::new(r.width() * 0.5, r.height() * 0.5))
}

/// Approximate distance from `p` to the outline of the ellipse in box `a`–`b`.
///
/// Uses `k0 (k0 − 1) / k1` with `k0 = |q / r|`, `k1 = |q / r²|`: the implicit
/// function divided by its gradient length. Exact on the axes and zero on the
/// outline. A zero radius collapses the ellipse to a segment.
fn ellipse_outline_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let (center, radii) = ellipse_frame(a, b);
    if radii.x <= 0.0 || radii.y <= 0.0 {
        return distance_to_segment(p, center - radii, center + radii);
    }
    let offset = p - center;
    let k0 = Vec2::new(offset.x / radii.x, offset.y / radii.y).length();
    let k1 = Vec2::new(
        offset.x / (radii.x * radii.x),
        offset.y / (radii.y * radii.y),
    )
    .length();
    if k1.is_nan() || k1 <= 0.0 {
        // At the centre the gradient vanishes; the nearest outline point is
        // the end of the shorter axis.
        return radii.x.min(radii.y);
    }
    (k0 * (k0 - 1.0) / k1).abs()
}

/// Smallest distance from `p` to the edges of the closed polygon `vertices`.
fn polygon_edge_distance(p: Vec2, vertices: &[Vec2]) -> f32 {
    let n = vertices.len();
    (0..n)
        .map(|i| distance_to_segment(p, vertices[i], vertices[(i + 1) % n]))
        .fold(f32::INFINITY, f32::min)
}

/// `true` if `p` is inside or on the triangle, for either winding.
/// A degenerate (zero-area or non-finite) triangle contains nothing.
fn point_in_triangle(p: Vec2, [t0, t1, t2]: [Vec2; 3]) -> bool {
    let area = cross(t1 - t0, t2 - t0).abs();
    if area.is_nan() || area <= f32::EPSILON {
        return false;
    }
    let d0 = cross(t1 - t0, p - t0);
    let d1 = cross(t2 - t1, p - t1);
    let d2 = cross(t0 - t2, p - t2);
    let has_neg = d0 < 0.0 || d1 < 0.0 || d2 < 0.0;
    let has_pos = d0 > 0.0 || d1 > 0.0 || d2 > 0.0;
    !(has_neg && has_pos)
}

/// 2D cross product (z component of the 3D cross product).
fn cross(u: Vec2, w: Vec2) -> f32 {
    u.x * w.y - u.y * w.x
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
            label: None,
        }
    }

    fn ellipse(a: Vec2, b: Vec2, width: f32, fill: Option<ColorId>) -> Shape {
        Shape::Ellipse {
            a,
            b,
            style: style(width),
            fill,
            label: None,
        }
    }

    fn grid(a: Vec2, b: Vec2, cols: u32, rows: u32, width: f32) -> Shape {
        Shape::Grid {
            a,
            b,
            cols,
            rows,
            style: style(width),
            axes: false,
        }
    }

    fn grid_with_axes(a: Vec2, b: Vec2, cols: u32, rows: u32) -> Shape {
        Shape::Grid {
            a,
            b,
            cols,
            rows,
            style: style(0.0),
            axes: true,
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

        assert!(
            !r.contains(v(10.5, 5.0)),
            "within stroke but outside geometry"
        );
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
        let s = stroke(
            &[v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0), v(0.0, 0.0)],
            2.0,
        );

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
        assert!(approx_eq(
            tip.distance(base_mid),
            ARROW_HEAD_MIN_LENGTH,
            EPS
        ));
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

    // ---- T16 AC-1: grid ----

    #[test]
    fn grid_bounds_include_half_width() {
        let g = grid(v(30.0, 20.0), v(0.0, 0.0), 3, 2, 2.0);

        assert_aabb_eq(g.bounds(), v(-1.0, -1.0), v(31.0, 21.0));
    }

    #[test]
    fn grid_hit_on_inner_line() {
        // 3 × 2 cells of 10 × 10; width 2 gives reach 1 at tol 0.
        let g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        assert!(g.hit(v(10.0, 5.0), 0.0), "inner vertical line");
        assert!(g.hit(v(20.9, 15.0), 0.0), "within reach of x = 20");
        assert!(g.hit(v(5.0, 10.0), 0.0), "inner horizontal line");
        assert!(g.hit(v(30.0, 3.0), 0.0), "outer edge");
    }

    #[test]
    fn grid_hit_between_lines_is_false() {
        let g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        assert!(!g.hit(v(15.0, 5.0), 0.0), "cell centre");
        assert!(!g.hit(v(12.5, 15.0), 0.5), "beyond reach");
        assert!(!g.hit(v(40.0, 5.0), 0.5), "outside");
        assert!(!g.hit(v(f32::NAN, 5.0), 1.0), "non-finite");
    }

    #[test]
    fn grid_contains_nothing_and_is_open() {
        let g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        assert!(!g.contains(v(15.0, 5.0)));
        assert!(!g.is_closed());
        assert_eq!(g.fill(), None);
        assert_eq!(g.label(), None);
    }

    #[test]
    fn grid_with_fill_is_noop() {
        let g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        assert_eq!(g.clone().with_fill(Some(color(2))), g);
    }

    #[test]
    fn grid_translate_moves_corners() {
        let mut g = grid(v(0.0, 0.0), v(30.0, 20.0), 3, 2, 2.0);

        g.translate(v(5.0, -5.0));

        assert_eq!(g, grid(v(5.0, -5.0), v(35.0, 15.0), 3, 2, 2.0));
    }

    #[test]
    fn grid_is_finite_checks_corners() {
        assert!(grid(v(0.0, 0.0), v(1.0, 1.0), 4, 4, 1.0).is_finite());
        assert!(!grid(v(f32::NAN, 0.0), v(1.0, 1.0), 4, 4, 1.0).is_finite());
        assert!(!grid(v(0.0, 0.0), v(1.0, f32::INFINITY), 4, 4, 1.0).is_finite());
        assert!(!grid(v(0.0, 0.0), v(1.0, 1.0), 4, 4, f32::NAN).is_finite());
    }

    #[test]
    fn grid_lines_count_and_positions() {
        // Corners in reverse order: lines follow the normalised box.
        let lines: Vec<[Vec2; 2]> = grid_lines(v(30.0, 20.0), v(0.0, 0.0), 3, 2).collect();

        assert_eq!(lines.len(), 4 + 3, "cols + 1 vertical, rows + 1 horizontal");
        for (i, [p, q]) in lines[..4].iter().enumerate() {
            let x = 10.0 * i as f32;
            assert_vec_eq(*p, v(x, 0.0));
            assert_vec_eq(*q, v(x, 20.0));
        }
        for (j, [p, q]) in lines[4..].iter().enumerate() {
            let y = 10.0 * j as f32;
            assert_vec_eq(*p, v(0.0, y));
            assert_vec_eq(*q, v(30.0, y));
        }
    }

    #[test]
    fn grid_lines_clamps_dims() {
        let (a, b) = (v(0.0, 0.0), v(10.0, 10.0));

        assert_eq!(grid_lines(a, b, 0, 0).count(), 2 + 2);
        assert_eq!(
            grid_lines(a, b, u32::MAX, 1000).count(),
            2 * (GRID_MAX_CELLS as usize + 1)
        );
    }

    // ---- T16 AC-1: labels ----

    #[test]
    fn label_accessor_and_with_label() {
        let r = rect(v(0.0, 0.0), v(10.0, 10.0), 1.0, None);
        let e = ellipse(v(0.0, 0.0), v(10.0, 10.0), 1.0, None);

        assert_eq!(r.label(), None);
        let r7 = r.with_label(Some(7));
        let e3 = e.with_label(Some(3));

        assert_eq!(r7.label(), Some(7));
        assert_eq!(e3.label(), Some(3));
        assert!(matches!(r7, Shape::Rect { label: Some(7), .. }));
        assert_eq!(e3.with_label(None).label(), None);
    }

    #[test]
    fn label_with_label_on_open_shape_is_noop() {
        let line = Shape::Line {
            a: v(0.0, 0.0),
            b: v(1.0, 0.0),
            style: style(1.0),
        };
        let g = grid(v(0.0, 0.0), v(10.0, 10.0), 2, 2, 1.0);
        let s = stroke(&[v(0.0, 0.0)], 1.0);

        for shape in [line, g, s] {
            assert_eq!(shape.clone().with_label(Some(4)), shape);
            assert_eq!(shape.label(), None);
        }
    }

    #[test]
    fn label_survives_translate_and_with_fill() {
        let mut r = rect(v(0.0, 0.0), v(10.0, 10.0), 1.0, None).with_label(Some(2));
        let e = ellipse(v(0.0, 0.0), v(10.0, 10.0), 1.0, None).with_label(Some(9));

        r.translate(v(1.0, 1.0));
        let filled = e.with_fill(Some(color(4)));

        assert_eq!(r.label(), Some(2));
        assert_eq!(filled.label(), Some(9));
        assert_eq!(filled.fill(), Some(color(4)));
    }

    // ---- properties ----

    fn coord() -> impl Strategy<Value = f32> {
        -1.0e3_f32..1.0e3_f32
    }

    fn point() -> impl Strategy<Value = Vec2> {
        (coord(), coord()).prop_map(|(x, y)| Vec2::new(x, y))
    }

    fn label() -> impl Strategy<Value = Option<u32>> {
        prop::option::of(1_u32..1000)
    }

    fn any_shape() -> impl Strategy<Value = Shape> {
        let width = 0.0_f32..20.0;
        prop_oneof![
            (prop::collection::vec(point(), 1..16), width.clone()).prop_map(|(points, w)| {
                Shape::Stroke {
                    points,
                    style: style(w),
                }
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
            (point(), point(), width.clone(), label())
                .prop_map(|(a, b, w, l)| rect(a, b, w, None).with_label(l)),
            (point(), point(), width.clone(), label())
                .prop_map(|(a, b, w, l)| ellipse(a, b, w, None).with_label(l)),
            (
                point(),
                point(),
                1..=GRID_MAX_CELLS,
                1..=GRID_MAX_CELLS,
                width
            )
                .prop_map(|(a, b, cols, rows, w)| grid(a, b, cols, rows, w)),
        ]
    }

    proptest! {
        #[test]
        fn translate_moves_bounds_by_delta(shape in any_shape(), delta in point()) {
            let before = shape.bounds();
            let mut moved = shape;

            let label = moved.label();
            moved.translate(delta);

            prop_assert_eq!(moved.label(), label);
            let expected = before.translate(delta);
            let after = moved.bounds();
            // Coordinates up to 2e3 in f32: rounding stays well below 1e-2.
            prop_assert!(after.min.approx_eq(expected.min, 1e-2), "{after:?} vs {expected:?}");
            prop_assert!(after.max.approx_eq(expected.max, 1e-2), "{after:?} vs {expected:?}");
        }
    }

    // ---- T18 AC-7 axis indices --------------------------------------------

    /// `(index, rect)` of each axis label of `shape`, columns then rows.
    fn axis(shape: &Shape) -> Vec<(u32, Aabb)> {
        shape
            .axis_labels()
            .into_iter()
            .map(|l| (l.index, l.rect))
            .collect()
    }

    fn rect_eq(got: Aabb, min: (f32, f32), max: (f32, f32)) -> bool {
        got.min.approx_eq(Vec2::new(min.0, min.1), 1e-4)
            && got.max.approx_eq(Vec2::new(max.0, max.1), 1e-4)
    }

    #[test]
    fn grid_axis_labels_top_left_drag() {
        // Arrange: 3 × 2 cells of 10 × 20, dragged from (0, 0) to (30, 40).
        let shape = grid_with_axes(Vec2::ZERO, Vec2::new(30.0, 40.0), 3, 2);

        // Act
        let labels = axis(&shape);

        // Assert: columns 0..3 above the top edge, rows 0..2 left of it.
        assert_eq!(labels.len(), 5);
        let indices: Vec<u32> = labels.iter().map(|l| l.0).collect();
        assert_eq!(indices, [0, 1, 2, 0, 1]);
        assert!(
            rect_eq(labels[0].1, (0.0, -20.0), (10.0, 0.0)),
            "{:?}",
            labels[0]
        );
        assert!(
            rect_eq(labels[2].1, (20.0, -20.0), (30.0, 0.0)),
            "{:?}",
            labels[2]
        );
        assert!(
            rect_eq(labels[3].1, (-10.0, 0.0), (0.0, 20.0)),
            "{:?}",
            labels[3]
        );
        assert!(
            rect_eq(labels[4].1, (-10.0, 20.0), (0.0, 40.0)),
            "{:?}",
            labels[4]
        );
    }

    #[test]
    fn grid_axis_labels_follow_drag_direction() {
        // Arrange: dragged from the bottom-right (30, 40) to the top-left.
        let shape = grid_with_axes(Vec2::new(30.0, 40.0), Vec2::ZERO, 3, 2);

        // Act
        let labels = axis(&shape);

        // Assert: column 0 is the rightmost, below the bottom edge; row 0 is
        // the bottom row, right of the right edge.
        assert!(
            rect_eq(labels[0].1, (20.0, 40.0), (30.0, 60.0)),
            "{:?}",
            labels[0]
        );
        assert!(
            rect_eq(labels[2].1, (0.0, 40.0), (10.0, 60.0)),
            "{:?}",
            labels[2]
        );
        assert!(
            rect_eq(labels[3].1, (30.0, 20.0), (40.0, 40.0)),
            "{:?}",
            labels[3]
        );
        assert!(
            rect_eq(labels[4].1, (30.0, 0.0), (40.0, 20.0)),
            "{:?}",
            labels[4]
        );
        let indices: Vec<u32> = labels.iter().map(|l| l.0).collect();
        assert_eq!(indices, [0, 1, 2, 0, 1]);
    }

    #[test]
    fn grid_axis_labels_none_without_axes() {
        let plain = grid(Vec2::ZERO, Vec2::new(30.0, 40.0), 3, 2, 1.0);
        let other = rect(Vec2::ZERO, Vec2::new(30.0, 40.0), 1.0, None);

        assert!(plain.axis_labels().is_empty());
        assert!(other.axis_labels().is_empty());
    }

    #[test]
    fn grid_bounds_include_axis_labels() {
        // Arrange
        let shape = grid_with_axes(Vec2::ZERO, Vec2::new(30.0, 40.0), 3, 2);

        // Act
        let bounds = shape.bounds();

        // Assert: one cell height above, one cell width to the left.
        assert!(rect_eq(bounds, (-10.0, -20.0), (30.0, 40.0)), "{bounds:?}");
        for (_, r) in axis(&shape) {
            assert!(bounds.contains(r.min) && bounds.contains(r.max));
        }
    }

    #[test]
    fn grid_axes_survive_translate() {
        // Arrange
        let mut shape = grid_with_axes(Vec2::new(30.0, 40.0), Vec2::ZERO, 3, 2);
        let before = axis(&shape);

        // Act
        shape.translate(Vec2::new(5.0, -7.0));

        // Assert: same orientation, moved by the delta.
        let after = axis(&shape);
        assert_eq!(before.len(), after.len());
        for ((i, r0), (j, r1)) in before.iter().zip(&after) {
            assert_eq!(i, j);
            assert!(r1.min.approx_eq(r0.min + Vec2::new(5.0, -7.0), 1e-4));
        }
    }
}
