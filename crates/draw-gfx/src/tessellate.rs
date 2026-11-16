//! Tessellation of shapes into triangles in world coordinates (T146).
//!
//! The result does not depend on the camera: a pan or a zoom changes the
//! uniform the vertex shader applies (T173), never these vertices.

use draw_core::core::geom::{Aabb, Vec2};
use draw_core::core::palette::{ColorId, palette};
use draw_core::core::shape::{Shape, arrow_head, grid_cell_rect, grid_lines};

use crate::gpu::{Geometry, Vertex};

/// Indexed triangles in world units.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    /// The vertices, `uv` zero, straight colour.
    pub vertices: Vec<Vertex>,
    /// Triangle list indices into `vertices`.
    pub indices: Vec<u32>,
}

impl Mesh {
    /// The mesh as inline geometry of a draw command.
    #[must_use]
    pub fn geometry<B>(&self) -> Geometry<B> {
        Geometry::Inline {
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
        }
    }
}

/// Tessellates `shape`; a shape with a non-finite coordinate, and text
/// (drawn by glyph quads), give an empty mesh.
#[must_use]
pub fn tessellate(shape: &Shape) -> Mesh {
    tessellate_at(shape, 1.0)
}

/// Minimum on-screen width of an outline, in pixels (as `draw_shape`).
const MIN_SCREEN_WIDTH_PX: f32 = 1.0;
/// Strokes wider than this (pixels) get round joints.
const JOINT_THRESHOLD_PX: f32 = 2.0;
/// Points closer than this (pixels) to the last kept one are skipped.
const STROKE_MIN_STEP_PX: f32 = CHORD_TOLERANCE_PX;
/// Longest mitre, in half widths, before a stroke breaks into a disc.
const MITER_LIMIT: f32 = 1.2;
/// Chord error allowed for a circle, in pixels.
const CHORD_TOLERANCE_PX: f32 = 0.25;
/// Fewest and most segments of a circle.
const MIN_SEGMENTS: f64 = 8.0;
const MAX_SEGMENTS: f64 = 256.0;
/// Half the feather: the edge is at coverage one half, one pixel ramp.
const HALF_FEATHER_PX: f32 = 0.5;

/// Tessellates `shape` for a view at `zoom` pixels per world unit
/// (ADR-T179-1): the minimum screen width, the joint rule and the segment
/// counts of the screen-space drawing, and the feather geometry
/// `Blend::AlphaFeathered` reads. Pass a [`zoom_bucket`].
#[must_use]
pub fn tessellate_at(shape: &Shape, zoom: f32) -> Mesh {
    if !shape.is_finite() {
        return Mesh::default();
    }
    let zoom = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    let mut b = Builder::new(zoom);
    match shape {
        Shape::Stroke {
            points,
            widths,
            style,
        } => {
            let width_at = |i: usize| {
                let factor = widths.as_ref().and_then(|w| w.get(i)).copied();
                style.width * factor.unwrap_or(1.0)
            };
            let pairs: Vec<(Vec2, f32)> = points
                .iter()
                .enumerate()
                .map(|(i, p)| (*p, width_at(i)))
                .collect();
            b.polyline(&pairs, rgba(style.color));
        }
        Shape::Line { a, b: end, style } => {
            b.segment_round(*a, *end, style.width, rgba(style.color));
        }
        Shape::Arrow { a, b: end, style } => {
            let color = rgba(style.color);
            let head = arrow_head(*a, *end, style.width);
            let base = head[1].lerp(head[2], 0.5);
            b.segment_round(*a, base, style.width, color);
            b.convex(&head, color);
        }
        Shape::Rect {
            a,
            b: end,
            style,
            fill,
            ..
        } => {
            let bounds = Aabb::from_corners(*a, *end);
            if let Some(fill) = fill {
                b.solid_rect(bounds, rgba(*fill));
            }
            b.rect_ring(bounds, style.width, rgba(style.color));
        }
        Shape::Ellipse {
            a,
            b: end,
            style,
            fill,
            ..
        } => {
            let bounds = Aabb::from_corners(*a, *end);
            let radii = Vec2::new(bounds.width() / 2.0, bounds.height() / 2.0);
            if let Some(fill) = fill {
                b.solid_ellipse(bounds.center(), radii, rgba(*fill));
            }
            b.ellipse_ring(bounds.center(), radii, style.width, rgba(style.color));
        }
        Shape::Grid {
            a,
            b: end,
            cols,
            rows,
            style,
            fills,
            ..
        } => {
            for cell in fills {
                let rect = grid_cell_rect(*a, *end, *cols, *rows, cell.col, cell.row);
                b.solid_rect(rect, rgba(cell.color));
            }
            let color = rgba(style.color);
            let half = b.width(style.width) / 2.0;
            for [p, q] in grid_lines(*a, *end, *cols, *rows) {
                b.solid_rect(Aabb::from_corners(p, q).expand(half), color);
            }
        }
        Shape::Text { .. } => {}
    }
    b.mesh
}

/// The zoom rounded to the nearest power of two in the log domain; `1` for a
/// zoom that is not finite and positive.
#[must_use]
pub fn zoom_bucket(zoom: f32) -> f32 {
    if !zoom.is_finite() || zoom <= 0.0 {
        return 1.0;
    }
    zoom.log2().round().exp2()
}

/// Most dots of [`dot_grid_mesh`].
pub const DOT_GRID_MAX_DOTS: usize = 100_000;

/// The snap dot grid in `view` (world units): one square of `dot` world
/// units per multiple of `spacing`, `color`, feather-free. Empty for a
/// spacing that is not finite and positive and for more than
/// [`DOT_GRID_MAX_DOTS`] dots.
#[must_use]
pub fn dot_grid_mesh(view: Aabb, spacing: f32, dot: f32, color: [u8; 4]) -> Mesh {
    let finite = |v: f32| v.is_finite();
    if !(finite(spacing) && spacing > 0.0 && finite(dot))
        || !(finite(view.min.x) && finite(view.min.y) && finite(view.max.x) && finite(view.max.y))
    {
        return Mesh::default();
    }
    let range = |lo: f32, hi: f32| {
        let first = (f64::from(lo) / f64::from(spacing)).ceil();
        let last = (f64::from(hi) / f64::from(spacing)).floor();
        (first, last)
    };
    let (x0, x1) = range(view.min.x, view.max.x);
    let (y0, y1) = range(view.min.y, view.max.y);
    let (nx, ny) = ((x1 - x0 + 1.0).max(0.0), (y1 - y0 + 1.0).max(0.0));
    if nx * ny > DOT_GRID_MAX_DOTS as f64 {
        return Mesh::default();
    }
    let mut b = Builder::new(1.0);
    let half = Vec2::new(dot / 2.0, dot / 2.0);
    // The counts are bounded by DOT_GRID_MAX_DOTS, so the casts are exact.
    for j in 0..ny as u64 {
        for i in 0..nx as u64 {
            let center = Vec2::new(
                ((x0 + i as f64) * f64::from(spacing)) as f32,
                ((y0 + j as f64) * f64::from(spacing)) as f32,
            );
            let dot = Aabb::from_corners(center - half, center + half);
            b.quad(corners(dot), [0.0; 4], color);
        }
    }
    b.mesh
}

fn rgba(id: ColorId) -> [u8; 4] {
    let c = palette(id);
    [c.r, c.g, c.b, c.a]
}

/// Number of segments for a circle of `radius_px` pixels (as
/// `draw_shape`): the fewest whose chord error is within the tolerance.
fn circle_segments(radius_px: f32) -> usize {
    if !radius_px.is_finite() || radius_px <= 0.0 {
        return MIN_SEGMENTS as usize;
    }
    let ratio = f64::from(CHORD_TOLERANCE_PX) / f64::from(radius_px);
    if ratio >= 1.0 {
        return MIN_SEGMENTS as usize;
    }
    let n = (std::f64::consts::PI / (1.0 - ratio).acos()).ceil();
    // Clamped to a small range, so the cast is exact.
    n.clamp(MIN_SEGMENTS, MAX_SEGMENTS) as usize
}

/// The unit circle in `n` steps, starting at `(1, 0)`.
fn unit_circle(n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let t = std::f32::consts::TAU * i as f32 / n as f32;
            Vec2::new(t.cos(), t.sin())
        })
        .collect()
}

/// Builds feathered triangles for a view at `zoom`. A vertex's `uv[0]` is
/// its distance inside the shape's edge in world units; the feather is
/// `half` world units on each side of the edge.
struct Builder {
    mesh: Mesh,
    zoom: f32,
    half: f32,
}

impl Builder {
    fn new(zoom: f32) -> Self {
        Self {
            mesh: Mesh::default(),
            zoom,
            half: HALF_FEATHER_PX / zoom,
        }
    }

    /// The world width of an outline of `world` units: at least one pixel.
    fn width(&self, world: f32) -> f32 {
        let px = world * self.zoom;
        let px = if px.is_finite() && px >= MIN_SCREEN_WIDTH_PX {
            px
        } else {
            MIN_SCREEN_WIDTH_PX
        };
        px / self.zoom
    }

    fn vertex(&mut self, p: Vec2, d: f32, color: [u8; 4]) -> u32 {
        let index = u32::try_from(self.mesh.vertices.len()).unwrap_or(u32::MAX);
        self.mesh.vertices.push(Vertex {
            pos: [p.x, p.y],
            uv: [d, 0.0],
            color,
        });
        index
    }

    /// A quad whose corners carry the distances `d`.
    fn quad(&mut self, corners: [Vec2; 4], d: [f32; 4], color: [u8; 4]) {
        let i = self.vertex(corners[0], d[0], color);
        for k in 1..4 {
            self.vertex(corners[k], d[k], color);
        }
        self.mesh.indices.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
    }

    /// Triangles between two loops of the same length, `d_a` on `a` and
    /// `d_b` on `b`.
    fn between(&mut self, a: &[Vec2], d_a: f32, b: &[Vec2], d_b: f32, color: [u8; 4]) {
        let n = a.len().min(b.len());
        for i in 0..n {
            let j = (i + 1) % n;
            self.quad([a[i], a[j], b[j], b[i]], [d_a, d_a, d_b, d_b], color);
        }
    }

    /// A triangle fan from `center` (`d_center`) to the loop `rim` (`d`).
    fn fan(&mut self, center: Vec2, d_center: f32, rim: &[Vec2], d: f32, color: [u8; 4]) {
        let c = self.vertex(center, d_center, color);
        for p in rim {
            self.vertex(*p, d, color);
        }
        let n = u32::try_from(rim.len()).unwrap_or(0);
        for k in 0..n {
            self.mesh
                .indices
                .extend([c, c + 1 + k, c + 1 + (k + 1) % n]);
        }
    }

    /// A feathered disc of `radius` world units (the edge radius).
    fn disc(&mut self, center: Vec2, radius: f32, color: [u8; 4]) {
        let f = self.half;
        let n = circle_segments((radius + f) * self.zoom);
        let unit = unit_circle(n);
        let ring = |r: f32| -> Vec<Vec2> { unit.iter().map(|u| center + *u * r).collect() };
        let inner = ring((radius - f).max(0.0));
        let outer = ring(radius + f);
        self.fan(center, f, &inner, f, color);
        self.between(&outer, -f, &inner, f, color);
    }

    /// A feathered axis-aligned rectangle (the edge is `r`'s boundary).
    fn solid_rect(&mut self, r: Aabb, color: [u8; 4]) {
        let f = self.half;
        let inset = r.expand(-f);
        // A rectangle thinner than the feather has no opaque core.
        let inner = if inset.min.x <= inset.max.x && inset.min.y <= inset.max.y {
            inset
        } else {
            Aabb::from_corners(r.center(), r.center())
        };
        self.rect_loops(r.expand(f), -f, inner, f, color);
        self.quad(corners(inner), [f; 4], color);
    }

    /// The ring between two rectangles' loops.
    fn rect_loops(&mut self, a: Aabb, d_a: f32, b: Aabb, d_b: f32, color: [u8; 4]) {
        self.between(&corners(a), d_a, &corners(b), d_b, color);
    }

    /// The outline of `bounds`, `world` wide, with square corners.
    fn rect_ring(&mut self, bounds: Aabb, world: f32, color: [u8; 4]) {
        let (h, f) = (self.width(world) / 2.0, self.half);
        let inner_half = Vec2::new(bounds.width() / 2.0 - h, bounds.height() / 2.0 - h);
        if inner_half.x <= f || inner_half.y <= f {
            self.solid_rect(bounds.expand(h), color);
            return;
        }
        self.rect_loops(bounds.expand(h + f), -f, bounds.expand(h - f), f, color);
        self.rect_loops(bounds.expand(h - f), f, bounds.expand(-h + f), f, color);
        self.rect_loops(bounds.expand(-h + f), f, bounds.expand(-h - f), -f, color);
    }

    /// A feathered ellipse with edge radii `radii`.
    fn solid_ellipse(&mut self, center: Vec2, radii: Vec2, color: [u8; 4]) {
        let f = self.half;
        let n = circle_segments(radii.x.max(radii.y) * self.zoom + HALF_FEATHER_PX);
        let unit = unit_circle(n);
        let ring = |d: f32| -> Vec<Vec2> {
            unit.iter()
                .map(|u| {
                    let r = Vec2::new((radii.x + d).max(0.0), (radii.y + d).max(0.0));
                    center + Vec2::new(u.x * r.x, u.y * r.y)
                })
                .collect()
        };
        let (outer, inner) = (ring(f), ring(-f));
        self.fan(center, f, &inner, f, color);
        self.between(&outer, -f, &inner, f, color);
    }

    /// The outline of an ellipse, `world` wide, centred on its edge.
    fn ellipse_ring(&mut self, center: Vec2, radii: Vec2, world: f32, color: [u8; 4]) {
        let (h, f) = (self.width(world) / 2.0, self.half);
        if radii.x - h <= f || radii.y - h <= f {
            self.solid_ellipse(center, Vec2::new(radii.x + h, radii.y + h), color);
            return;
        }
        let n = circle_segments((radii.x.max(radii.y) + h + f) * self.zoom);
        let unit = unit_circle(n);
        let ring = |d: f32| -> Vec<Vec2> {
            unit.iter()
                .map(|u| center + Vec2::new(u.x * (radii.x + d), u.y * (radii.y + d)))
                .collect()
        };
        let (o_out, o_in) = (ring(h + f), ring(h - f));
        let (i_out, i_in) = (ring(-h + f), ring(-h - f));
        self.between(&o_out, -f, &o_in, f, color);
        self.between(&o_in, f, &i_out, f, color);
        self.between(&i_out, f, &i_in, -f, color);
    }

    /// A feathered convex polygon (counter-clockwise or clockwise).
    fn convex(&mut self, points: &[Vec2], color: [u8; 4]) {
        let n = points.len();
        if n < 3 {
            return;
        }
        let area: f32 = (0..n)
            .map(|i| {
                let (p, q) = (points[i], points[(i + 1) % n]);
                p.x * q.y - p.y * q.x
            })
            .sum::<f32>();
        if !area.is_finite() || area.abs() <= f32::EPSILON {
            return;
        }
        let sign = if area > 0.0 { 1.0 } else { -1.0 };
        let f = self.half;
        // Outward offset of each vertex by `by`, along its mitre.
        let offset = |by: f32| -> Vec<Vec2> {
            (0..n)
                .map(|i| {
                    let prev = points[(i + n - 1) % n];
                    let next = points[(i + 1) % n];
                    let out = |a: Vec2, b: Vec2| {
                        let d = b - a;
                        let len = d.length().max(f32::EPSILON);
                        Vec2::new(d.y, -d.x) * (sign / len)
                    };
                    let (n0, n1) = (out(prev, points[i]), out(points[i], next));
                    let c = n0.dot(n1).max(-0.5);
                    points[i] + (n0 + n1) * (by / (1.0 + c))
                })
                .collect()
        };
        let (outer, inner) = (offset(f), offset(-f));
        let center = points.iter().fold(Vec2::ZERO, |s, p| s + *p) / n as f32;
        self.fan(center, f, &inner, f, color);
        self.between(&outer, -f, &inner, f, color);
    }

    /// A segment `world` wide (at least a pixel on screen), feathered at its
    /// sides, with a round disc at each end when thick.
    fn segment_round(&mut self, a: Vec2, b: Vec2, world: f32, color: [u8; 4]) {
        self.polyline(&[(a, world), (b, world)], color);
    }

    /// A stroke through `points` with a world width each (ADR-T24-4,
    /// ADR-T35-2): a mitred strip skipping points closer than a quarter
    /// pixel, breaking where the mitre is too long; round discs at the ends
    /// and breaks when thick; a single point is a dot.
    fn polyline(&mut self, points: &[(Vec2, f32)], color: [u8; 4]) {
        let Some(&(first, first_width)) = points.first() else {
            return;
        };
        let min_step = STROKE_MIN_STEP_PX / self.zoom;
        let half_of = |this: &Self, w: f32| this.width(w) / 2.0;
        let round_of = |this: &Self, w: f32| this.width(w) * this.zoom > JOINT_THRESHOLD_PX;
        let (mut prev, mut h) = (first, half_of(self, first_width));
        let mut round = round_of(self, first_width);
        let mut prev_normal = None::<Vec2>;
        let mut last = None::<[Vec2; 4]>;
        for &(p, width) in &points[1..] {
            let d = p - prev;
            let len = d.length();
            if !len.is_finite() || len < min_step {
                continue;
            }
            let n = Vec2::new(-d.y, d.x) * (1.0 / len);
            match prev_normal {
                None => {
                    if round {
                        self.disc(prev, h, color);
                    }
                    last = Some(self.row(None, prev, n * h, h, color));
                }
                Some(n0) => {
                    let c = n0.dot(n);
                    if f32::midpoint(1.0, c) * MITER_LIMIT * MITER_LIMIT >= 1.0 {
                        let offset = (n0 + n) * (1.0 / (1.0 + c));
                        last = Some(self.row(last, prev, offset * h, h, color));
                    } else {
                        self.row(last, prev, n0 * h, h, color);
                        if round {
                            self.disc(prev, h, color);
                        }
                        last = Some(self.row(None, prev, n * h, h, color));
                    }
                }
            }
            (prev, prev_normal) = (p, Some(n));
            (h, round) = (half_of(self, width), round_of(self, width));
        }
        match prev_normal {
            None => self.disc(first, h, color),
            Some(n0) => {
                self.row(last, prev, n0 * h, h, color);
                if round {
                    self.disc(prev, h, color);
                }
            }
        }
    }

    /// The cross-section of a strip at `at`, offset `side` of half width
    /// `h`: outer left, inner left, inner right, outer right, carrying the
    /// feather distances. When `from` holds the previous section, the three
    /// quads joining them are added.
    fn row(
        &mut self,
        from: Option<[Vec2; 4]>,
        at: Vec2,
        side: Vec2,
        h: f32,
        color: [u8; 4],
    ) -> [Vec2; 4] {
        let f = self.half;
        let (outer, inner) = ((h + f) / h, ((h - f) / h).max(0.0));
        let section = [
            at + side * outer,
            at + side * inner,
            at - side * inner,
            at - side * outer,
        ];
        if let Some(prev) = from {
            let d = [-f, f, f, -f];
            for k in 0..3 {
                self.quad(
                    [prev[k], prev[k + 1], section[k + 1], section[k]],
                    [d[k], d[k + 1], d[k + 1], d[k]],
                    color,
                );
            }
        }
        section
    }
}

fn corners(r: Aabb) -> [Vec2; 4] {
    [
        r.min,
        Vec2::new(r.max.x, r.min.y),
        r.max,
        Vec2::new(r.min.x, r.max.y),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use draw_core::core::palette::ColorId;
    use draw_core::core::shape::Style;

    use draw_core::core::geom::approx_eq;

    const EPS: f32 = 1e-3;

    fn style(width: f32) -> Style {
        Style {
            color: ColorId::INK,
            width,
        }
    }

    fn horizontal(width: f32) -> Shape {
        Shape::Line {
            a: Vec2::new(0.0, 0.0),
            b: Vec2::new(10.0, 0.0),
            style: style(width),
        }
    }

    fn max_abs_y(mesh: &Mesh) -> f32 {
        mesh.vertices
            .iter()
            .map(|v| v.pos[1].abs())
            .fold(0.0, f32::max)
    }

    fn uv_range(mesh: &Mesh) -> (f32, f32) {
        let ds = mesh.vertices.iter().map(|v| v.uv[0]);
        (
            ds.clone().fold(f32::INFINITY, f32::min),
            ds.fold(f32::NEG_INFINITY, f32::max),
        )
    }

    #[test]
    fn line_edge_vertices_carry_feather_distance() {
        // Arrange: a 4 px line at zoom 1, so the feather is 0.5 world units.
        let mesh = tessellate_at(&horizontal(4.0), 1.0);
        // Act
        let (low, high) = uv_range(&mesh);
        // Assert: outside the edge is negative, inside is positive.
        assert!(approx_eq(low, -0.5, EPS), "low {low}");
        assert!(approx_eq(high, 0.5, EPS), "high {high}");
        for v in mesh
            .vertices
            .iter()
            .filter(|v| approx_eq(v.pos[1].abs(), 2.5, EPS))
        {
            assert!(approx_eq(v.uv[0], -0.5, EPS));
        }
    }

    #[test]
    fn feathered_edge_reaches_half_a_pixel_outside() {
        // At zoom 4 half a pixel is 0.125 world units.
        let mesh = tessellate_at(&horizontal(4.0), 4.0);
        assert!(
            approx_eq(max_abs_y(&mesh), 2.125, EPS),
            "{}",
            max_abs_y(&mesh)
        );
    }

    #[test]
    fn thin_line_is_one_pixel_wide_at_any_zoom() {
        for zoom in [0.25_f32, 1.0, 4.0] {
            // The edge (coverage one half) is half a pixel from the axis, the
            // feather another half a pixel beyond it.
            let mesh = tessellate_at(&horizontal(0.01), zoom);
            assert!(
                approx_eq(max_abs_y(&mesh), 1.0 / zoom, EPS / zoom),
                "zoom {zoom}: {}",
                max_abs_y(&mesh)
            );
        }
    }

    #[test]
    fn wide_line_keeps_its_world_width_at_any_zoom() {
        for zoom in [0.25_f32, 1.0, 4.0] {
            let mesh = tessellate_at(&horizontal(40.0), zoom);
            let want = 20.0 + 0.5 / zoom;
            assert!(approx_eq(max_abs_y(&mesh), want, EPS), "zoom {zoom}");
        }
    }

    fn ellipse() -> Shape {
        Shape::Ellipse {
            a: Vec2::new(0.0, 0.0),
            b: Vec2::new(200.0, 100.0),
            style: style(2.0),
            fill: None,
            label: None,
            text: None,
        }
    }

    #[test]
    fn ellipse_segments_grow_with_zoom() {
        let far = tessellate_at(&ellipse(), 0.25).vertices.len();
        let near = tessellate_at(&ellipse(), 4.0).vertices.len();
        assert!(near > far, "{near} vertices at zoom 4, {far} at 0.25");
    }

    #[test]
    fn zoom_bucket_rounds_to_powers_of_two() {
        for (zoom, want) in [
            (0.25, 0.25),
            (1.0, 1.0),
            (4.0, 4.0),
            (1.3, 1.0),
            (1.5, 2.0),
            (0.0, 1.0),
            (f32::NAN, 1.0),
            (f32::INFINITY, 1.0),
            (-2.0, 1.0),
        ] {
            assert!(approx_eq(zoom_bucket(zoom), want, EPS), "{zoom}");
        }
    }

    #[test]
    fn arrow_shaft_stops_at_the_head_base() {
        // Arrange: width 4, so the head is `arrow_head`'s.
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(100.0, 0.0);
        let [tip, left, right] = arrow_head(a, b, 4.0);
        let base_x = left.lerp(right, 0.5).x;
        // Act
        let mesh = tessellate_at(
            &Shape::Arrow {
                a,
                b,
                style: style(4.0),
            },
            1.0,
        );
        // Assert: the head is there, and nothing but the head passes the base.
        let max_x = mesh
            .vertices
            .iter()
            .map(|v| v.pos[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(max_x >= tip.x - EPS);
        let shaft_end = mesh
            .vertices
            .iter()
            .filter(|v| v.pos[1].abs() > 2.6 || v.pos[0] < base_x - 1.0)
            .map(|v| v.pos[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(shaft_end <= base_x + 3.0, "shaft ends at {shaft_end}");
    }

    #[test]
    fn dot_grid_mesh_has_one_quad_per_dot() {
        let view = Aabb::from_corners(Vec2::new(0.0, 0.0), Vec2::new(30.0, 20.0));
        let mesh = dot_grid_mesh(view, 10.0, 2.0, [1, 2, 3, 4]);
        // x in 0, 10, 20, 30 and y in 0, 10, 20.
        assert_eq!(mesh.vertices.len(), 12 * 4);
        assert_eq!(mesh.indices.len(), 12 * 6);
        assert!(mesh.vertices.iter().all(|v| v.color == [1, 2, 3, 4]));
    }

    #[test]
    fn dot_grid_mesh_is_empty_for_too_many_dots_or_bad_spacing() {
        let view = Aabb::from_corners(Vec2::new(0.0, 0.0), Vec2::new(1.0e6, 1.0e6));
        assert_eq!(dot_grid_mesh(view, 1.0, 2.0, [0; 4]), Mesh::default());
        assert_eq!(dot_grid_mesh(view, 0.0, 2.0, [0; 4]), Mesh::default());
        assert_eq!(dot_grid_mesh(view, f32::NAN, 2.0, [0; 4]), Mesh::default());
    }

    #[test]
    fn text_shapes_tessellate_to_nothing() {
        let text = Shape::Text {
            origin: Vec2::new(0.0, 0.0),
            text: "hi".into(),
            size: 16.0,
            style: Style {
                color: ColorId::INK,
                width: 0.0,
            },
        };
        assert_eq!(tessellate(&text), Mesh::default());
    }
}
