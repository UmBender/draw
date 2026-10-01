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
use crate::core::shape::Shape;
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

    /// The snaps `helpers` switch on, or none while `Alt` is held.
    #[must_use]
    pub fn new(helpers: Helpers, mods: Modifiers) -> Self {
        let _ = (helpers, mods);
        todo!()
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
        let _ = shapes.into_iter();
        todo!()
    }

    /// Targets of every shape in `doc`.
    #[must_use]
    pub fn from_document(doc: &Document) -> Self {
        Self::from_shapes(doc.shapes().map(|(_, shape)| shape))
    }
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
    let _ = (p, step);
    todo!()
}

/// `end` moved so the box from `start` is square when it is within
/// [`ROUND_TOLERANCE`] of square: both sides take the larger length, the
/// drag direction is kept.
#[must_use]
pub fn snap_round(start: Vec2, end: Vec2) -> Vec2 {
    let _ = (start, end);
    todo!()
}

/// `end` moved so each side of the box from `start` is `k · c` for a size
/// `c` in `sizes` and `k` in `1..=MAX_SIZE_MULTIPLE`, when within
/// `tolerance` (world units). The nearest wins, the smaller `k` on ties.
#[must_use]
pub fn snap_size(start: Vec2, end: Vec2, sizes: &[f32], tolerance: f32) -> Vec2 {
    let _ = (start, end, sizes, tolerance);
    todo!()
}

/// `p` moved per axis onto the nearest target line within `tolerance`.
#[must_use]
pub fn align_point(p: Vec2, targets: &Targets, tolerance: f32) -> Vec2 {
    let _ = (p, targets, tolerance);
    todo!()
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
    let _ = (start, end, kind, targets, tolerance);
    todo!()
}

/// One guide for each x/y line of the drag (box edges and centre, or the
/// two points) lying on a target line, spanning the target and the drag.
#[must_use]
pub fn guides(start: Vec2, end: Vec2, kind: DragKind, targets: &Targets) -> Vec<[Vec2; 2]> {
    let _ = (start, end, kind, targets);
    todo!()
}

/// Snaps a drag's start point: grid, then alignment. No guides.
#[must_use]
pub fn snap_point(p: Vec2, snaps: Snaps, targets: &Targets, camera: &Camera) -> Vec2 {
    let _ = (p, snaps, targets, camera);
    todo!()
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
    let _ = (start, end, kind, snaps, targets, camera);
    todo!()
}

/// [`snap_point`] with the helpers, document and camera of `view`.
#[must_use]
pub fn snap_start(p: Vec2, mods: Modifiers, view: &ToolView<'_>) -> Vec2 {
    let _ = (p, mods, view);
    todo!()
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
    let _ = (start, end, kind, mods, view);
    todo!()
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
