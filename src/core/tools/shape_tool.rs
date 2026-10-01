//! Line, arrow, rectangle and ellipse tools (`ctx.tool` says which).
//!
//! A drag from `Down` to `Up` spans the shape; while dragging the shape is
//! shown as a preview and on `Up` it is committed as one undo step
//! (ADR-T08-1). `Shift` constrains the shape: square, circle, or a 45° step
//! for lines and arrows. Drags shorter than [`MIN_DRAG_PX`] on screen are
//! treated as stray clicks and commit nothing.
//!
//! With the snap helpers on, the start and the moving end are snapped
//! (ADR-T17-1): `Alt` turns snapping off for an event, and with `Shift`
//! only grid snap runs before the constraint.
//!
//! With numbering on, rectangles and ellipses carry the next number, in the
//! preview and when committed (ADR-T19-1); the editor moves the counter.

use super::{Overlay, Phase, Pointer, ToolCtx, ToolView};
use crate::core::command::Tool;
use crate::core::document::tx_insert;
use crate::core::geom::Vec2;
use crate::core::numbering;
use crate::core::shape::{Shape, Style};
use crate::core::snap::{self, DragKind, Snapped};

/// Shortest drag, in screen pixels, that creates a shape.
pub const MIN_DRAG_PX: f32 = 2.0;

/// Gesture state of this tool.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// The drag in progress, if the button is down.
    drag: Option<Drag>,
}

/// Which shape a drag creates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// [`Shape::Line`].
    Line,
    /// [`Shape::Arrow`].
    Arrow,
    /// [`Shape::Rect`].
    Rect,
    /// [`Shape::Ellipse`].
    Ellipse,
}

impl Kind {
    /// What a drag of this kind spans, for snapping.
    fn drag_kind(self) -> DragKind {
        match self {
            Self::Line | Self::Arrow => DragKind::Point,
            Self::Rect | Self::Ellipse => DragKind::Box,
        }
    }

    /// The kind drawn by `tool`, `None` for tools that are not shape tools.
    fn from_tool(tool: Tool) -> Option<Self> {
        match tool {
            Tool::Line => Some(Self::Line),
            Tool::Arrow => Some(Self::Arrow),
            Tool::Rect => Some(Self::Rect),
            Tool::Ellipse => Some(Self::Ellipse),
            Tool::Pen | Tool::Grid | Tool::Eraser | Tool::Bucket | Tool::Select | Tool::Hand => {
                None
            }
        }
    }
}

/// A drag in progress.
#[derive(Debug, Clone)]
struct Drag {
    /// Shape being created.
    kind: Kind,
    /// World position of the `Down`, snapped.
    start: Vec2,
    /// World position of the latest event, snapped.
    end: Vec2,
    /// Alignment guides of the latest event.
    guides: Vec<[Vec2; 2]>,
    /// Whether `Shift` was held on the latest event.
    shift: bool,
    /// Style sampled on `Down`, width already in world units.
    style: Style,
}

impl Drag {
    /// The shape this drag currently spans, constrained if `shift` is held.
    fn shape(&self) -> Shape {
        let (a, style) = (self.start, self.style);
        let b = match (self.shift, self.kind) {
            (false, _) => self.end,
            (true, Kind::Line | Kind::Arrow) => constrain_45(a, self.end),
            (true, Kind::Rect | Kind::Ellipse) => constrain_square(a, self.end),
        };
        match self.kind {
            Kind::Line => Shape::Line { a, b, style },
            Kind::Arrow => Shape::Arrow { a, b, style },
            Kind::Rect => Shape::Rect {
                a,
                b,
                style,
                fill: None,
                label: None,
            },
            Kind::Ellipse => Shape::Ellipse {
                a,
                b,
                style,
                fill: None,
                label: None,
            },
        }
    }
}

/// Handles one pointer event of a gesture. Returns whether a redraw is needed.
///
/// `Down` starts a drag if `ctx.tool` is a shape tool, `Move` updates its end
/// and `Shift` state, `Up` commits the shape as one undo step unless the drag
/// is shorter than [`MIN_DRAG_PX`] on screen. Events without a drag are
/// ignored.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    let world = ctx.camera.screen_to_world(pointer.pos);
    let shift = pointer.mods.shift;
    match pointer.phase {
        Phase::Down => {
            let Some(kind) = Kind::from_tool(ctx.tool) else {
                return cancel(state);
            };
            let start = snap::snap_start(world, pointer.mods, &ctx.view());
            state.drag = Some(Drag {
                kind,
                start,
                end: start,
                guides: Vec::new(),
                shift,
                style: Style {
                    color: ctx.style.color,
                    width: ctx.camera.world_len(ctx.style.width_px),
                },
            });
            true
        }
        Phase::Move => match &mut state.drag {
            Some(drag) => {
                let Snapped { point, guides } = snap::snap_end(
                    drag.start,
                    world,
                    drag.kind.drag_kind(),
                    pointer.mods,
                    &ctx.view(),
                );
                let changed = drag.end != point || drag.shift != shift || drag.guides != guides;
                drag.end = point;
                drag.guides = guides;
                drag.shift = shift;
                changed
            }
            None => false,
        },
        Phase::Up => {
            let Some(mut drag) = state.drag.take() else {
                return false;
            };
            let snapped = snap::snap_end(
                drag.start,
                world,
                drag.kind.drag_kind(),
                pointer.mods,
                &ctx.view(),
            );
            drag.end = snapped.point;
            drag.shift = shift;
            let drag_px = ctx.camera.screen_len(drag.start.distance(drag.end));
            if drag_px.is_nan() || drag_px < MIN_DRAG_PX {
                return true;
            }
            let shape = numbering::label_new(drag.shape(), &ctx.style.helpers);
            if shape.is_finite() {
                let tx = tx_insert(ctx.doc, [shape]);
                ctx.commit(tx);
            }
            true
        }
    }
}

/// What the gesture in progress draws on top of the document: the shape
/// being dragged (with the number it would get) and its alignment guides,
/// or nothing when idle.
#[must_use]
pub fn preview(state: &State, view: &ToolView<'_>) -> Overlay {
    Overlay {
        shapes: state
            .drag
            .iter()
            .map(|drag| numbering::label_new(drag.shape(), &view.style.helpers))
            .collect(),
        guides: state
            .drag
            .as_ref()
            .map(|drag| drag.guides.clone())
            .unwrap_or_default(),
        ..Overlay::default()
    }
}

/// Discards the gesture in progress without changing the document. Returns
/// whether a redraw is needed (there was a drag to discard).
pub fn cancel(state: &mut State) -> bool {
    state.drag.take().is_some()
}

/// `end` moved so that the box from `start` is a square.
fn constrain_square(start: Vec2, end: Vec2) -> Vec2 {
    let d = end - start;
    let side = d.x.abs().max(d.y.abs());
    start + Vec2::new(side.copysign(d.x), side.copysign(d.y))
}

/// `end` projected onto the multiple of 45° from `start` nearest the drag.
fn constrain_45(start: Vec2, end: Vec2) -> Vec2 {
    let d = end - start;
    let step = std::f32::consts::FRAC_PI_4;
    let angle = (d.y.atan2(d.x) / step).round() * step;
    let dir = Vec2::new(angle.cos(), angle.sin());
    start + dir * d.dot(dir).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::{Camera, ZOOM_MAX, ZOOM_MIN};
    use crate::core::clipboard::Clipboard;
    use crate::core::command::Tool;
    use crate::core::document::{Document, ShapeId};
    use crate::core::editor::DrawStyle;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::history::History;
    use crate::core::input::Modifiers;
    use crate::core::shape::Shape;
    use crate::core::smoothing::SmoothingLevel;
    use crate::core::tools::Phase;
    use proptest::prelude::*;

    /// Editor state a shape gesture runs against.
    struct Fixture {
        doc: Document,
        history: History,
        selection: Vec<ShapeId>,
        clipboard: Clipboard,
        camera: Camera,
        tool: Tool,
        style: DrawStyle,
        state: State,
    }

    impl Fixture {
        fn new(tool: Tool) -> Self {
            Self {
                doc: Document::new(),
                history: History::new(),
                selection: Vec::new(),
                clipboard: Clipboard::default(),
                camera: Camera::default(),
                tool,
                style: DrawStyle::default(),
                state: State::default(),
            }
        }

        fn send_mods(&mut self, phase: Phase, x: f32, y: f32, shift: bool) -> bool {
            let mods = Modifiers {
                shift,
                ..Modifiers::NONE
            };
            self.send_with(phase, x, y, mods)
        }

        fn send_with(&mut self, phase: Phase, x: f32, y: f32, mods: Modifiers) -> bool {
            let pos = Vec2::new(x, y);
            let mut ctx = ToolCtx {
                doc: &mut self.doc,
                history: &mut self.history,
                selection: &mut self.selection,
                clipboard: &mut self.clipboard,
                camera: &self.camera,
                tool: self.tool,
                style: self.style,
                smoothing: SmoothingLevel::Medium,
                cursor: pos,
            };
            on_pointer(&mut self.state, &mut ctx, Pointer { phase, pos, mods })
        }

        fn send(&mut self, phase: Phase, x: f32, y: f32) -> bool {
            self.send_mods(phase, x, y, false)
        }

        /// A full drag from `from` to `to` (screen), one move in between.
        fn drag(&mut self, from: (f32, f32), to: (f32, f32), shift: bool) -> bool {
            self.send_mods(Phase::Down, from.0, from.1, shift);
            self.send_mods(Phase::Move, to.0, to.1, shift);
            self.send_mods(Phase::Up, to.0, to.1, shift)
        }

        /// A full drag with `mods` held throughout.
        fn drag_with(&mut self, from: (f32, f32), to: (f32, f32), mods: Modifiers) -> bool {
            self.send_with(Phase::Down, from.0, from.1, mods);
            self.send_with(Phase::Move, to.0, to.1, mods);
            self.send_with(Phase::Up, to.0, to.1, mods)
        }

        /// Puts `shape` in the document as one undo step.
        fn insert(&mut self, shape: Shape) {
            let tx = tx_insert(&mut self.doc, [shape]);
            assert!(self.history.commit(&mut self.doc, tx).is_ok());
        }

        /// The shape added last.
        fn last_shape(&self) -> &Shape {
            match self.doc.shapes().last() {
                Some((_, shape)) => shape,
                None => panic!("document is empty"),
            }
        }

        fn overlay(&self) -> Overlay {
            preview(
                &self.state,
                &ToolView {
                    doc: &self.doc,
                    camera: &self.camera,
                    selection: &self.selection,
                    tool: self.tool,
                    style: self.style,
                    smoothing: SmoothingLevel::Medium,
                    cursor: Vec2::ZERO,
                },
            )
        }

        fn only_shape(&self) -> &Shape {
            assert_eq!(self.doc.len(), 1, "expected exactly one shape");
            match self.doc.shapes().next() {
                Some((_, shape)) => shape,
                None => panic!("document is empty"),
            }
        }
    }

    /// The defining points of a two-point shape.
    fn ends(shape: &Shape) -> (Vec2, Vec2) {
        match shape {
            Shape::Line { a, b, .. }
            | Shape::Arrow { a, b, .. }
            | Shape::Rect { a, b, .. }
            | Shape::Ellipse { a, b, .. } => (*a, *b),
            Shape::Stroke { .. } | Shape::Grid { .. } => {
                panic!("expected a two-point shape, got {shape:?}")
            }
        }
    }

    #[test]
    fn shape_drag_commits_line() {
        let mut f = Fixture::new(Tool::Line);

        let changed = f.drag((10.0, 20.0), (50.0, 80.0), false);

        assert!(changed);
        let shape = f.only_shape();
        assert!(matches!(shape, Shape::Line { .. }));
        let (a, b) = ends(shape);
        assert!(a.approx_eq(Vec2::new(10.0, 20.0), 1e-4));
        assert!(b.approx_eq(Vec2::new(50.0, 80.0), 1e-4));
    }

    #[test]
    fn shape_drag_commits_arrow() {
        let mut f = Fixture::new(Tool::Arrow);

        assert!(f.drag((0.0, 0.0), (30.0, 40.0), false));

        let shape = f.only_shape();
        assert!(matches!(shape, Shape::Arrow { .. }));
        let (a, b) = ends(shape);
        assert!(a.approx_eq(Vec2::ZERO, 1e-4));
        assert!(
            b.approx_eq(Vec2::new(30.0, 40.0), 1e-4),
            "head at the release point"
        );
    }

    #[test]
    fn shape_drag_commits_rect() {
        let mut f = Fixture::new(Tool::Rect);

        assert!(f.drag((0.0, 0.0), (30.0, 40.0), false));

        let shape = f.only_shape();
        assert!(matches!(shape, Shape::Rect { fill: None, .. }));
        let (a, b) = ends(shape);
        assert!(a.approx_eq(Vec2::ZERO, 1e-4));
        assert!(b.approx_eq(Vec2::new(30.0, 40.0), 1e-4));
    }

    #[test]
    fn shape_drag_commits_ellipse() {
        let mut f = Fixture::new(Tool::Ellipse);

        assert!(f.drag((0.0, 0.0), (30.0, 40.0), false));

        let shape = f.only_shape();
        assert!(matches!(shape, Shape::Ellipse { fill: None, .. }));
        let (a, b) = ends(shape);
        assert!(a.approx_eq(Vec2::ZERO, 1e-4));
        assert!(b.approx_eq(Vec2::new(30.0, 40.0), 1e-4));
    }

    #[test]
    fn shape_drag_uses_world_coordinates_and_style_color() {
        let mut f = Fixture::new(Tool::Rect);
        f.camera = Camera::new(Vec2::new(-40.0, 10.0), 0.5);
        let Some(color) = crate::core::palette::ColorId::new(2) else {
            panic!("palette has colour 2");
        };
        f.style.color = color;

        assert!(f.drag((0.0, 0.0), (30.0, 40.0), false));

        let shape = f.only_shape();
        let (a, b) = ends(shape);
        assert!(a.approx_eq(f.camera.screen_to_world(Vec2::ZERO), 1e-4));
        assert!(b.approx_eq(f.camera.screen_to_world(Vec2::new(30.0, 40.0)), 1e-4));
        assert_eq!(shape.style().color, f.style.color);
    }

    #[test]
    fn shape_preview_while_dragging() {
        // Arrange
        let mut f = Fixture::new(Tool::Ellipse);

        // Act
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 20.0, 10.0);
        let overlay = f.overlay();

        // Assert
        assert!(f.doc.is_empty(), "nothing is committed before up");
        assert_eq!(overlay.shapes.len(), 1);
        assert!(matches!(overlay.shapes[0], Shape::Ellipse { .. }));
        let (_, b) = ends(&overlay.shapes[0]);
        assert!(b.approx_eq(Vec2::new(20.0, 10.0), 1e-4));
        assert!(overlay.hidden.is_empty());
        assert!(overlay.marquee.is_none());
    }

    #[test]
    fn shape_preview_is_empty_when_idle() {
        let f = Fixture::new(Tool::Rect);

        assert!(f.overlay().is_empty());
    }

    #[test]
    fn shape_preview_is_empty_after_commit() {
        let mut f = Fixture::new(Tool::Rect);

        f.drag((0.0, 0.0), (30.0, 40.0), false);

        assert!(f.overlay().is_empty());
    }

    #[test]
    fn shape_drag_shorter_than_two_px_commits_nothing() {
        // Arrange: at zoom 0.5 a 1.5 px drag is 3 world units long, so the
        // threshold must be measured on screen.
        let mut f = Fixture::new(Tool::Line);
        f.camera = Camera::new(Vec2::ZERO, 0.5);

        // Act
        let redraw = f.drag((10.0, 10.0), (11.5, 10.0), false);

        // Assert: the tiny preview must still be erased, so a redraw is
        // requested, but the document is untouched.
        assert!(redraw);
        assert!(f.doc.is_empty());
        assert!(!f.history.can_undo());
        assert!(f.overlay().is_empty());
    }

    #[test]
    fn shape_drag_of_two_px_commits() {
        let mut f = Fixture::new(Tool::Rect);
        f.camera = Camera::new(Vec2::ZERO, 4.0);

        assert!(f.drag((10.0, 10.0), (12.0, 10.0), false));
        assert_eq!(f.doc.len(), 1);
        assert!(approx_eq(MIN_DRAG_PX, 2.0, 1e-6));
    }

    #[test]
    fn shape_tool_ignores_non_shape_tool() {
        let mut f = Fixture::new(Tool::Pen);

        assert!(!f.drag((0.0, 0.0), (30.0, 40.0), false));
        assert!(f.doc.is_empty());
    }

    #[test]
    fn shape_width_is_zoom_independent_on_screen() {
        let mut f = Fixture::new(Tool::Line);
        f.camera = Camera::new(Vec2::ZERO, 4.0);
        f.style.width_px = 8.0;

        assert!(f.drag((0.0, 0.0), (30.0, 40.0), false));

        let width = f.only_shape().style().width;
        assert!(approx_eq(width, 2.0, 1e-5));
        assert!(approx_eq(f.camera.screen_len(width), 8.0, 1e-5));
    }

    #[test]
    fn shift_constrains_rect_to_square() {
        let mut f = Fixture::new(Tool::Rect);

        assert!(f.drag((10.0, 10.0), (50.0, -20.0), true));

        let (a, b) = ends(f.only_shape());
        assert!(a.approx_eq(Vec2::new(10.0, 10.0), 1e-4));
        assert!(b.approx_eq(Vec2::new(50.0, -30.0), 1e-4));
    }

    #[test]
    fn shift_constrains_ellipse_to_circle() {
        let mut f = Fixture::new(Tool::Ellipse);

        assert!(f.drag((0.0, 0.0), (-10.0, 30.0), true));

        let (a, b) = ends(f.only_shape());
        assert!(a.approx_eq(Vec2::ZERO, 1e-4));
        assert!(b.approx_eq(Vec2::new(-30.0, 30.0), 1e-4));
    }

    #[test]
    fn shift_constrains_line_to_nearest_45_degrees() {
        let mut f = Fixture::new(Tool::Line);

        assert!(f.drag((0.0, 0.0), (100.0, 10.0), true));

        let (a, b) = ends(f.only_shape());
        assert!(a.approx_eq(Vec2::ZERO, 1e-4));
        assert!(b.approx_eq(Vec2::new(100.0, 0.0), 1e-3));
    }

    #[test]
    fn shift_constrains_arrow_to_diagonal() {
        let mut f = Fixture::new(Tool::Arrow);

        assert!(f.drag((0.0, 0.0), (40.0, -50.0), true));

        let (_, b) = ends(f.only_shape());
        assert!(b.approx_eq(Vec2::new(45.0, -45.0), 1e-3));
    }

    #[test]
    fn shift_released_before_up_is_unconstrained() {
        let mut f = Fixture::new(Tool::Rect);

        f.send_mods(Phase::Down, 0.0, 0.0, true);
        f.send_mods(Phase::Move, 30.0, 10.0, true);
        f.send_mods(Phase::Up, 30.0, 10.0, false);

        let (_, b) = ends(f.only_shape());
        assert!(b.approx_eq(Vec2::new(30.0, 10.0), 1e-4));
    }

    #[test]
    fn cancel_discards_gesture() {
        // Arrange
        let mut f = Fixture::new(Tool::Rect);
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 30.0, 40.0);

        // Act
        let cancelled = cancel(&mut f.state);

        // Assert
        assert!(cancelled);
        assert!(f.overlay().is_empty());
        assert!(!f.send(Phase::Up, 30.0, 40.0));
        assert!(f.doc.is_empty());
        assert!(!f.history.can_undo());
    }

    #[test]
    fn cancel_when_idle_returns_false() {
        let mut f = Fixture::new(Tool::Line);

        assert!(!cancel(&mut f.state));
    }

    #[test]
    fn one_gesture_one_undo() {
        // Arrange
        let mut f = Fixture::new(Tool::Arrow);

        // Act
        f.send(Phase::Down, 0.0, 0.0);
        for i in 1..20_u8 {
            f.send(Phase::Move, f32::from(i) * 5.0, f32::from(i));
        }
        f.send(Phase::Up, 100.0, 20.0);

        // Assert
        assert_eq!(f.history.undo_len(), 1);
        assert_eq!(f.doc.len(), 1);
        assert!(f.history.undo(&mut f.doc));
        assert!(f.doc.is_empty());
    }

    const STYLE: crate::core::shape::Style = crate::core::shape::Style {
        color: crate::core::palette::ColorId::INK,
        width: 1.0,
    };

    /// A 50 × 30 rectangle at the origin.
    fn box_50x30() -> Shape {
        Shape::Rect {
            a: Vec2::ZERO,
            b: Vec2::new(50.0, 30.0),
            style: STYLE,
            fill: None,
            label: None,
        }
    }

    const ALT: Modifiers = Modifiers {
        shift: false,
        ctrl: false,
        alt: true,
    };

    #[test]
    fn snap_off_by_default_is_unchanged() {
        let mut f = Fixture::new(Tool::Rect);
        f.insert(box_50x30());

        assert!(f.drag((100.0, 101.0), (152.0, 131.0), false));

        let (a, b) = ends(f.last_shape());
        assert!(a.approx_eq(Vec2::new(100.0, 101.0), 1e-4));
        assert!(b.approx_eq(Vec2::new(152.0, 131.0), 1e-4));
    }

    #[test]
    fn snap_grid_applies_to_line_endpoints() {
        let mut f = Fixture::new(Tool::Line);
        f.style.helpers.grid_snap = true;

        assert!(f.drag((11.0, 9.0), (52.0, 68.0), false));

        let (a, b) = ends(f.only_shape());
        assert!(a.approx_eq(Vec2::new(20.0, 0.0), 1e-4), "{a:?}");
        assert!(b.approx_eq(Vec2::new(60.0, 60.0), 1e-4), "{b:?}");
    }

    #[test]
    fn snap_smart_rect_matches_existing_size() {
        // Arrange
        let mut f = Fixture::new(Tool::Rect);
        f.style.helpers.smart_snap = true;
        f.insert(box_50x30());

        // Act: start near y = 0, end near a 50 × 30 box.
        f.send(Phase::Down, 200.0, 3.0);
        f.send(Phase::Move, 253.0, 28.0);
        let overlay = f.overlay();
        f.send(Phase::Up, 253.0, 28.0);

        // Assert: aligned top, matched size, guides while dragging.
        let (a, b) = ends(f.last_shape());
        assert!(a.approx_eq(Vec2::new(200.0, 0.0), 1e-4), "{a:?}");
        assert!(b.approx_eq(Vec2::new(250.0, 30.0), 1e-4), "{b:?}");
        assert!(!overlay.guides.is_empty());
        assert!(f.overlay().is_empty(), "guides go away with the drag");
    }

    #[test]
    fn snap_alt_disables_snapping() {
        let mut f = Fixture::new(Tool::Rect);
        f.style.helpers.smart_snap = true;
        f.style.helpers.grid_snap = true;
        f.insert(box_50x30());

        f.send_with(Phase::Down, 200.0, 3.0, ALT);
        f.send_with(Phase::Move, 253.0, 28.0, ALT);
        let overlay = f.overlay();
        f.send_with(Phase::Up, 253.0, 28.0, ALT);

        let (a, b) = ends(f.last_shape());
        assert!(a.approx_eq(Vec2::new(200.0, 3.0), 1e-4), "{a:?}");
        assert!(b.approx_eq(Vec2::new(253.0, 28.0), 1e-4), "{b:?}");
        assert!(overlay.guides.is_empty());
    }

    #[test]
    fn snap_shift_constraint_wins() {
        // Arrange
        let mut f = Fixture::new(Tool::Rect);
        f.style.helpers.smart_snap = true;
        f.style.helpers.grid_snap = true;
        f.insert(box_50x30());

        // Act
        f.send_mods(Phase::Down, 200.0, 0.0, true);
        f.send_mods(Phase::Move, 233.0, 47.0, true);
        let overlay = f.overlay();
        f.send_mods(Phase::Up, 233.0, 47.0, true);

        // Assert: grid takes the end to (240, 40), Shift squares it; no
        // size snap to 50 × 30, no guides.
        let (a, b) = ends(f.last_shape());
        assert!(a.approx_eq(Vec2::new(200.0, 0.0), 1e-4), "{a:?}");
        assert!(b.approx_eq(Vec2::new(240.0, 40.0), 1e-4), "{b:?}");
        assert!(overlay.guides.is_empty());
    }

    #[test]
    fn snap_line_is_not_size_snapped() {
        let mut f = Fixture::new(Tool::Line);
        f.style.helpers.smart_snap = true;
        f.insert(box_50x30());

        assert!(f.drag_with((200.0, 200.0), (253.0, 253.0), Modifiers::NONE));

        let (_, b) = ends(f.last_shape());
        assert!(b.approx_eq(Vec2::new(253.0, 253.0), 1e-4), "{b:?}");
    }

    // ---- T19 numbering ----------------------------------------------------

    /// Numbering on, next number `next`.
    fn numbering(f: &mut Fixture, next: u32) {
        f.style.helpers.numbering = true;
        f.style.helpers.next_number = next;
    }

    #[test]
    fn preview_shows_next_number() {
        // Arrange
        let mut f = Fixture::new(Tool::Ellipse);
        numbering(&mut f, 4);

        // Act
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 30.0, 30.0);
        let overlay = f.overlay();

        // Assert
        assert_eq!(overlay.shapes.len(), 1);
        assert_eq!(overlay.shapes[0].label(), Some(4));
    }

    #[test]
    fn commit_labels_ellipse() {
        let mut f = Fixture::new(Tool::Ellipse);
        numbering(&mut f, 9);

        assert!(f.drag((0.0, 0.0), (30.0, 30.0), false));

        assert_eq!(f.only_shape().label(), Some(9));
    }

    #[test]
    fn commit_leaves_arrow_unlabelled() {
        let mut f = Fixture::new(Tool::Arrow);
        numbering(&mut f, 9);

        assert!(f.drag((0.0, 0.0), (30.0, 30.0), false));

        assert_eq!(f.only_shape().label(), None);
    }

    fn shape_kind() -> impl Strategy<Value = Tool> {
        prop_oneof![
            Just(Tool::Line),
            Just(Tool::Arrow),
            Just(Tool::Rect),
            Just(Tool::Ellipse),
        ]
    }

    fn coord() -> impl Strategy<Value = f32> {
        -5000.0_f32..5000.0
    }

    proptest! {
        #[test]
        fn shape_gesture_never_panics_and_is_finite(
            tool in shape_kind(),
            zoom in ZOOM_MIN..ZOOM_MAX,
            from in (coord(), coord()),
            moves in proptest::collection::vec((coord(), coord(), any::<bool>()), 0..8),
            to in (coord(), coord()),
            shift in any::<bool>(),
            alt in any::<bool>(),
            grid_snap in any::<bool>(),
            smart_snap in any::<bool>(),
            neighbour in (coord(), coord(), coord(), coord()),
        ) {
            let mut f = Fixture::new(tool);
            f.camera = Camera::new(Vec2::new(13.0, -7.0), zoom);
            f.style.helpers.grid_snap = grid_snap;
            f.style.helpers.smart_snap = smart_snap;
            f.insert(Shape::Rect {
                a: Vec2::new(neighbour.0, neighbour.1),
                b: Vec2::new(neighbour.2, neighbour.3),
                style: STYLE,
                fill: None,
                label: None,
            });
            let mods = |shift| Modifiers { shift, ctrl: false, alt };

            f.send_with(Phase::Down, from.0, from.1, mods(shift));
            for (x, y, s) in moves {
                f.send_with(Phase::Move, x, y, mods(s));
            }
            f.send_with(Phase::Up, to.0, to.1, mods(shift));

            prop_assert!(f.history.undo_len() <= 2);
            prop_assert_eq!(f.doc.len(), f.history.undo_len());
            prop_assert!(f.doc.shapes().all(|(_, s)| s.is_finite()));
            prop_assert!(f.overlay().is_empty());
        }

        #[test]
        fn shift_constrained_line_angle_is_multiple_of_45_degrees(
            from in (coord(), coord()),
            to in (coord(), coord()),
        ) {
            let mut f = Fixture::new(Tool::Line);

            f.drag(from, to, true);

            if let Some((_, shape)) = f.doc.shapes().next() {
                let (a, b) = ends(shape);
                let d = b - a;
                let tol = 1e-3 * d.length().max(1.0);
                prop_assert!(
                    d.x.abs() <= tol
                        || d.y.abs() <= tol
                        || (d.x.abs() - d.y.abs()).abs() <= tol,
                    "{d:?} is not on a 45 degree direction"
                );
            }
        }
    }
}
