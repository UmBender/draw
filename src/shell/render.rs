//! Draws shapes, previews and selection with macroquad.
//!
//! Everything is drawn in **screen space** with macroquad's default camera:
//! world points go through [`Camera::world_to_screen`] and widths through
//! [`screen_width`] (ADR-T07-1). Off-screen shapes are culled by their
//! bounding box (ADR-0006). Shapes are tessellated here into a [`Batch`] and
//! submitted with one `draw_mesh` per few thousand indices instead of one
//! `draw_*` call per triangle (ADR-T24-3). Grids and labels follow
//! ADR-T16-1.

use macroquad::color::Color;
use macroquad::math::Vec2 as MqVec2;
use macroquad::models::{Mesh, Vertex, draw_mesh};
use macroquad::shapes::draw_line;
use macroquad::text::{TextParams, draw_text_ex, measure_text};

use crate::core::camera::Camera;
use crate::core::editor::Editor;
use crate::core::geom::{Aabb, Vec2};
use crate::core::palette::{ColorId, Rgba, THEME, palette};
use crate::core::shape::{
    GRID_MAX_CELLS, Shape, arrow_head, grid_axis_labels, grid_cell_rect, grid_lines,
};
use crate::core::snap::GRID_STEP;

/// Most vertices one [`Batch`] chunk holds; below macroquad's per-draw-call
/// capacity of 10 000 (ADR-T24-3).
pub const BATCH_MAX_VERTICES: usize = 4_800;

/// Most indices one [`Batch`] chunk holds; below macroquad's per-draw-call
/// capacity of 5 000 and a multiple of 6, so whole quads fit (ADR-T24-3).
pub const BATCH_MAX_INDICES: usize = 4_800;

/// Where a [`Batch`] sends finished chunks.
pub trait MeshSink {
    /// Draws (or records) one chunk of triangles.
    fn draw(&mut self, mesh: &Mesh);
}

/// Sends chunks to macroquad with `draw_mesh`.
#[derive(Debug, Clone, Copy, Default)]
pub struct GlSink;

impl MeshSink for GlSink {
    fn draw(&mut self, mesh: &Mesh) {
        draw_mesh(mesh);
    }
}

/// Collects untextured triangles and submits them in chunks of at most
/// [`BATCH_MAX_VERTICES`] / [`BATCH_MAX_INDICES`] (ADR-T24-3). Callers
/// [`Batch::flush`] before drawing anything else (text) to keep z-order.
pub struct Batch<S: MeshSink = GlSink> {
    /// The chunk being filled.
    mesh: Mesh,
    /// Receives finished chunks.
    sink: S,
}

impl<S: MeshSink> Batch<S> {
    /// An empty batch sending chunks to `sink`.
    pub fn new(sink: S) -> Self {
        Self {
            mesh: Mesh {
                vertices: Vec::with_capacity(BATCH_MAX_VERTICES),
                indices: Vec::with_capacity(BATCH_MAX_INDICES),
                texture: None,
            },
            sink,
        }
    }

    /// Adds triangle `a b c`.
    pub fn triangle(&mut self, a: Vec2, b: Vec2, c: Vec2, color: Color) {
        let base = self.reserve(3, 3);
        for p in [a, b, c] {
            self.vertex(p, color);
        }
        self.mesh.indices.extend([base, base + 1, base + 2]);
    }

    /// Adds an axis-aligned rectangle as one quad.
    pub fn rect(&mut self, rect: Aabb, color: Color) {
        let (min, max) = (rect.min, rect.max);
        self.quad(
            [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)],
            color,
        );
    }

    /// Adds segment `a → b` as a quad `width` pixels wide with flat ends,
    /// like macroquad's `draw_line`. A zero-length segment adds nothing.
    pub fn line(&mut self, a: Vec2, b: Vec2, width: f32, color: Color) {
        let d = b - a;
        let len = d.length();
        if !len.is_finite() || len <= 0.0 {
            return;
        }
        let n = Vec2::new(-d.y, d.x) * (width * 0.5 / len);
        self.quad([a + n, b + n, b - n, a - n], color);
    }

    /// Adds a triangle fan around `center`: one triangle per consecutive
    /// pair of `rim` points (pass the first point again at the end to close
    /// the loop).
    pub fn fan(&mut self, center: Vec2, rim: impl ExactSizeIterator<Item = Vec2>, color: Color) {
        let points = rim.len();
        if points < 2 {
            return;
        }
        let base = self.reserve(1 + points, 3 * (points - 1));
        self.vertex(center, color);
        for p in rim {
            self.vertex(p, color);
        }
        for i in 1..points as u16 {
            self.mesh.indices.extend([base, base + i, base + i + 1]);
        }
    }

    /// Adds a strip of quads between consecutive `(outer, inner)` pairs. A
    /// strip longer than a chunk continues in the next one (ADR-T24-4).
    pub fn strip(&mut self, pairs: impl IntoIterator<Item = (Vec2, Vec2)>, color: Color) {
        let mut last = None;
        for pair in pairs {
            last = Some(self.strip_push(last, pair, color));
        }
    }

    /// Adds a polyline of screen `points` `width` pixels wide as one strip
    /// with mitred joins, skipping points closer than
    /// [`STROKE_MIN_STEP_PX`] to the last kept one. Where the mitre would
    /// exceed [`MITER_LIMIT`] half-widths the strip breaks; thick lines
    /// ([`stroke_needs_joints`]) get round discs at breaks and ends. A
    /// single point is a dot (ADR-T24-4).
    pub fn polyline(&mut self, points: impl IntoIterator<Item = Vec2>, width: f32, color: Color) {
        let h = width * 0.5;
        let round = stroke_needs_joints(width);
        let mut points = points.into_iter();
        let Some(first) = points.next() else {
            return;
        };
        let (mut prev, mut prev_normal) = (first, None::<Vec2>);
        let mut last = None;
        for p in points {
            let d = p - prev;
            let len = d.length();
            // Also skips non-finite steps.
            if !len.is_finite() || len < STROKE_MIN_STEP_PX {
                continue;
            }
            let n = Vec2::new(-d.y, d.x) * (1.0 / len);
            match prev_normal {
                None => {
                    if round {
                        self.disc(prev, h, color);
                    }
                    last = Some(self.strip_push(None, (prev + n * h, prev - n * h), color));
                }
                Some(n0) => {
                    // cos² of half the turn; the mitre is h / cos(turn / 2).
                    let c = n0.dot(n);
                    if (1.0 + c) * 0.5 * MITER_LIMIT * MITER_LIMIT >= 1.0 {
                        let offset = (n0 + n) * (h / (1.0 + c));
                        last = Some(self.strip_push(last, (prev + offset, prev - offset), color));
                    } else {
                        self.strip_push(last, (prev + n0 * h, prev - n0 * h), color);
                        if round {
                            self.disc(prev, h, color);
                        }
                        last = Some(self.strip_push(None, (prev + n * h, prev - n * h), color));
                    }
                }
            }
            (prev, prev_normal) = (p, Some(n));
        }
        match prev_normal {
            None => self.disc(first, h, color),
            Some(n0) => {
                self.strip_push(last, (prev + n0 * h, prev - n0 * h), color);
                if round {
                    self.disc(prev, h, color);
                }
            }
        }
    }

    /// Adds a filled disc of `radius` pixels centred at `center`.
    pub fn disc(&mut self, center: Vec2, radius: f32, color: Color) {
        let n = circle_segments(radius);
        self.fan(center, unit_circle(n).map(|u| center + u * radius), color);
    }

    /// Sends the current chunk to the sink, if it holds anything.
    pub fn flush(&mut self) {
        if !self.mesh.indices.is_empty() {
            self.sink.draw(&self.mesh);
        }
        self.mesh.vertices.clear();
        self.mesh.indices.clear();
    }

    /// Flushes and returns the sink.
    pub fn finish(mut self) -> S {
        self.flush();
        self.sink
    }

    /// Adds a quad with `corners` in order around it.
    fn quad(&mut self, corners: [Vec2; 4], color: Color) {
        let base = self.reserve(4, 6);
        for p in corners {
            self.vertex(p, color);
        }
        self.mesh
            .indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Adds `pair` to a strip whose previous pair is `last` (its first
    /// vertex index in the current chunk, and its points), joined by a quad;
    /// `None` starts a strip. If the quad does not fit, the chunk is flushed
    /// and the previous pair repeated in the next one. Returns the new last.
    fn strip_push(
        &mut self,
        last: Option<(u16, (Vec2, Vec2))>,
        pair: (Vec2, Vec2),
        color: Color,
    ) -> (u16, (Vec2, Vec2)) {
        let Some((mut prev_index, prev)) = last else {
            // Room for the first quad too, so the pair is not stranded.
            let base = self.reserve(4, 6);
            self.vertex(pair.0, color);
            self.vertex(pair.1, color);
            return (base, pair);
        };
        let mesh = &self.mesh;
        if mesh.vertices.len() + 2 > BATCH_MAX_VERTICES
            || mesh.indices.len() + 6 > BATCH_MAX_INDICES
        {
            self.flush();
            prev_index = 0;
            self.vertex(prev.0, color);
            self.vertex(prev.1, color);
        }
        // At most BATCH_MAX_VERTICES, which fits in u16.
        let base = self.mesh.vertices.len() as u16;
        self.vertex(pair.0, color);
        self.vertex(pair.1, color);
        let (o0, i0, o1, i1) = (prev_index, prev_index + 1, base, base + 1);
        self.mesh.indices.extend([o0, o1, i0, i0, o1, i1]);
        (base, pair)
    }

    /// Makes room for `vertices` more vertices and `indices` more indices,
    /// flushing first if they would not fit. Returns the index the first new
    /// vertex gets. A single fan or quad never exceeds a chunk: the largest
    /// is a disc of [`MAX_SEGMENTS`] segments; strips continue across chunks.
    fn reserve(&mut self, vertices: usize, indices: usize) -> u16 {
        let mesh = &self.mesh;
        if mesh.vertices.len() + vertices > BATCH_MAX_VERTICES
            || mesh.indices.len() + indices > BATCH_MAX_INDICES
        {
            self.flush();
        }
        // At most BATCH_MAX_VERTICES, which fits in u16.
        self.mesh.vertices.len() as u16
    }

    /// Appends one untextured vertex at screen point `p`.
    fn vertex(&mut self, p: Vec2, color: Color) {
        self.mesh
            .vertices
            .push(Vertex::new(p.x, p.y, 0.0, 0.0, 0.0, color));
    }
}

/// Smallest on-screen outline width in pixels (ADR-0013).
pub const MIN_SCREEN_WIDTH_PX: f32 = 1.0;

/// Strokes wider than this many pixels get round joints and caps.
pub const JOINT_THRESHOLD_PX: f32 = 2.0;

/// Polyline points closer than this many pixels to the last kept point are
/// skipped (the chord tolerance, ADR-T24-4).
pub const STROKE_MIN_STEP_PX: f32 = CHORD_TOLERANCE_PX;

/// Longest polyline mitre, in half-widths, before the strip breaks
/// (ADR-T24-4).
pub const MITER_LIMIT: f32 = 1.2;

/// Extra pixels around the viewport kept when culling, covering the
/// overhang of outlines clamped to [`MIN_SCREEN_WIDTH_PX`].
pub const CULL_MARGIN_PX: f32 = 2.0;

/// Largest allowed distance in pixels between a circle and its polygon.
pub const CHORD_TOLERANCE_PX: f32 = 0.25;

/// Fewest segments used for a disc or ellipse.
pub const MIN_SEGMENTS: u16 = 8;

/// Most segments used for a disc or ellipse.
pub const MAX_SEGMENTS: u16 = 256;

/// Width of the selection outline in pixels, independent of zoom.
pub const SELECTION_WIDTH_PX: f32 = 1.0;

/// Gap in pixels between the selected shapes' bounds and the selection outline.
pub const SELECTION_PAD_PX: f32 = 4.0;

/// Labels smaller than this many pixels on screen are not drawn.
pub const LABEL_MIN_PX: f32 = 8.0;

/// Largest label font size in pixels.
pub const LABEL_MAX_PX: f32 = 256.0;

/// Label font size as a fraction of the label area's height.
pub const LABEL_HEIGHT_RATIO: f32 = 0.6;

/// Fraction of the label area's width the text may use.
pub const LABEL_WIDTH_RATIO: f32 = 0.8;

/// Width of one digit per pixel of font size (an upper bound for the
/// default font's digits).
pub const LABEL_CHAR_ASPECT: f32 = 0.6;

/// Font sizes labels are rasterized at, ascending; other sizes are scaled
/// from the nearest one at or above (or the largest).
pub const LABEL_RASTER_SIZES: [u16; 4] = [16, 32, 64, 128];

/// Smallest distance in pixels between two dots of the snap dot grid.
pub const DOT_GRID_MIN_PX: f32 = 12.0;

/// Side of one dot of the snap dot grid, in pixels.
pub const DOT_SIZE_PX: f32 = 2.0;

/// Opacity of the dot grid (the theme border colour, faded).
pub const DOT_GRID_ALPHA: f32 = 0.35;

/// Most dots drawn in one frame.
pub const DOT_GRID_MAX_DOTS: usize = 100_000;

/// Width of an alignment guide in pixels.
pub const GUIDE_WIDTH_PX: f32 = 1.0;

/// World distance between drawn dots at `zoom`: [`GRID_STEP`] doubled until
/// the dots are at least [`DOT_GRID_MIN_PX`] apart on screen. `None` for a
/// non-finite or non-positive zoom, or one so small no spacing fits.
#[must_use]
pub fn dot_grid_spacing(zoom: f32) -> Option<f32> {
    /// Doublings tried before giving up (2^64 covers any usable zoom).
    const MAX_DOUBLINGS: u8 = 64;
    if !(zoom.is_finite() && zoom > 0.0) {
        return None;
    }
    let mut spacing = GRID_STEP;
    for _ in 0..MAX_DOUBLINGS {
        if spacing * zoom >= DOT_GRID_MIN_PX {
            return Some(spacing);
        }
        spacing *= 2.0;
    }
    None
}

/// The multiples of `spacing` on both axes inside `view`, row by row, at
/// most [`DOT_GRID_MAX_DOTS`]. Empty for a non-finite view or a spacing
/// that is not finite and positive.
pub fn dot_grid_points(view: Aabb, spacing: f32) -> impl Iterator<Item = Vec2> {
    let valid =
        spacing.is_finite() && spacing > 0.0 && view.min.is_finite() && view.max.is_finite();
    // Index range of the multiples of `spacing` in `lo..=hi`; the casts
    // saturate, and the dot cap bounds the work.
    let range = |lo: f32, hi: f32| {
        if valid {
            ((lo / spacing).ceil() as i64, (hi / spacing).floor() as i64)
        } else {
            (0, -1)
        }
    };
    let (x0, x1) = range(view.min.x, view.max.x);
    let (y0, y1) = range(view.min.y, view.max.y);
    let cols = usize::try_from(x1.saturating_sub(x0).saturating_add(1)).unwrap_or(0);
    let rows = usize::try_from(y1.saturating_sub(y0).saturating_add(1)).unwrap_or(0);
    let total = if cols.saturating_mul(rows) > DOT_GRID_MAX_DOTS {
        0
    } else {
        cols * rows
    };
    (0..total).map(move |i| {
        let col = x0.saturating_add(i64::try_from(i % cols).unwrap_or(0));
        let row = y0.saturating_add(i64::try_from(i / cols).unwrap_or(0));
        Vec2::new(col as f32 * spacing, row as f32 * spacing)
    })
}

/// On-screen outline width in pixels for a world-space `world_width`:
/// `world_width * zoom`, at least [`MIN_SCREEN_WIDTH_PX`]. A NaN, infinite or
/// negative product gives [`MIN_SCREEN_WIDTH_PX`].
#[must_use]
pub fn screen_width(world_width: f32, zoom: f32) -> f32 {
    let px = world_width * zoom;
    if px.is_finite() && px >= MIN_SCREEN_WIDTH_PX {
        px
    } else {
        MIN_SCREEN_WIDTH_PX
    }
}

/// `true` if `bounds` overlaps or touches `view`; `false` if either box has a
/// NaN coordinate (every comparison with NaN is false).
#[must_use]
pub fn is_visible(bounds: Aabb, view: Aabb) -> bool {
    bounds.intersects(&view)
}

/// World rectangle used for culling: the area visible through a viewport of
/// `viewport` pixels, grown by [`CULL_MARGIN_PX`] (in world units).
#[must_use]
pub fn cull_rect(camera: &Camera, viewport: Vec2) -> Aabb {
    camera
        .visible_world_rect(viewport)
        .expand(camera.world_len(CULL_MARGIN_PX))
}

/// Converts a core vector to a macroquad vector.
#[must_use]
pub fn to_mq(v: Vec2) -> MqVec2 {
    MqVec2::new(v.x, v.y)
}

/// Converts a palette colour to a macroquad colour (channels in `0..=1`).
#[must_use]
pub fn to_mq_color(rgba: Rgba) -> Color {
    let [red, green, blue, alpha] = rgba.to_f32();
    Color::new(red, green, blue, alpha)
}

/// `true` if a stroke `width_px` wide needs round joints, i.e. it is wider
/// than [`JOINT_THRESHOLD_PX`]. Thinner strokes skip them to stay cheap.
#[must_use]
pub fn stroke_needs_joints(width_px: f32) -> bool {
    width_px > JOINT_THRESHOLD_PX
}

/// Number of segments for a circle of `radius_px` pixels: the fewest whose
/// chord error `r (1 − cos(π/n))` is at most [`CHORD_TOLERANCE_PX`], clamped
/// to [`MIN_SEGMENTS`]`..=`[`MAX_SEGMENTS`]. A non-finite or non-positive
/// radius gives [`MIN_SEGMENTS`].
#[must_use]
pub fn circle_segments(radius_px: f32) -> u16 {
    if !radius_px.is_finite() || radius_px <= 0.0 {
        return MIN_SEGMENTS;
    }
    let ratio = f64::from(CHORD_TOLERANCE_PX) / f64::from(radius_px);
    if ratio >= 1.0 {
        return MIN_SEGMENTS;
    }
    // r (1 − cos(π/n)) ≤ tol  ⇔  π/n ≤ acos(1 − tol/r).
    let n = (std::f64::consts::PI / (1.0 - ratio).acos()).ceil();
    let (lo, hi) = (f64::from(MIN_SEGMENTS), f64::from(MAX_SEGMENTS));
    // Clamped to a u16 range, so the cast is exact.
    n.clamp(lo, hi) as u16
}

/// Screen rectangle of the selection outline: `bounds` mapped to pixels and
/// grown by [`SELECTION_PAD_PX`].
#[must_use]
pub fn selection_rect(bounds: Aabb, camera: &Camera) -> Aabb {
    Aabb::from_corners(
        camera.world_to_screen(bounds.min),
        camera.world_to_screen(bounds.max),
    )
    .expand(SELECTION_PAD_PX)
}

/// Font size in pixels for a label of `chars` characters inside an area of
/// `area` pixels: as tall as [`LABEL_HEIGHT_RATIO`] of the height, narrowed
/// so the text fits [`LABEL_WIDTH_RATIO`] of the width, capped at
/// [`LABEL_MAX_PX`]. `None` (not drawn) below [`LABEL_MIN_PX`] or for a
/// non-finite area.
#[must_use]
pub fn label_size(chars: usize, area: Vec2) -> Option<f32> {
    if !area.is_finite() {
        return None;
    }
    let text_width = chars.max(1) as f32 * LABEL_CHAR_ASPECT;
    let size = (area.y * LABEL_HEIGHT_RATIO)
        .min(area.x * LABEL_WIDTH_RATIO / text_width)
        .min(LABEL_MAX_PX);
    (size >= LABEL_MIN_PX).then_some(size)
}

/// Font size in pixels shared by every axis index of a `cols × rows` grid
/// whose cells are `cell` pixels on screen: [`label_size`] of the longest
/// index, `max(cols, rows) - 1` (ADR-T18-3).
#[must_use]
pub fn axis_label_size(cols: u32, rows: u32, cell: Vec2) -> Option<f32> {
    let longest = cols.max(rows).clamp(1, GRID_MAX_CELLS) - 1;
    label_size(longest.to_string().len(), cell)
}

/// Size in pixels of the area a label may use inside the screen rectangle
/// `rect` of a shape: the rectangle itself, or for an ellipse the largest
/// axis-aligned box inscribed in it (`rect` scaled by `1/√2`).
#[must_use]
pub fn label_area(rect: Aabb, ellipse: bool) -> Vec2 {
    let size = Vec2::new(rect.width(), rect.height());
    if ellipse {
        size * std::f32::consts::FRAC_1_SQRT_2
    } else {
        size
    }
}

/// Raster size and scale for drawing text at `size` pixels: the smallest of
/// [`LABEL_RASTER_SIZES`] at least `size` (else the largest), and the scale
/// mapping it to `size`. A NaN `size` gives the largest raster and a NaN
/// scale; callers pass sizes from [`label_size`], which are finite.
#[must_use]
pub fn label_raster(size: f32) -> (u16, f32) {
    let largest = LABEL_RASTER_SIZES[LABEL_RASTER_SIZES.len() - 1];
    let raster = LABEL_RASTER_SIZES
        .into_iter()
        .find(|&r| f32::from(r) >= size)
        .unwrap_or(largest);
    (raster, size / f32::from(raster))
}

/// Draws what lies under the shapes: the snap dot grid while grid snap is
/// on (ADR-T17-1), as seen through `camera` in a viewport of `viewport`
/// pixels. Nothing is drawn when the view has too many dots.
pub fn draw_underlay(editor: &Editor, camera: &Camera, viewport: Vec2) {
    if !editor.helpers().grid_snap {
        return;
    }
    let Some(spacing) = dot_grid_spacing(camera.zoom()) else {
        return;
    };
    let mut color = to_mq_color(THEME.border);
    color.a *= DOT_GRID_ALPHA;
    let half = Vec2::new(DOT_SIZE_PX / 2.0, DOT_SIZE_PX / 2.0);
    let mut batch = Batch::new(GlSink);
    for dot in dot_grid_points(camera.visible_world_rect(viewport), spacing) {
        let p = camera.world_to_screen(dot);
        batch.rect(Aabb::from_corners(p - half, p + half), color);
    }
    batch.finish();
}

/// Draws alignment guides, world-space segments, on top of the overlay,
/// [`GUIDE_WIDTH_PX`] wide in the theme accent colour.
pub fn draw_guides(guides: &[[Vec2; 2]], camera: &Camera) {
    let color = to_mq_color(THEME.accent);
    for [a, b] in guides {
        let (a, b) = (camera.world_to_screen(*a), camera.world_to_screen(*b));
        draw_line(a.x, a.y, b.x, b.y, GUIDE_WIDTH_PX, color);
    }
}

/// Draws every shape whose bounds are visible through `camera` in a viewport
/// of `viewport` pixels, in iteration order (later shapes on top).
pub fn draw_shapes<'a>(
    shapes: impl IntoIterator<Item = &'a Shape>,
    camera: &Camera,
    viewport: Vec2,
) {
    let view = cull_rect(camera, viewport);
    let mut batch = Batch::new(GlSink);
    for shape in shapes {
        if is_visible(shape.bounds(), view) {
            draw_shape(&mut batch, shape, camera);
        }
    }
    batch.finish();
}

/// Draws the shape being created, without culling.
pub fn draw_preview(shape: &Shape, camera: &Camera) {
    let mut batch = Batch::new(GlSink);
    draw_shape(&mut batch, shape, camera);
    batch.finish();
}

/// Draws the selection outline around world `bounds`, [`SELECTION_WIDTH_PX`]
/// wide in the theme accent colour.
pub fn draw_selection(bounds: Aabb, camera: &Camera) {
    let rect = selection_rect(bounds, camera);
    let mut batch = Batch::new(GlSink);
    draw_rect_outline(
        &mut batch,
        rect,
        SELECTION_WIDTH_PX,
        to_mq_color(THEME.accent),
    );
    batch.finish();
}

/// Adds one shape to `batch` (its labels are drawn right away, after a
/// flush); non-finite shapes are skipped.
fn draw_shape(batch: &mut Batch, shape: &Shape, camera: &Camera) {
    if !shape.is_finite() {
        return;
    }
    let style = shape.style();
    let width = screen_width(style.width, camera.zoom());
    let color = color_of(style.color);
    let to_screen = |p: Vec2| camera.world_to_screen(p);
    match shape {
        Shape::Stroke { points, .. } => draw_polyline(batch, points, camera, width, color),
        Shape::Line { a, b, .. } => {
            draw_segment(batch, to_screen(*a), to_screen(*b), width, color);
        }
        Shape::Arrow { a, b, style } => {
            let [tip, left, right] = arrow_head(*a, *b, style.width).map(to_screen);
            let base = left.lerp(right, 0.5);
            draw_segment(batch, to_screen(*a), base, width, color);
            batch.triangle(tip, left, right, color);
        }
        Shape::Rect {
            a, b, fill, label, ..
        } => {
            let rect = Aabb::from_corners(to_screen(*a), to_screen(*b));
            if let Some(fill) = fill {
                batch.rect(rect, color_of(*fill));
            }
            draw_rect_outline(batch, rect, width, color);
            draw_label(batch, *label, rect, false, label_color(*fill, color));
        }
        Shape::Ellipse {
            a, b, fill, label, ..
        } => {
            let rect = Aabb::from_corners(to_screen(*a), to_screen(*b));
            let center = rect.center();
            let radii = Vec2::new(rect.width() * 0.5, rect.height() * 0.5);
            if let Some(fill) = fill {
                draw_ellipse_fill(batch, center, radii, color_of(*fill));
            }
            draw_ellipse_ring(batch, center, radii, width, color);
            draw_label(batch, *label, rect, true, label_color(*fill, color));
        }
        Shape::Grid {
            a,
            b,
            cols,
            rows,
            axes,
            fills,
            ..
        } => {
            // The camera only scales and offsets, so grid lines, cells and
            // index boxes map to those of the mapped corners.
            let (a, b) = (to_screen(*a), to_screen(*b));
            // Cell fills first, so the lines stay on top (ADR-T21-1).
            for fill in fills {
                let cell = grid_cell_rect(a, b, *cols, *rows, fill.col, fill.row);
                batch.rect(cell, color_of(fill.color));
            }
            for [start, end] in grid_lines(a, b, *cols, *rows) {
                draw_band(batch, start, end, width, color);
            }
            if *axes {
                draw_axis_labels(batch, a, b, *cols, *rows, color);
            }
        }
    }
}

/// Text colour of a label: the outline colour, or the background colour on a
/// filled shape so the number stays readable.
fn label_color(fill: Option<ColorId>, outline: Color) -> Color {
    if fill.is_some() {
        to_mq_color(THEME.bg)
    } else {
        outline
    }
}

/// Draws `label` centred in the screen rectangle `rect` of a rectangle or
/// (`ellipse`) an ellipse, if it is big enough on screen. Flushes `batch`
/// first so the text lands on top of the shape.
fn draw_label(batch: &mut Batch, label: Option<u32>, rect: Aabb, ellipse: bool, color: Color) {
    let Some(label) = label else {
        return;
    };
    let text = label.to_string();
    let Some(size) = label_size(text.len(), label_area(rect, ellipse)) else {
        return;
    };
    let (font_size, font_scale) = label_raster(size);
    batch.flush();
    draw_text_centered(&text, rect, font_size, font_scale, color);
}

/// Draws the axis indices of a grid with screen corners `a` (drag start) and
/// `b`, all at one size, if the cells are big enough on screen. Flushes
/// `batch` first so the text lands on top of the grid.
fn draw_axis_labels(batch: &mut Batch, a: Vec2, b: Vec2, cols: u32, rows: u32, color: Color) {
    let Some(first) = grid_axis_labels(a, b, cols, rows).next() else {
        return;
    };
    let cell = Vec2::new(first.rect.width(), first.rect.height());
    let Some(size) = axis_label_size(cols, rows, cell) else {
        return;
    };
    let (font_size, font_scale) = label_raster(size);
    batch.flush();
    for label in grid_axis_labels(a, b, cols, rows) {
        draw_text_centered(
            &label.index.to_string(),
            label.rect,
            font_size,
            font_scale,
            color,
        );
    }
}

/// Draws `text` centred in the screen rectangle `rect`.
fn draw_text_centered(text: &str, rect: Aabb, font_size: u16, font_scale: f32, color: Color) {
    let dims = measure_text(text, None, font_size, font_scale);
    let c = rect.center();
    draw_text_ex(
        text,
        c.x - dims.width * 0.5,
        c.y + dims.offset_y * 0.5,
        TextParams {
            font_size,
            font_scale,
            color,
            ..TextParams::default()
        },
    );
}

/// Draws an axis-aligned screen segment as a band `width` pixels wide that
/// overhangs both ends by half the width, so crossing bands meet with square
/// corners.
fn draw_band(batch: &mut Batch, start: Vec2, end: Vec2, width: f32, color: Color) {
    batch.rect(Aabb::from_corners(start, end).expand(width * 0.5), color);
}

/// The macroquad colour of a palette entry.
fn color_of(id: ColorId) -> Color {
    to_mq_color(palette(id))
}

/// Draws a polyline of world `points` (ADR-T24-4). A single point is a
/// dot; an empty polyline draws nothing.
fn draw_polyline(batch: &mut Batch, points: &[Vec2], camera: &Camera, width: f32, color: Color) {
    batch.polyline(
        points.iter().map(|p| camera.world_to_screen(*p)),
        width,
        color,
    );
}

/// Draws a screen-space segment with round caps when thick.
fn draw_segment(batch: &mut Batch, a: Vec2, b: Vec2, width: f32, color: Color) {
    batch.line(a, b, width, color);
    if stroke_needs_joints(width) {
        let radius = width * 0.5;
        draw_disc(batch, a, radius, color);
        draw_disc(batch, b, radius, color);
    }
}

/// Draws the outline of a screen rectangle as four opaque bands of `width`
/// pixels centred on its edges (square corners).
fn draw_rect_outline(batch: &mut Batch, rect: Aabb, width: f32, color: Color) {
    let h = width * 0.5;
    let (x0, y0, x1, y1) = (
        rect.min.x - h,
        rect.min.y - h,
        rect.max.x + h,
        rect.max.y + h,
    );
    let band = |ax: f32, ay: f32, bx: f32, by: f32| {
        Aabb::from_corners(Vec2::new(ax, ay), Vec2::new(bx, by))
    };
    batch.rect(band(x0, y0, x1, y0 + width), color);
    batch.rect(band(x0, y1 - width, x1, y1), color);
    batch.rect(band(x0, y0, x0 + width, y1), color);
    batch.rect(band(x1 - width, y0, x1, y1), color);
}

/// Unit vectors around a circle in `n` equal steps, starting at `(1, 0)`,
/// generated by repeated rotation (no trig per vertex). Yields `n + 1`
/// items, the last equal to the first, so consecutive pairs close the loop.
fn unit_circle(n: u16) -> impl ExactSizeIterator<Item = Vec2> {
    let step = std::f32::consts::TAU / f32::from(n);
    let (sin, cos) = step.sin_cos();
    let mut current = Vec2::new(1.0, 0.0);
    (0..=n).map(move |i| {
        let out = if i == n { Vec2::new(1.0, 0.0) } else { current };
        current = Vec2::new(
            current.x * cos - current.y * sin,
            current.x * sin + current.y * cos,
        );
        out
    })
}

/// Scales a unit vector by per-axis radii.
fn scale(u: Vec2, radii: Vec2) -> Vec2 {
    Vec2::new(u.x * radii.x, u.y * radii.y)
}

/// Draws a filled disc of `radius` pixels centred at screen `center`.
fn draw_disc(batch: &mut Batch, center: Vec2, radius: f32, color: Color) {
    batch.disc(center, radius, color);
}

/// Adds a filled axis-aligned ellipse (screen space) as a triangle fan.
fn draw_ellipse_fill(batch: &mut Batch, center: Vec2, radii: Vec2, color: Color) {
    let n = circle_segments(radii.x.max(radii.y));
    let rim = unit_circle(n).map(|u| center + scale(u, radii));
    batch.fan(center, rim, color);
}

/// Adds an ellipse outline `width` pixels wide, centred on the ellipse
/// (screen space), as a ring of quads between the inner and outer ellipses.
fn draw_ellipse_ring(batch: &mut Batch, center: Vec2, radii: Vec2, width: f32, color: Color) {
    let h = width * 0.5;
    let outer = Vec2::new(radii.x + h, radii.y + h);
    let inner = Vec2::new((radii.x - h).max(0.0), (radii.y - h).max(0.0));
    let n = circle_segments(outer.x.max(outer.y));
    let pairs = unit_circle(n).map(|u| (center + scale(u, outer), center + scale(u, inner)));
    batch.strip(pairs, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::Camera;
    use crate::core::geom::{Aabb, Vec2, approx_eq};
    use crate::core::palette::{ColorId, Rgba, palette};
    use proptest::prelude::*;

    const EPS: f32 = 1e-4;

    fn aabb(x0: f32, y0: f32, x1: f32, y1: f32) -> Aabb {
        Aabb::from_corners(Vec2::new(x0, y0), Vec2::new(x1, y1))
    }

    fn aabb_approx_eq(a: Aabb, b: Aabb) -> bool {
        a.min.approx_eq(b.min, EPS) && a.max.approx_eq(b.max, EPS)
    }

    // ---- T24 AC-3a batching ------------------------------------------------

    /// Records every chunk a batch submits.
    #[derive(Default)]
    struct Recorder {
        chunks: Vec<(Vec<Vertex>, Vec<u16>)>,
    }

    impl MeshSink for Recorder {
        fn draw(&mut self, mesh: &Mesh) {
            self.chunks
                .push((mesh.vertices.clone(), mesh.indices.clone()));
        }
    }

    const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);

    fn position(v: &Vertex) -> Vec2 {
        Vec2::new(v.position.x, v.position.y)
    }

    #[test]
    fn batch_triangle_adds_three_vertices() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());

        // Act
        batch.triangle(Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0), RED);
        let rec = batch.finish();

        // Assert
        assert_eq!(rec.chunks.len(), 1);
        let (vertices, indices) = &rec.chunks[0];
        assert_eq!(vertices.len(), 3);
        assert_eq!(indices, &[0, 1, 2]);
    }

    #[test]
    fn batch_quad_shares_four_vertices() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());

        // Act
        batch.rect(aabb(0.0, 0.0, 4.0, 2.0), RED);
        let rec = batch.finish();

        // Assert
        let (vertices, indices) = &rec.chunks[0];
        assert_eq!(vertices.len(), 4);
        assert_eq!(indices.len(), 6);
        let corners: Vec<Vec2> = vertices.iter().map(position).collect();
        for corner in [
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 2.0),
            Vec2::new(0.0, 2.0),
        ] {
            assert!(
                corners.iter().any(|c| c.approx_eq(corner, EPS)),
                "{corner:?}"
            );
        }
    }

    #[test]
    fn batch_fan_shares_the_center() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        let rim: Vec<Vec2> = unit_circle(8).collect();

        // Act: 9 rim points close the loop, so 8 triangles.
        batch.fan(Vec2::ZERO, rim.into_iter(), RED);
        let rec = batch.finish();

        // Assert
        let (vertices, indices) = &rec.chunks[0];
        assert_eq!(vertices.len(), 1 + 9);
        assert_eq!(indices.len(), 3 * 8);
        assert!(indices.chunks(3).all(|t| t[0] == 0));
    }

    #[test]
    fn batch_line_is_a_width_wide_quad() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());

        // Act
        batch.line(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 4.0, RED);
        let rec = batch.finish();

        // Assert
        let (vertices, indices) = &rec.chunks[0];
        assert_eq!((vertices.len(), indices.len()), (4, 6));
        for v in vertices {
            let p = position(v);
            assert!(approx_eq(p.y.abs(), 2.0, EPS), "{p:?}");
            assert!(
                approx_eq(p.x, 0.0, EPS) || approx_eq(p.x, 10.0, EPS),
                "{p:?}"
            );
        }
    }

    #[test]
    fn batch_line_of_zero_length_draws_nothing() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());

        // Act
        batch.line(Vec2::new(3.0, 3.0), Vec2::new(3.0, 3.0), 4.0, RED);
        let rec = batch.finish();

        // Assert
        assert!(rec.chunks.is_empty());
    }

    #[test]
    fn batch_flushes_before_exceeding_limits() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        let quads = BATCH_MAX_INDICES; // far more than one chunk holds

        // Act
        for i in 0..quads {
            let x = i as f32;
            batch.rect(aabb(x, 0.0, x + 1.0, 1.0), RED);
        }
        let rec = batch.finish();

        // Assert
        assert!(rec.chunks.len() > 1);
        let total: usize = rec.chunks.iter().map(|(_, i)| i.len()).sum();
        assert_eq!(total, quads * 6);
        for (vertices, indices) in &rec.chunks {
            assert!(vertices.len() <= BATCH_MAX_VERTICES);
            assert!(indices.len() <= BATCH_MAX_INDICES);
        }
    }

    #[test]
    fn batch_finish_draws_the_rest_once() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        batch.rect(aabb(0.0, 0.0, 1.0, 1.0), RED);
        batch.flush();
        batch.flush();

        // Act
        batch.rect(aabb(2.0, 0.0, 3.0, 1.0), RED);
        let rec = batch.finish();

        // Assert: one chunk per non-empty flush, none for empty ones.
        assert_eq!(rec.chunks.len(), 2);
    }

    #[test]
    fn batch_limits_fit_a_macroquad_draw_call() {
        // macroquad clamps geometry at 10 000 vertices / 5 000 indices.
        const {
            assert!(BATCH_MAX_VERTICES < 10_000);
            assert!(BATCH_MAX_INDICES < 5_000);
            assert!(BATCH_MAX_INDICES % 6 == 0);
        }
    }

    // AC-3d

    /// Vertices and indices of one disc of `radius` pixels.
    fn disc_size(radius: f32) -> (usize, usize) {
        let n = usize::from(circle_segments(radius));
        (n + 2, 3 * n)
    }

    /// The single chunk a batch drew.
    fn only_chunk(rec: &Recorder) -> &(Vec<Vertex>, Vec<u16>) {
        assert_eq!(rec.chunks.len(), 1);
        &rec.chunks[0]
    }

    fn points(xy: &[(f32, f32)]) -> Vec<Vec2> {
        xy.iter().map(|&(x, y)| Vec2::new(x, y)).collect()
    }

    #[test]
    fn batch_polyline_empty_draws_nothing() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());

        // Act
        batch.polyline(Vec::new(), 4.0, RED);
        let rec = batch.finish();

        // Assert
        assert!(rec.chunks.is_empty());
    }

    #[test]
    fn batch_polyline_single_point_is_a_dot() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        let center = Vec2::new(5.0, 5.0);

        // Act
        batch.polyline(vec![center], 4.0, RED);
        let rec = batch.finish();

        // Assert
        let (vertices, indices) = only_chunk(&rec);
        assert_eq!((vertices.len(), indices.len()), disc_size(2.0));
        for v in vertices {
            assert!(position(v).distance(center) <= 2.0 + EPS);
        }
    }

    #[test]
    fn batch_polyline_coincident_points_are_a_dot() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());

        // Act
        batch.polyline(points(&[(5.0, 5.0); 3]), 4.0, RED);
        let rec = batch.finish();

        // Assert
        let (vertices, indices) = only_chunk(&rec);
        assert_eq!((vertices.len(), indices.len()), disc_size(2.0));
    }

    #[test]
    fn batch_polyline_thin_shares_two_vertices_per_point() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        let line = points(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (30.0, 0.0)]);

        // Act
        batch.polyline(line, 1.0, RED);
        let rec = batch.finish();

        // Assert: one strip of 3 quads, no caps.
        let (vertices, indices) = only_chunk(&rec);
        assert_eq!((vertices.len(), indices.len()), (8, 18));
        for v in vertices {
            assert!(approx_eq(position(v).y.abs(), 0.5, EPS), "{v:?}");
        }
    }

    #[test]
    fn batch_polyline_thick_adds_round_caps_only_at_ends() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        let line = points(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (30.0, 0.0)]);

        // Act
        batch.polyline(line, 4.0, RED);
        let rec = batch.finish();

        // Assert: the strip plus one disc per end.
        let (vertices, indices) = only_chunk(&rec);
        let (dv, di) = disc_size(2.0);
        assert_eq!((vertices.len(), indices.len()), (8 + 2 * dv, 18 + 2 * di));
    }

    #[test]
    fn batch_polyline_gentle_turn_is_mitred() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        let corner = Vec2::new(10.0, 0.0);
        let line = vec![Vec2::ZERO, corner, Vec2::new(20.0, 5.0)];
        let half_turn = 0.5_f32.atan() / 2.0;

        // Act: 2 px is not thick, so no caps.
        batch.polyline(line, 2.0, RED);
        let rec = batch.finish();

        // Assert: the corner pair sits on the mitre, on both sides.
        let (vertices, indices) = only_chunk(&rec);
        assert_eq!((vertices.len(), indices.len()), (6, 12));
        let mitre = 1.0 / half_turn.cos();
        let (a, b) = (position(&vertices[2]), position(&vertices[3]));
        assert!(approx_eq(a.distance(corner), mitre, EPS), "{a:?}");
        assert!(approx_eq(b.distance(corner), mitre, EPS), "{b:?}");
        assert!(a.lerp(b, 0.5).approx_eq(corner, EPS));
    }

    #[test]
    fn batch_polyline_sharp_turn_breaks_with_round_join() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        let line = points(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);

        // Act: a right angle needs a mitre of √2 half-widths.
        batch.polyline(line, 4.0, RED);
        let rec = batch.finish();

        // Assert: two one-quad strips, two caps and one join.
        let (vertices, indices) = only_chunk(&rec);
        let (dv, di) = disc_size(2.0);
        assert_eq!((vertices.len(), indices.len()), (8 + 3 * dv, 12 + 3 * di));
    }

    #[test]
    fn batch_polyline_skips_points_closer_than_min_step() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        let step = STROKE_MIN_STEP_PX * 0.4;
        let line = points(&[(0.0, 0.0), (step, 0.0), (2.0 * step, 0.0), (10.0, 0.0)]);

        // Act
        batch.polyline(line, 1.0, RED);
        let rec = batch.finish();

        // Assert: one quad from the first to the last point.
        let (vertices, indices) = only_chunk(&rec);
        assert_eq!((vertices.len(), indices.len()), (4, 6));
    }

    #[test]
    fn batch_strip_longer_than_a_chunk_continues() {
        // Arrange
        let mut batch = Batch::new(Recorder::default());
        let pairs = BATCH_MAX_VERTICES; // twice what one chunk holds

        // Act
        batch.strip(
            (0..pairs).map(|i| {
                let x = i as f32;
                (Vec2::new(x, 0.0), Vec2::new(x, 1.0))
            }),
            RED,
        );
        let rec = batch.finish();

        // Assert: every quad drawn once, each chunk starting where the
        // previous one ended.
        assert!(rec.chunks.len() > 1);
        let total: usize = rec.chunks.iter().map(|(_, i)| i.len()).sum();
        assert_eq!(total, 6 * (pairs - 1));
        for pair in rec.chunks.windows(2) {
            let (prev, next) = (&pair[0].0, &pair[1].0);
            let tail = &prev[prev.len() - 2..];
            assert!(position(&next[0]).approx_eq(position(&tail[0]), EPS));
            assert!(position(&next[1]).approx_eq(position(&tail[1]), EPS));
        }
    }

    proptest! {
        #[test]
        fn batch_indices_stay_inside_their_chunk(
            shapes in prop::collection::vec((0u8..4, 0u16..300), 1..120),
        ) {
            let mut batch = Batch::new(Recorder::default());
            for (kind, n) in shapes {
                let p = Vec2::new(f32::from(n), 1.0);
                match kind {
                    0 => batch.triangle(Vec2::ZERO, p, Vec2::new(0.0, 5.0), RED),
                    1 => batch.rect(aabb(0.0, 0.0, f32::from(n) + 1.0, 2.0), RED),
                    2 => batch.line(Vec2::ZERO, p, 3.0, RED),
                    _ => batch.fan(p, unit_circle(n.max(3))
                        .map(|u| p + u * 10.0), RED),
                }
            }
            let rec = batch.finish();
            for (vertices, indices) in &rec.chunks {
                prop_assert!(!indices.is_empty());
                prop_assert_eq!(indices.len() % 3, 0);
                prop_assert!(indices.iter().all(|&i| usize::from(i) < vertices.len()));
                prop_assert!(vertices.len() <= BATCH_MAX_VERTICES);
                prop_assert!(indices.len() <= BATCH_MAX_INDICES);
            }
        }

        #[test]
        fn batch_polyline_indices_stay_inside_their_chunk(
            xy in prop::collection::vec((-500.0_f32..500.0, -500.0_f32..500.0), 0..3_000),
            width in 0.5_f32..40.0,
        ) {
            let mut batch = Batch::new(Recorder::default());
            batch.polyline(xy.iter().map(|&(x, y)| Vec2::new(x, y)), width, RED);
            let rec = batch.finish();
            for (vertices, indices) in &rec.chunks {
                prop_assert!(!indices.is_empty());
                prop_assert_eq!(indices.len() % 3, 0);
                prop_assert!(indices.iter().all(|&i| usize::from(i) < vertices.len()));
                prop_assert!(vertices.len() <= BATCH_MAX_VERTICES);
                prop_assert!(indices.len() <= BATCH_MAX_INDICES);
                prop_assert!(vertices.iter().all(|v| position(v).is_finite()));
            }
        }
    }

    // AC-1

    #[test]
    fn screen_width_scales_with_zoom() {
        // Arrange
        let (world, zoom) = (3.0, 2.5);
        // Act
        let px = screen_width(world, zoom);
        // Assert
        assert!(approx_eq(px, 7.5, EPS));
    }

    #[test]
    fn screen_width_clamps_to_one_pixel() {
        assert!(approx_eq(screen_width(1.0, 0.05), MIN_SCREEN_WIDTH_PX, EPS));
        assert!(approx_eq(screen_width(0.0, 4.0), MIN_SCREEN_WIDTH_PX, EPS));
    }

    #[test]
    fn screen_width_non_finite_or_negative_is_one_pixel() {
        for (w, z) in [
            (f32::NAN, 1.0),
            (1.0, f32::NAN),
            (f32::INFINITY, 1.0),
            (f32::MAX, 20.0),
            (-5.0, 2.0),
            (f32::NEG_INFINITY, 1.0),
        ] {
            assert!(
                approx_eq(screen_width(w, z), MIN_SCREEN_WIDTH_PX, EPS),
                "width {w}, zoom {z}"
            );
        }
    }

    proptest! {
        #[test]
        fn screen_width_is_at_least_one_pixel_and_finite(w in any::<f32>(), z in any::<f32>()) {
            let px = screen_width(w, z);
            prop_assert!(px.is_finite());
            prop_assert!(px >= MIN_SCREEN_WIDTH_PX);
        }
    }

    // AC-2

    #[test]
    fn is_visible_overlapping_is_true() {
        assert!(is_visible(
            aabb(5.0, 5.0, 15.0, 15.0),
            aabb(0.0, 0.0, 10.0, 10.0)
        ));
    }

    #[test]
    fn is_visible_touching_edge_is_true() {
        assert!(is_visible(
            aabb(10.0, 0.0, 20.0, 5.0),
            aabb(0.0, 0.0, 10.0, 10.0)
        ));
    }

    #[test]
    fn is_visible_disjoint_is_false() {
        let view = aabb(0.0, 0.0, 10.0, 10.0);
        assert!(!is_visible(aabb(11.0, 0.0, 20.0, 5.0), view));
        assert!(!is_visible(aabb(0.0, -9.0, 5.0, -1.0), view));
    }

    #[test]
    fn is_visible_bounds_containing_view_is_true() {
        assert!(is_visible(
            aabb(-100.0, -100.0, 100.0, 100.0),
            aabb(0.0, 0.0, 10.0, 10.0)
        ));
    }

    #[test]
    fn is_visible_nan_bounds_is_false() {
        let view = aabb(0.0, 0.0, 10.0, 10.0);
        let nan = Aabb {
            min: Vec2::new(f32::NAN, 0.0),
            max: Vec2::new(5.0, 5.0),
        };
        assert!(!is_visible(nan, view));
        assert!(!is_visible(view, nan));
    }

    #[test]
    fn cull_rect_is_view_grown_by_margin() {
        // Arrange
        let camera = Camera::new(Vec2::new(10.0, 20.0), 2.0);
        let viewport = Vec2::new(200.0, 100.0);
        // Act
        let rect = cull_rect(&camera, viewport);
        // Assert
        let m = CULL_MARGIN_PX / 2.0;
        assert!(aabb_approx_eq(
            rect,
            aabb(10.0 - m, 20.0 - m, 110.0 + m, 70.0 + m)
        ));
    }

    // AC-3

    #[test]
    fn conversion_vec2_keeps_coordinates() {
        let v = to_mq(Vec2::new(1.5, -2.25));
        assert!(approx_eq(v.x, 1.5, EPS));
        assert!(approx_eq(v.y, -2.25, EPS));
    }

    #[test]
    fn conversion_color_scales_channels() {
        let c = to_mq_color(Rgba {
            r: 255,
            g: 0,
            b: 51,
            a: 102,
        });
        assert!(approx_eq(c.r, 1.0, EPS));
        assert!(approx_eq(c.g, 0.0, EPS));
        assert!(approx_eq(c.b, 0.2, EPS));
        assert!(approx_eq(c.a, 0.4, EPS));
    }

    #[test]
    fn conversion_palette_ink_is_opaque() {
        let c = to_mq_color(palette(ColorId::INK));
        assert!(approx_eq(c.a, 1.0, EPS));
    }

    // AC-4

    #[test]
    fn joints_threshold() {
        assert!(!stroke_needs_joints(1.0));
        assert!(!stroke_needs_joints(JOINT_THRESHOLD_PX));
        assert!(stroke_needs_joints(JOINT_THRESHOLD_PX + 0.01));
        assert!(stroke_needs_joints(12.0));
        assert!(!stroke_needs_joints(f32::NAN));
    }

    // AC-8

    #[test]
    fn circle_segments_small_radius_is_minimum() {
        assert_eq!(circle_segments(0.5), MIN_SEGMENTS);
        assert_eq!(circle_segments(1.0), MIN_SEGMENTS);
    }

    #[test]
    fn circle_segments_grows_with_radius() {
        let small = circle_segments(20.0);
        let large = circle_segments(400.0);
        assert!(small > MIN_SEGMENTS);
        assert!(large > small);
        assert!(large < MAX_SEGMENTS);
    }

    #[test]
    fn circle_segments_huge_radius_is_capped() {
        assert_eq!(circle_segments(1.0e6), MAX_SEGMENTS);
        assert_eq!(circle_segments(f32::MAX), MAX_SEGMENTS);
    }

    #[test]
    fn circle_segments_non_finite_is_minimum() {
        for r in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -3.0] {
            assert_eq!(circle_segments(r), MIN_SEGMENTS, "radius {r}");
        }
    }

    proptest! {
        #[test]
        fn circle_segments_chord_error_within_tolerance(r in 0.01_f32..10_000.0) {
            let n = circle_segments(r);
            prop_assert!((MIN_SEGMENTS..=MAX_SEGMENTS).contains(&n));
            if n < MAX_SEGMENTS {
                let err = r * (1.0 - (std::f32::consts::PI / f32::from(n)).cos());
                prop_assert!(err <= CHORD_TOLERANCE_PX + EPS, "r {r}, n {n}, err {err}");
            }
        }
    }

    // AC-9

    #[test]
    fn selection_rect_pads_screen_bounds() {
        // Arrange
        let camera = Camera::new(Vec2::new(-10.0, 0.0), 1.0);
        let bounds = aabb(0.0, 0.0, 30.0, 20.0);
        // Act
        let rect = selection_rect(bounds, &camera);
        // Assert
        let p = SELECTION_PAD_PX;
        assert!(aabb_approx_eq(rect, aabb(10.0 - p, -p, 40.0 + p, 20.0 + p)));
    }

    #[test]
    fn selection_rect_padding_is_independent_of_zoom() {
        // Arrange
        let camera = Camera::new(Vec2::ZERO, 4.0);
        let bounds = aabb(1.0, 2.0, 3.0, 5.0);
        // Act
        let rect = selection_rect(bounds, &camera);
        // Assert
        let p = SELECTION_PAD_PX;
        assert!(aabb_approx_eq(
            rect,
            aabb(4.0 - p, 8.0 - p, 12.0 + p, 20.0 + p)
        ));
    }

    // T18 AC-7: axis indices

    #[test]
    fn axis_label_size_fits_longest_index() {
        // Arrange: 12 columns → indices up to "11", two characters.
        let cell = Vec2::new(40.0, 40.0);

        // Act
        let wide = axis_label_size(12, 3, cell);
        let narrow = axis_label_size(3, 10, cell);

        // Assert
        assert_eq!(wide, label_size(2, cell));
        assert_eq!(narrow, label_size(1, cell), "index 9 is one digit");
        assert_eq!(axis_label_size(1, 1, Vec2::new(2.0, 2.0)), None);
    }

    // T16 AC-2: labels

    #[test]
    fn label_size_fits_box() {
        // Arrange
        let area = Vec2::new(100.0, 100.0);
        // Act
        let size = label_size(1, area);
        // Assert: limited by the height, and the text fits the width.
        let Some(size) = size else {
            panic!("a 100 px box shows its label");
        };
        assert!(approx_eq(size, 100.0 * LABEL_HEIGHT_RATIO, EPS));
        assert!(size * LABEL_CHAR_ASPECT <= area.x * LABEL_WIDTH_RATIO + EPS);
    }

    #[test]
    fn label_size_shrinks_with_more_digits() {
        let area = Vec2::new(100.0, 100.0);

        let (Some(one), Some(three)) = (label_size(1, area), label_size(3, area)) else {
            panic!("both labels fit");
        };

        assert!(three < one);
        assert!(3.0 * three * LABEL_CHAR_ASPECT <= area.x * LABEL_WIDTH_RATIO + EPS);
    }

    #[test]
    fn label_size_hidden_when_too_small() {
        assert_eq!(label_size(1, Vec2::new(10.0, 10.0)), None);
        assert_eq!(label_size(5, Vec2::new(25.0, 400.0)), None);
        assert_eq!(label_size(1, Vec2::new(-50.0, 50.0)), None);
    }

    #[test]
    fn label_size_is_capped() {
        let size = label_size(1, Vec2::new(1.0e5, 1.0e5));

        assert!(
            size.is_some_and(|s| approx_eq(s, LABEL_MAX_PX, EPS)),
            "{size:?}"
        );
    }

    #[test]
    fn label_size_non_finite_is_none() {
        for area in [
            Vec2::new(f32::NAN, 100.0),
            Vec2::new(100.0, f32::INFINITY),
            Vec2::new(f32::NEG_INFINITY, f32::NAN),
        ] {
            assert_eq!(label_size(1, area), None, "{area:?}");
        }
    }

    #[test]
    fn label_area_ellipse_is_inscribed_box() {
        let rect = aabb(0.0, 0.0, 100.0, 50.0);

        let boxed = label_area(rect, false);
        let round = label_area(rect, true);

        assert!(boxed.approx_eq(Vec2::new(100.0, 50.0), EPS));
        let k = std::f32::consts::FRAC_1_SQRT_2;
        assert!(round.approx_eq(Vec2::new(100.0 * k, 50.0 * k), EPS));
    }

    #[test]
    fn label_raster_quantizes() {
        let cases = [
            (10.0, 16, 10.0 / 16.0),
            (16.0, 16, 1.0),
            (17.0, 32, 17.0 / 32.0),
            (100.0, 128, 100.0 / 128.0),
            (200.0, 128, 200.0 / 128.0),
        ];
        for (size, raster, scale) in cases {
            let (got_raster, got_scale) = label_raster(size);
            assert_eq!(got_raster, raster, "size {size}");
            assert!(approx_eq(got_scale, scale, EPS), "size {size}: {got_scale}");
        }
    }

    #[test]
    fn dot_grid_spacing_at_unit_zoom_is_grid_step() {
        let spacing = dot_grid_spacing(1.0);

        assert!(spacing.is_some_and(|s| approx_eq(s, GRID_STEP, EPS)));
    }

    #[test]
    fn dot_grid_spacing_coarsens_when_zoomed_out() {
        // Arrange: at zoom 0.05 a grid step is 1 px on screen.
        let zoom = 0.05;

        // Act
        let spacing = dot_grid_spacing(zoom);

        // Assert: a power-of-two multiple of the step, the first at least
        // DOT_GRID_MIN_PX apart on screen.
        let Some(spacing) = spacing else {
            panic!("finite zoom has a spacing");
        };
        let ratio = spacing / GRID_STEP;
        assert!(
            approx_eq(ratio.log2().round().exp2(), ratio, EPS),
            "{ratio}"
        );
        assert!(spacing * zoom >= DOT_GRID_MIN_PX);
        assert!(spacing * zoom / 2.0 < DOT_GRID_MIN_PX);
    }

    #[test]
    fn dot_grid_spacing_non_finite_is_none() {
        assert!(dot_grid_spacing(f32::NAN).is_none());
        assert!(dot_grid_spacing(f32::INFINITY).is_none());
        assert!(dot_grid_spacing(0.0).is_none());
        assert!(dot_grid_spacing(-1.0).is_none());
        assert!(dot_grid_spacing(1e-30).is_none());
    }

    #[test]
    fn dot_grid_points_cover_view_on_multiples() {
        // Arrange
        let view = aabb(-25.0, -5.0, 45.0, 30.0);

        // Act
        let points: Vec<Vec2> = dot_grid_points(view, 20.0).collect();

        // Assert: x in {-20, 0, 20, 40}, y in {0, 20}.
        assert_eq!(points.len(), 8, "{points:?}");
        for p in &points {
            assert!(view.contains(*p), "{p:?}");
            assert!(approx_eq(p.x / 20.0, (p.x / 20.0).round(), EPS));
            assert!(approx_eq(p.y / 20.0, (p.y / 20.0).round(), EPS));
        }
    }

    #[test]
    fn dot_grid_points_are_capped_and_total() {
        let huge = aabb(-1.0e30, -1.0e30, 1.0e30, 1.0e30);
        let nan = Aabb {
            min: Vec2::new(f32::NAN, 0.0),
            max: Vec2::new(10.0, 10.0),
        };

        assert!(dot_grid_points(huge, 20.0).count() <= DOT_GRID_MAX_DOTS);
        assert_eq!(dot_grid_points(nan, 20.0).count(), 0);
        assert_eq!(dot_grid_points(aabb(0.0, 0.0, 10.0, 10.0), 0.0).count(), 0);
    }
}
