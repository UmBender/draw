//! Rounded rectangles and rings for the menus (ADR-T126-1).
//!
//! macroquad has no rounded rectangle. These are built from triangles that
//! never overlap, so a translucent fill shows no seams where parts meet.
//! The geometry is pure ([`rounded_fill`], [`rounded_ring`]); [`fill`] and
//! [`ring`] only hand the triangles to macroquad.

use crate::core::geom::{Aabb, Vec2};
use crate::shell::render::{Batch, GlSink, MeshSink};
use macroquad::color::Color;
use std::f32::consts::FRAC_PI_2;

/// Segments of the arc of one rounded corner.
const CORNER_SEGMENTS: usize = 5;

/// A triangle in screen pixels.
pub type Triangle = [Vec2; 3];

/// The radius to use for `rect`: at most half its shorter side, never
/// negative and never not-a-number.
fn clamp_radius(rect: Aabb, radius: f32) -> f32 {
    let limit = rect.width().min(rect.height()) * 0.5;
    if radius.is_finite() {
        radius.clamp(0.0, limit.max(0.0))
    } else {
        0.0
    }
}

/// Whether `rect` has an area worth drawing.
fn drawable(rect: Aabb) -> bool {
    rect.min.is_finite() && rect.max.is_finite() && rect.width() > 0.0 && rect.height() > 0.0
}

/// The two triangles of the rectangle `min`–`max`.
fn quad(min: Vec2, max: Vec2) -> [Triangle; 2] {
    let (right_top, left_bottom) = (Vec2::new(max.x, min.y), Vec2::new(min.x, max.y));
    [[min, right_top, max], [min, max, left_bottom]]
}

/// The points of the quarter circle of radius `radius` about `centre`,
/// from the angle `start` for a quarter turn.
fn arc(centre: Vec2, radius: f32, start: f32) -> Vec<Vec2> {
    (0..=CORNER_SEGMENTS)
        .map(|i| {
            let angle = start + FRAC_PI_2 * i as f32 / CORNER_SEGMENTS as f32;
            centre + Vec2::new(radius * angle.cos(), radius * angle.sin())
        })
        .collect()
}

/// The corners of `rect` with the angle at which each arc starts:
/// top-left, top-right, bottom-right, bottom-left.
fn corners(rect: Aabb, radius: f32) -> [(Vec2, f32); 4] {
    let (min, max) = (rect.min, rect.max);
    [
        (Vec2::new(min.x + radius, min.y + radius), 2.0 * FRAC_PI_2),
        (Vec2::new(max.x - radius, min.y + radius), 3.0 * FRAC_PI_2),
        (Vec2::new(max.x - radius, max.y - radius), 0.0),
        (Vec2::new(min.x + radius, max.y - radius), FRAC_PI_2),
    ]
}

/// The triangles of `rect` with corners of `radius` (clamped to half the
/// shorter side). Empty for a rectangle without area.
#[must_use]
pub fn rounded_fill(rect: Aabb, radius: f32) -> Vec<Triangle> {
    if !drawable(rect) {
        return Vec::new();
    }
    let r = clamp_radius(rect, radius);
    let (min, max) = (rect.min, rect.max);
    if r <= 0.0 {
        return quad(min, max).to_vec();
    }
    // The full-height middle, then the two side pieces between the arcs.
    let mut out: Vec<Triangle> = Vec::new();
    out.extend(quad(
        Vec2::new(min.x + r, min.y),
        Vec2::new(max.x - r, max.y),
    ));
    out.extend(quad(
        Vec2::new(min.x, min.y + r),
        Vec2::new(min.x + r, max.y - r),
    ));
    out.extend(quad(
        Vec2::new(max.x - r, min.y + r),
        Vec2::new(max.x, max.y - r),
    ));
    for (centre, start) in corners(rect, r) {
        let points = arc(centre, r, start);
        out.extend(points.windows(2).map(|pair| [centre, pair[0], pair[1]]));
    }
    out
}

/// The triangles of the border of `rect`: a ring `width` thick, inside the
/// rectangle, with corners of outer radius `radius`.
#[must_use]
pub fn rounded_ring(rect: Aabb, radius: f32, width: f32) -> Vec<Triangle> {
    if !drawable(rect) || !width.is_finite() || width <= 0.0 {
        return Vec::new();
    }
    let width = width.min(rect.width().min(rect.height()) * 0.5);
    let r = clamp_radius(rect, radius);
    let (min, max) = (rect.min, rect.max);
    let mut out: Vec<Triangle> = Vec::new();
    // Square corners: top and bottom bars across, side bars between them.
    // Rounded corners: every bar stops where the arcs start.
    let across = if r <= 0.0 { 0.0 } else { r };
    let between = if r <= 0.0 { width } else { r };
    out.extend(quad(
        Vec2::new(min.x + across, min.y),
        Vec2::new(max.x - across, min.y + width),
    ));
    out.extend(quad(
        Vec2::new(min.x + across, max.y - width),
        Vec2::new(max.x - across, max.y),
    ));
    out.extend(quad(
        Vec2::new(min.x, min.y + between),
        Vec2::new(min.x + width, max.y - between),
    ));
    out.extend(quad(
        Vec2::new(max.x - width, min.y + between),
        Vec2::new(max.x, max.y - between),
    ));
    if r <= 0.0 {
        return out;
    }
    let inner = (r - width).max(0.0);
    for (centre, start) in corners(rect, r) {
        let outside = arc(centre, r, start);
        let inside = arc(centre, inner, start);
        for i in 0..CORNER_SEGMENTS {
            out.push([outside[i], outside[i + 1], inside[i + 1]]);
            out.push([outside[i], inside[i + 1], inside[i]]);
        }
    }
    out
}

/// Adds `triangles` in `color` to `batch`.
fn push_all<S: MeshSink>(batch: &mut Batch<S>, triangles: &[Triangle], color: Color) {
    for [a, b, c] in triangles {
        batch.triangle(*a, *b, *c, color);
    }
}

/// Adds the fill of `rect` with rounded corners to `batch`.
pub fn push_fill<S: MeshSink>(batch: &mut Batch<S>, rect: Aabb, radius: f32, color: Color) {
    push_all(batch, &rounded_fill(rect, radius), color);
}

/// Adds the `width` px border of `rect` with rounded corners to `batch`.
pub fn push_ring<S: MeshSink>(
    batch: &mut Batch<S>,
    rect: Aabb,
    radius: f32,
    width: f32,
    color: Color,
) {
    push_all(batch, &rounded_ring(rect, radius, width), color);
}

/// Fills the axis-aligned rectangle at (`x`, `y`) of `w` × `h` pixels.
pub fn rect_xywh(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let mut batch = Batch::new(GlSink);
    batch.rect(
        Aabb::from_corners(Vec2::new(x, y), Vec2::new(x + w, y + h)),
        color,
    );
    batch.finish();
}

/// Fills `rect` with rounded corners.
pub fn fill(rect: Aabb, radius: f32, color: Color) {
    let mut batch = Batch::new(GlSink);
    push_fill(&mut batch, rect, radius, color);
    batch.finish();
}

/// Draws the `width` px border of `rect` with rounded corners.
pub fn ring(rect: Aabb, radius: f32, width: f32, color: Color) {
    let mut batch = Batch::new(GlSink);
    push_ring(&mut batch, rect, radius, width, color);
    batch.finish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;
    use proptest::prelude::*;
    use std::f32::consts::PI;

    const EPS: f32 = 1e-2;

    use draw_gfx::gpu::Vertex;
    use crate::shell::render::{Batch, MeshSink};

    /// A sink that counts the triangles sent to it.
    #[derive(Default)]
    struct Counter {
        triangles: usize,
    }

    impl MeshSink for Counter {
        fn draw(&mut self, _vertices: &[Vertex], indices: &[u32], _page: Option<u32>) {
            self.triangles += indices.len() / 3;
        }
    }

    #[test]
    fn push_fill_emits_the_triangles_of_the_fill() {
        // Arrange
        let r = rect(80.0, 40.0);
        let expected = rounded_fill(r, 8.0).len();
        let mut batch = Batch::new(Counter::default());

        // Act
        push_fill(&mut batch, r, 8.0, Color::new(1.0, 0.0, 0.0, 1.0));
        let sink = batch.finish();

        // Assert
        assert!(expected > 0);
        assert_eq!(sink.triangles, expected);
    }

    #[test]
    fn push_ring_emits_the_triangles_of_the_ring() {
        // Arrange
        let r = rect(80.0, 40.0);
        let expected = rounded_ring(r, 8.0, 2.0).len();
        let mut batch = Batch::new(Counter::default());

        // Act
        push_ring(&mut batch, r, 8.0, 2.0, Color::new(0.0, 1.0, 0.0, 1.0));
        let sink = batch.finish();

        // Assert
        assert!(expected > 0);
        assert_eq!(sink.triangles, expected);
    }

    #[test]
    fn toolbar_and_menus_use_no_macroquad_draw_calls() {
        // Arrange: the code of the four files, without their test modules.
        let code = |source: &'static str| source.split("#[cfg(test)]").next().unwrap_or("");
        let files = [
            ("toolbar.rs", code(include_str!("toolbar.rs"))),
            ("sheet.rs", code(include_str!("sheet.rs"))),
            ("command_palette.rs", code(include_str!("command_palette.rs"))),
            ("ui_shapes.rs", code(include_str!("ui_shapes.rs"))),
        ];
        let forbidden = [
            "draw_rectangle(",
            "draw_triangle(",
            "draw_line(",
            "draw_mesh(",
            "draw_texture_ex(",
        ];

        // Act / Assert
        for (name, source) in files {
            for call in forbidden {
                assert!(!source.contains(call), "{name} uses {call}");
            }
        }
    }

    fn rect(w: f32, h: f32) -> Aabb {
        Aabb::from_corners(Vec2::new(10.0, 20.0), Vec2::new(10.0 + w, 20.0 + h))
    }

    /// The area of a triangle.
    fn area(t: &Triangle) -> f32 {
        let (a, b, c) = (t[0], t[1], t[2]);
        ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs() * 0.5
    }

    fn total(triangles: &[Triangle]) -> f32 {
        triangles.iter().map(area).sum()
    }

    /// Whether two triangles share area: a point inside both, found by
    /// sampling the first one's interior.
    fn overlap(first: &Triangle, second: &Triangle) -> bool {
        let inside = |t: &Triangle, point: Vec2| {
            let sign = |p1: Vec2, p2: Vec2, p3: Vec2| {
                (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
            };
            let (d1, d2, d3) = (
                sign(point, t[0], t[1]),
                sign(point, t[1], t[2]),
                sign(point, t[2], t[0]),
            );
            let neg = d1 < -1e-3 || d2 < -1e-3 || d3 < -1e-3;
            let pos = d1 > 1e-3 || d2 > 1e-3 || d3 > 1e-3;
            !(neg && pos)
        };
        // Barycentric samples well inside `first`.
        let steps = 6;
        (1..steps).any(|i| {
            (1..steps - i).any(|j| {
                let (across, down) = (i as f32 / steps as f32, j as f32 / steps as f32);
                let point = first[0] * (1.0 - across - down) + first[1] * across + first[2] * down;
                inside(second, point)
            })
        })
    }

    #[test]
    fn rounded_fill_covers_the_rounded_rectangle() {
        // Arrange
        let (w, h, r) = (120.0, 60.0, 10.0);

        // Act
        let triangles = rounded_fill(rect(w, h), r);

        // Assert: the rectangle minus the four corner gaps (the arcs are
        // polygons, so a hair less than the circle).
        let expected = w * h - (4.0 - PI) * r * r;
        assert!(
            (total(&triangles) - expected).abs() < 0.01 * expected,
            "{}",
            total(&triangles)
        );
        let bounds = rect(w, h);
        for t in &triangles {
            for p in t {
                assert!(p.x >= bounds.min.x - EPS && p.x <= bounds.max.x + EPS);
                assert!(p.y >= bounds.min.y - EPS && p.y <= bounds.max.y + EPS);
            }
        }
    }

    #[test]
    fn rounded_fill_never_overlaps_itself() {
        let triangles = rounded_fill(rect(80.0, 40.0), 8.0);
        for (i, a) in triangles.iter().enumerate() {
            for b in &triangles[i + 1..] {
                assert!(!overlap(a, b), "{a:?} / {b:?}");
            }
        }
    }

    #[test]
    fn rounded_ring_covers_the_border() {
        // Arrange
        let (w, h, r, width) = (100.0, 50.0, 8.0, 2.0);

        // Act
        let ring = rounded_ring(rect(w, h), r, width);

        // Assert: the outer rounded area minus the inner one.
        let outer = w * h - (4.0 - PI) * r * r;
        let (iw, ih, ir) = (w - 2.0 * width, h - 2.0 * width, r - width);
        let inner = iw * ih - (4.0 - PI) * ir * ir;
        let expected = outer - inner;
        assert!(
            (total(&ring) - expected).abs() < 0.02 * expected,
            "{}",
            total(&ring)
        );
        for (i, a) in ring.iter().enumerate() {
            for b in &ring[i + 1..] {
                assert!(!overlap(a, b), "{a:?} / {b:?}");
            }
        }
    }

    #[test]
    fn radius_is_clamped_and_zero_is_a_rectangle() {
        // Arrange
        let square = rect(40.0, 20.0);

        // Act / Assert: no radius, a plain rectangle.
        assert!(approx_eq(total(&rounded_fill(square, 0.0)), 800.0, EPS));
        assert!(approx_eq(total(&rounded_fill(square, -5.0)), 800.0, EPS));
        assert!(approx_eq(
            total(&rounded_fill(square, f32::NAN)),
            800.0,
            EPS
        ));
        // A huge radius becomes half the shorter side: a stadium.
        let stadium = rounded_fill(square, 1e6);
        let expected = 800.0 - (4.0 - PI) * 100.0;
        assert!((total(&stadium) - expected).abs() < 0.01 * expected);
        // A zero-radius ring is four plain bars.
        let bars = rounded_ring(square, 0.0, 1.0);
        assert!(approx_eq(total(&bars), 40.0 * 20.0 - 38.0 * 18.0, EPS));
    }

    #[test]
    fn degenerate_rectangles_give_no_triangles() {
        let flat = Aabb::from_corners(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0));
        assert_eq!(rounded_fill(flat, 3.0).len(), 0);
        assert_eq!(rounded_ring(flat, 3.0, 1.0).len(), 0);
        assert_eq!(rounded_ring(rect(10.0, 10.0), 3.0, 0.0).len(), 0);
        assert_eq!(rounded_ring(rect(10.0, 10.0), 3.0, f32::NAN).len(), 0);
        let wild = Aabb::from_corners(Vec2::new(f32::NAN, 0.0), Vec2::new(10.0, 10.0));
        assert_eq!(rounded_fill(wild, 3.0).len(), 0);
    }

    proptest! {
        #[test]
        fn shapes_are_finite_for_any_input(
            w in 0.0_f32..400.0,
            h in 0.0_f32..400.0,
            radius in -10.0_f32..300.0,
            width in -2.0_f32..20.0,
        ) {
            for t in rounded_fill(rect(w, h), radius).iter().chain(&rounded_ring(rect(w, h), radius, width)) {
                prop_assert!(t.iter().all(|p| p.is_finite()));
            }
        }
    }
}
