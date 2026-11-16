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
    if !shape.is_finite() {
        return Mesh::default();
    }
    let mut b = Builder::default();
    match shape {
        Shape::Stroke {
            points,
            widths,
            style,
        } => {
            let color = rgba(style.color);
            let width_at = |i: usize| {
                let factor = widths.as_ref().and_then(|w| w.get(i)).copied();
                style.width * factor.unwrap_or(1.0)
            };
            for (i, p) in points.iter().enumerate() {
                b.disc(*p, width_at(i) / 2.0, color);
                if let Some(q) = points.get(i + 1) {
                    b.segment(*p, *q, width_at(i), width_at(i + 1), color);
                }
            }
        }
        Shape::Line { a, b: end, style } => {
            b.segment(*a, *end, style.width, style.width, rgba(style.color));
        }
        Shape::Arrow { a, b: end, style } => {
            let color = rgba(style.color);
            b.segment(*a, *end, style.width, style.width, color);
            let [tip, left, right] = arrow_head(*a, *end, style.width);
            b.triangle(tip, left, right, color);
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
                b.rect(bounds, rgba(*fill));
            }
            b.outline(&rect_corners(bounds), style.width, rgba(style.color));
        }
        Shape::Ellipse {
            a,
            b: end,
            style,
            fill,
            ..
        } => {
            let bounds = Aabb::from_corners(*a, *end);
            let rim = ellipse_rim(bounds);
            if let Some(fill) = fill {
                b.fan(bounds.center(), &rim, rgba(*fill));
            }
            b.outline(&rim, style.width, rgba(style.color));
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
                b.rect(rect, rgba(cell.color));
            }
            let color = rgba(style.color);
            for [p, q] in grid_lines(*a, *end, *cols, *rows) {
                b.segment(p, q, style.width, style.width, color);
            }
        }
        Shape::Text { .. } => {}
    }
    b.mesh
}

/// Segments of the rim of an ellipse and of a join disc.
const ELLIPSE_SEGMENTS: usize = 48;
const DISC_SEGMENTS: usize = 8;

fn rgba(id: ColorId) -> [u8; 4] {
    let c = palette(id);
    [c.r, c.g, c.b, c.a]
}

fn rect_corners(r: Aabb) -> [Vec2; 4] {
    [
        r.min,
        Vec2::new(r.max.x, r.min.y),
        r.max,
        Vec2::new(r.min.x, r.max.y),
    ]
}

fn ellipse_rim(r: Aabb) -> Vec<Vec2> {
    let (c, rx, ry) = (r.center(), r.width() / 2.0, r.height() / 2.0);
    (0..ELLIPSE_SEGMENTS)
        .map(|i| {
            let t = std::f32::consts::TAU * i as f32 / ELLIPSE_SEGMENTS as f32;
            Vec2::new(c.x + rx * t.cos(), c.y + ry * t.sin())
        })
        .collect()
}

#[derive(Default)]
struct Builder {
    mesh: Mesh,
}

impl Builder {
    fn vertex(&mut self, p: Vec2, color: [u8; 4]) -> u32 {
        let index = u32::try_from(self.mesh.vertices.len()).unwrap_or(u32::MAX);
        self.mesh.vertices.push(Vertex {
            pos: [p.x, p.y],
            uv: [0.0, 0.0],
            color,
        });
        index
    }

    fn triangle(&mut self, a: Vec2, b: Vec2, c: Vec2, color: [u8; 4]) {
        let i = self.vertex(a, color);
        self.vertex(b, color);
        self.vertex(c, color);
        self.mesh.indices.extend([i, i + 1, i + 2]);
    }

    fn quad(&mut self, corners: [Vec2; 4], color: [u8; 4]) {
        let i = self.vertex(corners[0], color);
        for c in &corners[1..] {
            self.vertex(*c, color);
        }
        self.mesh.indices.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
    }

    fn rect(&mut self, r: Aabb, color: [u8; 4]) {
        self.quad(rect_corners(r), color);
    }

    /// A segment with the given widths at its ends; nothing if degenerate.
    fn segment(&mut self, a: Vec2, b: Vec2, wa: f32, wb: f32, color: [u8; 4]) {
        let d = b - a;
        let len = d.length();
        if len <= f32::EPSILON {
            return;
        }
        let n = Vec2::new(-d.y, d.x) / len;
        let (ha, hb) = (wa.max(0.0) / 2.0, wb.max(0.0) / 2.0);
        self.quad([a + n * ha, b + n * hb, b - n * hb, a - n * ha], color);
    }

    fn disc(&mut self, center: Vec2, radius: f32, color: [u8; 4]) {
        if radius <= 0.0 {
            return;
        }
        let rim: Vec<Vec2> = (0..DISC_SEGMENTS)
            .map(|i| {
                let t = std::f32::consts::TAU * i as f32 / DISC_SEGMENTS as f32;
                center + Vec2::new(t.cos(), t.sin()) * radius
            })
            .collect();
        self.fan(center, &rim, color);
    }

    fn fan(&mut self, center: Vec2, rim: &[Vec2], color: [u8; 4]) {
        let c = self.vertex(center, color);
        for p in rim {
            self.vertex(*p, color);
        }
        let n = u32::try_from(rim.len()).unwrap_or(0);
        for k in 0..n {
            self.mesh
                .indices
                .extend([c, c + 1 + k, c + 1 + (k + 1) % n]);
        }
    }

    /// A closed outline through `points`, with a disc at each corner.
    fn outline(&mut self, points: &[Vec2], width: f32, color: [u8; 4]) {
        for (i, p) in points.iter().enumerate() {
            let q = points[(i + 1) % points.len()];
            self.segment(*p, q, width, width, color);
            self.disc(*p, width / 2.0, color);
        }
    }
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
        for v in mesh.vertices.iter().filter(|v| approx_eq(v.pos[1].abs(), 2.5, EPS)) {
            assert!(approx_eq(v.uv[0], -0.5, EPS));
        }
    }

    #[test]
    fn feathered_edge_reaches_half_a_pixel_outside() {
        // At zoom 4 half a pixel is 0.125 world units.
        let mesh = tessellate_at(&horizontal(4.0), 4.0);
        assert!(approx_eq(max_abs_y(&mesh), 2.125, EPS), "{}", max_abs_y(&mesh));
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
