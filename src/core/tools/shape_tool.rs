//! Line, arrow, rectangle and ellipse tools (`ctx.tool` says which).
//!
//! Owned by T09; the signatures are fixed by ADR-T08-1, the bodies are stubs
//! until T09 fills them in.

use super::{Overlay, Pointer, ToolCtx, ToolView};

/// Gesture state of this tool.
#[derive(Debug, Clone, Default)]
pub struct State;

/// Handles one pointer event of a gesture. Returns whether a redraw is needed.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    let _ = (state, ctx, pointer);
    false
}

/// What the gesture in progress draws on top of the document.
#[must_use]
pub fn preview(state: &State, view: &ToolView<'_>) -> Overlay {
    let _ = (state, view);
    Overlay::default()
}

/// Discards the gesture in progress without changing the document. Returns
/// whether a redraw is needed.
pub fn cancel(state: &mut State) -> bool {
    let _ = state;
    false
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
            let mods = Modifiers {
                shift,
                ..Modifiers::NONE
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
            self.doc.shapes().next().map(|(_, s)| s).expect("one shape")
        }
    }

    /// The defining points of a two-point shape.
    fn ends(shape: &Shape) -> (Vec2, Vec2) {
        match shape {
            Shape::Line { a, b, .. }
            | Shape::Arrow { a, b, .. }
            | Shape::Rect { a, b, .. }
            | Shape::Ellipse { a, b, .. } => (*a, *b),
            Shape::Stroke { .. } => panic!("expected a two-point shape, got a stroke"),
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
        assert!(b.approx_eq(Vec2::new(30.0, 40.0), 1e-4), "head at the release point");
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
        f.style.color = crate::core::palette::ColorId::new(2).expect("colour 2");

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
        let changed = f.drag((10.0, 10.0), (11.5, 10.0), false);

        // Assert
        assert!(!changed);
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
        ) {
            let mut f = Fixture::new(tool);
            f.camera = Camera::new(Vec2::new(13.0, -7.0), zoom);

            f.send_mods(Phase::Down, from.0, from.1, shift);
            for (x, y, s) in moves {
                f.send_mods(Phase::Move, x, y, s);
            }
            f.send_mods(Phase::Up, to.0, to.1, shift);

            prop_assert!(f.history.undo_len() <= 1);
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
