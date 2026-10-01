//! Snapping of dragged points (T17, ADR-T17-1).
//!
//! Smart snap (round shapes, matching sizes, alignment) and grid snap are
//! switched by [`Helpers`] flags, which tools read from
//! [`ToolView::style`] (ADR-T16-3). Alignment guides go to
//! [`Overlay::guides`](crate::core::tools::Overlay::guides).
//!
//! The moving end of a drag goes grid → size → align → round
//! ([`snap_drag`]); the start point goes grid → align ([`snap_point`]).
//! Tolerances are screen pixels converted with the camera. Every function
//! is pure and total: a step whose result would not be finite is skipped.

use crate::core::camera::Camera;
use crate::core::document::Document;
use crate::core::editor::Helpers;
use crate::core::geom::Vec2;
use crate::core::input::Modifiers;
use crate::core::shape::{GRID_MAX_CELLS, Shape};
use crate::core::tools::ToolView;

/// A box drag snaps to round when `min(|w|,|h|) / max(|w|,|h|)` is at least
/// `1 - ROUND_TOLERANCE`.
pub const ROUND_TOLERANCE: f32 = 0.1;

/// How close, in screen pixels, a dragged size must be to a target size.
pub const SIZE_TOLERANCE_PX: f32 = 6.0;

/// How close, in screen pixels, a dragged line must be to a target line.
pub const ALIGN_TOLERANCE_PX: f32 = 6.0;

/// Largest integer multiple of a target size a drag snaps to.
pub const MAX_SIZE_MULTIPLE: u8 = 8;

/// Distance between grid lines in world units.
pub const GRID_STEP: f32 = 20.0;

/// What a drag spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragKind {
    /// Two free points: lines and arrows. Only grid and align apply.
    Point,
    /// A box with corners at the two points: rectangles and ellipses.
    Box,
}

/// Which snaps are active for one event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Snaps {
    /// Snap to the world grid.
    pub grid: bool,
    /// Snap sizes, alignment and round shapes.
    pub smart: bool,
}

impl Snaps {
    /// No snapping.
    pub const NONE: Self = Self {
        grid: false,
        smart: false,
    };

    /// The snaps `helpers` switch on: none while `Alt` is held, only grid
    /// snap while `Shift` is held (its constraint must hold exactly).
    #[must_use]
    pub fn new(helpers: Helpers, mods: Modifiers) -> Self {
        Self {
            grid: helpers.grid_snap && !mods.alt,
            smart: helpers.smart_snap && !mods.alt && !mods.shift,
        }
    }
}

/// A line a dragged coordinate can align with: `value` on one axis, from
/// `lo` to `hi` on the other (used to draw the guide).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    /// Coordinate of the line.
    pub value: f32,
    /// Smallest extent of the target along the line.
    pub lo: f32,
    /// Largest extent of the target along the line.
    pub hi: f32,
}

/// What a drag can snap to, collected from shapes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Targets {
    /// Vertical lines (`x = value`).
    pub xs: Vec<Anchor>,
    /// Horizontal lines (`y = value`).
    pub ys: Vec<Anchor>,
    /// Box widths and heights, grid cell sizes; all finite and positive.
    pub sizes: Vec<f32>,
}

impl Targets {
    /// Targets of `shapes`: edges and centres of rectangle, ellipse and grid
    /// boxes, line and arrow endpoints, box and grid cell sizes. Strokes and
    /// non-finite values are skipped.
    #[must_use]
    pub fn from_shapes<'a>(shapes: impl IntoIterator<Item = &'a Shape>) -> Self {
        let mut targets = Self::default();
        for shape in shapes {
            match *shape {
                Shape::Rect { a, b, .. } | Shape::Ellipse { a, b, .. } => {
                    targets.add_box(a, b);
                }
                Shape::Grid {
                    a, b, cols, rows, ..
                } => {
                    targets.add_box(a, b);
                    let cols = cols.clamp(1, GRID_MAX_CELLS) as f32;
                    let rows = rows.clamp(1, GRID_MAX_CELLS) as f32;
                    targets.add_size((b.x - a.x).abs() / cols);
                    targets.add_size((b.y - a.y).abs() / rows);
                }
                Shape::Line { a, b, .. } | Shape::Arrow { a, b, .. } => {
                    for p in [a, b] {
                        add_anchor(&mut targets.xs, p.x, p.y, p.y);
                        add_anchor(&mut targets.ys, p.y, p.x, p.x);
                    }
                }
                Shape::Stroke { .. } => {}
            }
        }
        targets
    }

    /// Targets of every shape in `doc`.
    #[must_use]
    pub fn from_document(doc: &Document) -> Self {
        Self::from_shapes(doc.shapes().map(|(_, shape)| shape))
    }

    /// Adds the edges, centre and sides of the box spanned by `a` and `b`.
    fn add_box(&mut self, a: Vec2, b: Vec2) {
        let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
        let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
        for x in [x0, x1, midpoint(x0, x1)] {
            add_anchor(&mut self.xs, x, y0, y1);
        }
        for y in [y0, y1, midpoint(y0, y1)] {
            add_anchor(&mut self.ys, y, x0, x1);
        }
        self.add_size(x1 - x0);
        self.add_size(y1 - y0);
    }

    /// Adds `size` if it is finite and positive.
    fn add_size(&mut self, size: f32) {
        if size.is_finite() && size > 0.0 {
            self.sizes.push(size);
        }
    }
}

/// Pushes an anchor if all its numbers are finite.
fn add_anchor(anchors: &mut Vec<Anchor>, value: f32, lo: f32, hi: f32) {
    if value.is_finite() && lo.is_finite() && hi.is_finite() {
        anchors.push(Anchor { value, lo, hi });
    }
}

/// Midpoint of `a` and `b` that does not overflow for finite inputs.
fn midpoint(a: f32, b: f32) -> f32 {
    a * 0.5 + b * 0.5
}

/// `next` if it is finite, else `current`: a step that would leave the
/// finite range is skipped.
fn finite_or(next: Vec2, current: Vec2) -> Vec2 {
    if next.is_finite() { next } else { current }
}

/// A snapped point and the guides that explain it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapped {
    /// The point after snapping, in world coordinates.
    pub point: Vec2,
    /// Alignment guides as world-space segments.
    pub guides: Vec<[Vec2; 2]>,
}

/// `p` moved to the nearest multiple of `step` on each axis. Unchanged if
/// `step` is not finite and positive or the result would not be finite.
#[must_use]
pub fn snap_to_grid(p: Vec2, step: f32) -> Vec2 {
    if !(step.is_finite() && step > 0.0) {
        return p;
    }
    let snap = |c: f32| (c / step).round() * step;
    finite_or(Vec2::new(snap(p.x), snap(p.y)), p)
}

/// `end` moved so the box from `start` is square when it is within
/// [`ROUND_TOLERANCE`] of square: both sides take the larger length, the
/// drag direction is kept.
#[must_use]
pub fn snap_round(start: Vec2, end: Vec2) -> Vec2 {
    let d = end - start;
    let (w, h) = (d.x.abs(), d.y.abs());
    let side = w.max(h);
    if side > 0.0 && w.min(h) >= (1.0 - ROUND_TOLERANCE) * side {
        let next = start + Vec2::new(side.copysign(d.x), side.copysign(d.y));
        finite_or(next, end)
    } else {
        end
    }
}

/// `end` moved so each side of the box from `start` is `k · c` for a size
/// `c` in `sizes` and `k` in `1..=MAX_SIZE_MULTIPLE`, when within
/// `tolerance` (world units). The nearest wins, the smaller `k` on ties.
#[must_use]
pub fn snap_size(start: Vec2, end: Vec2, sizes: &[f32], tolerance: f32) -> Vec2 {
    let d = end - start;
    let next = start
        + Vec2::new(
            snap_len(d.x, sizes, tolerance),
            snap_len(d.y, sizes, tolerance),
        );
    finite_or(next, end)
}

/// The signed length `d` with its magnitude snapped to the nearest
/// multiple of a size within `tolerance`. Multiples not longer than the
/// tolerance are ignored, so a degenerate side never grows.
fn snap_len(d: f32, sizes: &[f32], tolerance: f32) -> f32 {
    let len = d.abs();
    let mut best: Option<(f32, f32)> = None;
    for &size in sizes {
        for k in 1..=MAX_SIZE_MULTIPLE {
            let target = size * f32::from(k);
            let dist = (len - target).abs();
            let closer = best.is_none_or(|(best_dist, _)| dist < best_dist);
            if target > tolerance && dist <= tolerance && closer {
                best = Some((dist, target));
            }
        }
    }
    best.map_or(d, |(_, target)| target.copysign(d))
}

/// The value of the anchor nearest `c` within `tolerance`, as
/// `(distance, value)`.
fn nearest(anchors: &[Anchor], c: f32, tolerance: f32) -> Option<(f32, f32)> {
    anchors
        .iter()
        .map(|anchor| ((c - anchor.value).abs(), anchor.value))
        .filter(|&(dist, _)| dist <= tolerance)
        .min_by(|a, b| a.0.total_cmp(&b.0))
}

/// `p` moved per axis onto the nearest target line within `tolerance`.
#[must_use]
pub fn align_point(p: Vec2, targets: &Targets, tolerance: f32) -> Vec2 {
    let x = nearest(&targets.xs, p.x, tolerance).map_or(p.x, |(_, v)| v);
    let y = nearest(&targets.ys, p.y, tolerance).map_or(p.y, |(_, v)| v);
    Vec2::new(x, y)
}

/// `end` moved per axis onto the nearest target line within `tolerance`;
/// for a [`DragKind::Box`] the box centre may land on the line instead.
#[must_use]
pub fn align_end(
    start: Vec2,
    end: Vec2,
    kind: DragKind,
    targets: &Targets,
    tolerance: f32,
) -> Vec2 {
    let axis = |anchors: &[Anchor], s: f32, e: f32| {
        let edge = nearest(anchors, e, tolerance);
        let centre = match kind {
            DragKind::Point => None,
            DragKind::Box => nearest(anchors, midpoint(s, e), tolerance)
                .map(|(dist, v)| (dist, 2.0 * v - s))
                .filter(|(_, e)| e.is_finite()),
        };
        match (edge, centre) {
            (Some(edge), Some(centre)) if centre.0 < edge.0 => centre.1,
            (Some((_, v)), _) | (None, Some((_, v))) => v,
            (None, None) => e,
        }
    };
    let next = Vec2::new(
        axis(&targets.xs, start.x, end.x),
        axis(&targets.ys, start.y, end.y),
    );
    finite_or(next, end)
}

/// One guide for each x/y line of the drag (box edges and centre, or the
/// two points) lying on a target line, spanning the target and the drag.
#[must_use]
pub fn guides(start: Vec2, end: Vec2, kind: DragKind, targets: &Targets) -> Vec<[Vec2; 2]> {
    let lines = |s: f32, e: f32| -> Vec<f32> {
        let mut lines = vec![s, e];
        if kind == DragKind::Box {
            lines.push(midpoint(s, e));
        }
        lines.retain(|c| c.is_finite());
        lines.sort_by(f32::total_cmp);
        lines.dedup_by(|a, b| on_line(*a, *b));
        lines
    };
    let span = |s: f32, e: f32| (s.min(e), s.max(e));
    let mut out = Vec::new();
    for x in lines(start.x, end.x) {
        if let Some((lo, hi)) = guide_span(&targets.xs, x, span(start.y, end.y)) {
            out.push([Vec2::new(x, lo), Vec2::new(x, hi)]);
        }
    }
    for y in lines(start.y, end.y) {
        if let Some((lo, hi)) = guide_span(&targets.ys, y, span(start.x, end.x)) {
            out.push([Vec2::new(lo, y), Vec2::new(hi, y)]);
        }
    }
    out
}

/// Whether two coordinates are the same line, up to rounding.
fn on_line(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-4 * a.abs().max(b.abs()).max(1.0)
}

/// The extent of a guide at `c`: the union of `drag` and every anchor
/// lying on `c`, or `None` if no anchor does.
fn guide_span(anchors: &[Anchor], c: f32, drag: (f32, f32)) -> Option<(f32, f32)> {
    anchors
        .iter()
        .filter(|anchor| on_line(anchor.value, c))
        .fold(None, |span: Option<(f32, f32)>, anchor| {
            let (lo, hi) = span.unwrap_or(drag);
            Some((lo.min(anchor.lo), hi.max(anchor.hi)))
        })
}

/// Snaps a drag's start point: grid, then alignment. No guides.
#[must_use]
pub fn snap_point(p: Vec2, snaps: Snaps, targets: &Targets, camera: &Camera) -> Vec2 {
    let mut p = p;
    if snaps.grid {
        p = snap_to_grid(p, GRID_STEP);
    }
    if snaps.smart {
        let tolerance = camera.world_len(ALIGN_TOLERANCE_PX);
        p = finite_or(align_point(p, targets, tolerance), p);
    }
    p
}

/// Snaps the moving `end` of a drag from `start`: grid, size, align, round
/// (size and round only for boxes), with guides when smart snap is on.
#[must_use]
pub fn snap_drag(
    start: Vec2,
    end: Vec2,
    kind: DragKind,
    snaps: Snaps,
    targets: &Targets,
    camera: &Camera,
) -> Snapped {
    let mut point = end;
    if snaps.grid {
        point = snap_to_grid(point, GRID_STEP);
    }
    if !snaps.smart {
        return Snapped {
            point,
            guides: Vec::new(),
        };
    }
    if kind == DragKind::Box {
        point = snap_size(
            start,
            point,
            &targets.sizes,
            camera.world_len(SIZE_TOLERANCE_PX),
        );
    }
    point = align_end(
        start,
        point,
        kind,
        targets,
        camera.world_len(ALIGN_TOLERANCE_PX),
    );
    if kind == DragKind::Box {
        point = snap_round(start, point);
    }
    Snapped {
        point,
        guides: guides(start, point, kind, targets),
    }
}

/// [`snap_point`] with the helpers, document and camera of `view`.
#[must_use]
pub fn snap_start(p: Vec2, mods: Modifiers, view: &ToolView<'_>) -> Vec2 {
    let snaps = Snaps::new(view.style.helpers, mods);
    if snaps == Snaps::NONE {
        return p;
    }
    snap_point(p, snaps, &Targets::from_document(view.doc), view.camera)
}

/// [`snap_drag`] with the helpers, document and camera of `view`.
#[must_use]
pub fn snap_end(
    start: Vec2,
    end: Vec2,
    kind: DragKind,
    mods: Modifiers,
    view: &ToolView<'_>,
) -> Snapped {
    let snaps = Snaps::new(view.style.helpers, mods);
    let targets = if snaps.smart {
        Targets::from_document(view.doc)
    } else {
        Targets::default()
    };
    snap_drag(start, end, kind, snaps, &targets, view.camera)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::{Camera, ZOOM_MAX, ZOOM_MIN};
    use crate::core::command::Tool;
    use crate::core::document::{Document, tx_insert};
    use crate::core::editor::DrawStyle;
    use crate::core::geom::approx_eq;
    use crate::core::history::History;
    use crate::core::input::Modifiers;
    use crate::core::shape::Shape;
    use crate::core::smoothing::SmoothingLevel;
    use proptest::prelude::*;

    const EPS: f32 = 1e-4;

    const STYLE: crate::core::shape::Style = crate::core::shape::Style {
        color: crate::core::palette::ColorId::INK,
        width: 1.0,
    };

    fn v(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    fn rect(a: Vec2, b: Vec2) -> Shape {
        Shape::Rect {
            a,
            b,
            style: STYLE,
            fill: None,
            label: None,
        }
    }

    fn line(a: Vec2, b: Vec2) -> Shape {
        Shape::Line { a, b, style: STYLE }
    }

    fn targets(shapes: &[Shape]) -> Targets {
        Targets::from_shapes(shapes)
    }

    const SMART: Snaps = Snaps {
        grid: false,
        smart: true,
    };

    const GRID: Snaps = Snaps {
        grid: true,
        smart: false,
    };

    const BOTH: Snaps = Snaps {
        grid: true,
        smart: true,
    };

    // AC-1

    #[test]
    fn near_circle_becomes_circle() {
        // Arrange: 95/100 is within 10 % of round.
        let (start, end) = (Vec2::ZERO, v(100.0, 95.0));

        // Act
        let round = snap_round(start, end);
        let snapped = snap_drag(
            start,
            v(100.0, 93.0),
            DragKind::Box,
            SMART,
            &Targets::default(),
            &Camera::default(),
        );

        // Assert
        assert!(round.approx_eq(v(100.0, 100.0), EPS), "{round:?}");
        assert!(snapped.point.approx_eq(v(100.0, 100.0), EPS));
    }

    #[test]
    fn far_from_round_is_unchanged() {
        let end = v(100.0, 80.0);

        assert!(snap_round(Vec2::ZERO, end).approx_eq(end, EPS));
        assert!(snap_round(Vec2::ZERO, v(100.0, 0.0)).approx_eq(v(100.0, 0.0), EPS));
    }

    #[test]
    fn round_keeps_drag_direction() {
        let start = v(10.0, 10.0);

        let round = snap_round(start, v(-90.0, 105.0));

        assert!(round.approx_eq(v(-90.0, 110.0), EPS), "{round:?}");
    }

    // AC-2

    #[test]
    fn size_snaps_to_existing_shape() {
        // Arrange: a 50 × 30 box far away, so nothing aligns.
        let t = targets(&[rect(v(500.0, 500.0), v(550.0, 530.0))]);

        // Act
        let end = snap_size(Vec2::ZERO, v(53.0, 28.0), &t.sizes, 6.0);

        // Assert
        assert!(end.approx_eq(v(50.0, 30.0), EPS), "{end:?}");
    }

    #[test]
    fn size_snaps_to_multiple() {
        let t = targets(&[rect(v(500.0, 500.0), v(540.0, 525.0))]);

        let end = snap_size(Vec2::ZERO, v(-118.0, 300.0), &t.sizes, 6.0);

        assert!(end.approx_eq(v(-120.0, 300.0), EPS), "{end:?}");
    }

    #[test]
    fn size_snaps_to_grid_cell() {
        // Arrange: 90 × 60 grid of 3 × 2 cells of 30 × 30.
        let grid = Shape::Grid {
            a: v(1000.0, 1000.0),
            b: v(1090.0, 1060.0),
            cols: 3,
            rows: 2,
            style: STYLE,
        };
        let t = targets(&[grid]);

        // Act
        let end = snap_size(Vec2::ZERO, v(31.0, 200.0), &t.sizes, 6.0);

        // Assert
        assert!(approx_eq(end.x, 30.0, EPS), "{end:?}");
        assert!(approx_eq(end.y, 200.0, EPS), "{end:?}");
    }

    #[test]
    fn size_out_of_tolerance_is_unchanged() {
        let t = targets(&[rect(v(500.0, 500.0), v(550.0, 530.0))]);
        let end = v(70.0, 75.0);

        assert!(snap_size(Vec2::ZERO, end, &t.sizes, 6.0).approx_eq(end, EPS));
    }

    // AC-3

    #[test]
    fn edge_aligns_to_neighbour() {
        // Arrange
        let t = targets(&[rect(v(100.0, 0.0), v(200.0, 50.0))]);
        let start = v(0.0, 100.0);

        // Act
        let end = align_end(start, v(97.0, 163.0), DragKind::Box, &t, 6.0);
        let lines = guides(start, end, DragKind::Box, &t);

        // Assert: the right edge lands on x = 100; one vertical guide spans
        // the neighbour and the dragged box.
        assert!(end.approx_eq(v(100.0, 163.0), EPS), "{end:?}");
        assert_eq!(lines.len(), 1, "{lines:?}");
        let [a, b] = lines[0];
        assert!(approx_eq(a.x, 100.0, EPS) && approx_eq(b.x, 100.0, EPS));
        assert!(approx_eq(a.y.min(b.y), 0.0, EPS));
        assert!(approx_eq(a.y.max(b.y), 163.0, EPS));
    }

    #[test]
    fn centre_aligns() {
        // Arrange: the neighbour's centre is x = 150.
        let t = targets(&[rect(v(100.0, 0.0), v(200.0, 50.0))]);
        let start = v(120.0, 100.0);

        // Act: the dragged box's centre is at 149.
        let end = align_end(start, v(178.0, 140.0), DragKind::Box, &t, 6.0);

        // Assert
        assert!(end.approx_eq(v(180.0, 140.0), EPS), "{end:?}");
        let lines = guides(start, end, DragKind::Box, &t);
        assert!(
            lines
                .iter()
                .any(|[a, b]| approx_eq(a.x, 150.0, EPS) && approx_eq(b.x, 150.0, EPS)),
            "{lines:?}"
        );
    }

    #[test]
    fn line_end_aligns_to_endpoint() {
        let t = targets(&[line(Vec2::ZERO, v(100.0, 40.0))]);

        let end = align_end(v(300.0, 300.0), v(304.0, 43.0), DragKind::Point, &t, 6.0);
        // A point drag has no centre: x 196 stays although (0 + 196) / 2 is
        // near the endpoint x = 100.
        let free = align_end(v(0.0, 300.0), v(196.0, 500.0), DragKind::Point, &t, 6.0);

        assert!(end.approx_eq(v(304.0, 40.0), EPS), "{end:?}");
        assert!(free.approx_eq(v(196.0, 500.0), EPS), "{free:?}");
    }

    #[test]
    fn align_out_of_tolerance_is_unchanged() {
        let t = targets(&[rect(v(100.0, 0.0), v(200.0, 50.0))]);
        let end = v(90.0, 70.0);

        let snapped = align_end(v(-300.0, 300.0), end, DragKind::Box, &t, 6.0);
        let start = align_point(v(-300.0, 58.0), &t, 6.0);

        assert!(snapped.approx_eq(end, EPS));
        assert!(start.approx_eq(v(-300.0, 58.0), EPS));
    }

    #[test]
    fn start_point_aligns() {
        let t = targets(&[rect(v(100.0, 0.0), v(200.0, 50.0))]);

        let start = align_point(v(-300.0, 52.0), &t, 6.0);

        assert!(start.approx_eq(v(-300.0, 50.0), EPS), "{start:?}");
    }

    #[test]
    fn guides_listed_in_overlay() {
        // Arrange: a document with a neighbour, smart snap on.
        let mut doc = Document::new();
        let mut history = History::new();
        let tx = tx_insert(&mut doc, [rect(v(100.0, 0.0), v(200.0, 50.0))]);
        assert!(history.commit(&mut doc, tx).is_ok());
        let camera = Camera::default();
        let mut style = DrawStyle::default();
        style.helpers.smart_snap = true;
        let view = ToolView {
            doc: &doc,
            camera: &camera,
            selection: &[],
            tool: Tool::Rect,
            style,
            smoothing: SmoothingLevel::Medium,
            cursor: Vec2::ZERO,
        };

        // Act
        let start = snap_start(v(-300.0, 2.0), Modifiers::NONE, &view);
        let snapped = snap_end(
            start,
            v(-250.0, 300.0),
            DragKind::Box,
            Modifiers::NONE,
            &view,
        );
        let alt = Modifiers {
            alt: true,
            ..Modifiers::NONE
        };
        let raw = snap_end(start, v(-250.0, 300.0), DragKind::Box, alt, &view);

        // Assert: the top edge sits on y = 0 and a horizontal guide says so.
        assert!(start.approx_eq(v(-300.0, 0.0), EPS), "{start:?}");
        assert!(
            snapped
                .guides
                .iter()
                .any(|[a, b]| approx_eq(a.y, 0.0, EPS) && approx_eq(b.y, 0.0, EPS)),
            "{snapped:?}"
        );
        assert!(raw.guides.is_empty());
        assert!(raw.point.approx_eq(v(-250.0, 300.0), EPS));
    }

    #[test]
    fn snaps_follow_helpers_and_alt() {
        let mut style = DrawStyle::default();
        style.helpers.grid_snap = true;
        let alt = Modifiers {
            alt: true,
            ..Modifiers::NONE
        };

        assert_eq!(Snaps::new(style.helpers, Modifiers::NONE), GRID);
        assert_eq!(Snaps::new(style.helpers, alt), Snaps::NONE);
        assert_eq!(
            Snaps::new(DrawStyle::default().helpers, Modifiers::NONE),
            Snaps::NONE
        );
    }

    // AC-4

    #[test]
    fn point_snaps_to_grid() {
        let p = snap_to_grid(v(29.0, -11.0), GRID_STEP);
        let end = snap_drag(
            Vec2::ZERO,
            v(52.0, 68.0),
            DragKind::Point,
            GRID,
            &Targets::default(),
            &Camera::default(),
        );
        let start = snap_point(v(11.0, 9.0), GRID, &Targets::default(), &Camera::default());

        assert!(approx_eq(GRID_STEP, 20.0, EPS));
        assert!(p.approx_eq(v(20.0, -20.0), EPS), "{p:?}");
        assert!(end.point.approx_eq(v(60.0, 60.0), EPS));
        assert!(end.guides.is_empty());
        assert!(start.approx_eq(v(20.0, 0.0), EPS), "{start:?}");
    }

    #[test]
    fn grid_snap_with_bad_step_is_unchanged() {
        let p = v(29.0, -11.0);

        assert!(snap_to_grid(p, 0.0).approx_eq(p, EPS));
        assert!(snap_to_grid(p, f32::NAN).approx_eq(p, EPS));
        assert!(snap_to_grid(p, -20.0).approx_eq(p, EPS));
    }

    #[test]
    fn grid_runs_before_smart_snaps() {
        // Arrange: a 45-wide box; grid snap alone would give x = 40.
        let t = targets(&[rect(v(500.0, 500.0), v(545.0, 530.0))]);

        // Act
        let snapped = snap_drag(
            Vec2::ZERO,
            v(47.0, 200.0),
            DragKind::Box,
            BOTH,
            &t,
            &Camera::default(),
        );

        // Assert: grid took 47 to 40, then size matched 45.
        assert!(snapped.point.approx_eq(v(45.0, 200.0), EPS), "{snapped:?}");
    }

    #[test]
    fn tolerance_is_in_screen_pixels() {
        // At zoom 4, 6 px is 1.5 world units: a 3-unit gap does not align.
        let t = targets(&[rect(v(100.0, 0.0), v(200.0, 50.0))]);
        let zoomed = Camera::new(Vec2::ZERO, 4.0);
        let end = v(97.0, 163.0);

        let snapped = snap_drag(v(0.0, 300.0), end, DragKind::Point, SMART, &t, &zoomed);

        assert!(snapped.point.approx_eq(end, EPS));
    }

    // AC-6

    fn any_coord() -> impl Strategy<Value = f32> {
        prop_oneof![
            -1.0e4_f32..1.0e4,
            prop::num::f32::NORMAL,
            Just(f32::MAX),
            Just(f32::MIN),
            Just(0.0),
        ]
    }

    fn any_point() -> impl Strategy<Value = Vec2> {
        (any_coord(), any_coord()).prop_map(|(x, y)| Vec2::new(x, y))
    }

    fn any_shape() -> impl Strategy<Value = Shape> {
        (any_point(), any_point(), 0_u8..3, 1_u32..70, 1_u32..70).prop_map(
            |(a, b, kind, cols, rows)| match kind {
                0 => rect(a, b),
                1 => line(a, b),
                _ => Shape::Grid {
                    a,
                    b,
                    cols,
                    rows,
                    style: STYLE,
                },
            },
        )
    }

    fn any_snaps() -> impl Strategy<Value = Snaps> {
        (any::<bool>(), any::<bool>()).prop_map(|(grid, smart)| Snaps { grid, smart })
    }

    proptest! {
        #[test]
        fn snap_is_finite_for_any_input(
            start in any_point(),
            end in any_point(),
            shapes in proptest::collection::vec(any_shape(), 0..6),
            snaps in any_snaps(),
            boxed in any::<bool>(),
            zoom in ZOOM_MIN..ZOOM_MAX,
        ) {
            let t = Targets::from_shapes(&shapes);
            let camera = Camera::new(Vec2::ZERO, zoom);
            let kind = if boxed { DragKind::Box } else { DragKind::Point };

            let s = snap_point(start, snaps, &t, &camera);
            let snapped = snap_drag(s, end, kind, snaps, &t, &camera);

            prop_assert!(s.is_finite(), "{s:?}");
            prop_assert!(snapped.point.is_finite(), "{snapped:?}");
            prop_assert!(snapped.guides.iter().all(|[a, b]| a.is_finite() && b.is_finite()));
            if !snaps.grid && !snaps.smart {
                prop_assert_eq!(s, start);
                prop_assert_eq!(snapped.point, end);
            }
        }

        #[test]
        fn grid_snap_is_finite_for_any_input(p in any_point(), step in any::<f32>()) {
            let snapped = snap_to_grid(p, step);

            prop_assert!(snapped.is_finite());
        }

        #[test]
        fn grid_snap_moves_at_most_half_a_step(x in -1.0e4_f32..1.0e4, y in -1.0e4_f32..1.0e4) {
            let p = Vec2::new(x, y);

            let snapped = snap_to_grid(p, GRID_STEP);

            prop_assert!((snapped.x - x).abs() <= GRID_STEP / 2.0 + EPS);
            prop_assert!((snapped.y - y).abs() <= GRID_STEP / 2.0 + EPS);
        }
    }
}
