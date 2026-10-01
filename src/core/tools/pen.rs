//! Freehand pen with anti-tremor smoothing.
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
    use crate::core::camera::Camera;
    use crate::core::clipboard::Clipboard;
    use crate::core::command::Tool;
    use crate::core::document::{Document, ShapeId};
    use crate::core::editor::DrawStyle;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::core::history::History;
    use crate::core::input::Modifiers;
    use crate::core::palette::ColorId;
    use crate::core::shape::Shape;
    use crate::core::smoothing::SmoothingLevel;
    use crate::core::tools::Phase;

    /// Editor state a pen gesture runs against.
    struct Fixture {
        doc: Document,
        history: History,
        selection: Vec<ShapeId>,
        clipboard: Clipboard,
        camera: Camera,
        style: DrawStyle,
        smoothing: SmoothingLevel,
        state: State,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                doc: Document::new(),
                history: History::new(),
                selection: Vec::new(),
                clipboard: Clipboard::default(),
                camera: Camera::default(),
                style: DrawStyle::default(),
                smoothing: SmoothingLevel::Off,
                state: State::default(),
            }
        }

        fn send(&mut self, phase: Phase, x: f32, y: f32) -> bool {
            let pos = Vec2::new(x, y);
            let mut ctx = ToolCtx {
                doc: &mut self.doc,
                history: &mut self.history,
                selection: &mut self.selection,
                clipboard: &mut self.clipboard,
                camera: &self.camera,
                tool: Tool::Pen,
                style: self.style,
                smoothing: self.smoothing,
                cursor: pos,
            };
            on_pointer(
                &mut self.state,
                &mut ctx,
                Pointer {
                    phase,
                    pos,
                    mods: Modifiers::NONE,
                },
            )
        }

        fn overlay(&self) -> Overlay {
            preview(
                &self.state,
                &ToolView {
                    doc: &self.doc,
                    camera: &self.camera,
                    selection: &self.selection,
                    tool: Tool::Pen,
                    style: self.style,
                    smoothing: self.smoothing,
                    cursor: Vec2::ZERO,
                },
            )
        }

        fn only_shape(&self) -> &Shape {
            assert_eq!(self.doc.len(), 1, "expected exactly one shape");
            self.doc.shapes().next().map(|(_, s)| s).expect("one shape")
        }
    }

    fn stroke_points(shape: &Shape) -> &[Vec2] {
        match shape {
            Shape::Stroke { points, .. } => points,
            other => panic!("expected a stroke, got {other:?}"),
        }
    }

    #[test]
    fn pen_drag_commits_one_stroke() {
        // Arrange
        let mut f = Fixture::new();

        // Act
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 10.0, 0.0);
        f.send(Phase::Move, 20.0, 5.0);
        let changed = f.send(Phase::Up, 30.0, 0.0);

        // Assert
        assert!(changed);
        let points = stroke_points(f.only_shape());
        assert!(points.len() >= 2);
        assert!(points[0].approx_eq(Vec2::ZERO, 1e-4));
        assert!(points[points.len() - 1].approx_eq(Vec2::new(30.0, 0.0), 1e-4));
        assert!(f.overlay().is_empty());
    }

    #[test]
    fn pen_preview_shows_live_stroke() {
        let mut f = Fixture::new();

        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 10.0, 0.0);
        f.send(Phase::Move, 20.0, 5.0);
        let overlay = f.overlay();

        assert!(f.doc.is_empty(), "nothing is committed before up");
        assert_eq!(overlay.shapes.len(), 1);
        assert_eq!(stroke_points(&overlay.shapes[0]).len(), 3);
        assert!(overlay.hidden.is_empty());
        assert!(overlay.marquee.is_none());
    }

    #[test]
    fn pen_preview_is_empty_when_idle() {
        let f = Fixture::new();

        assert!(f.overlay().is_empty());
    }

    #[test]
    fn pen_click_without_movement_commits_dot() {
        let mut f = Fixture::new();

        f.send(Phase::Down, 5.0, 7.0);
        let changed = f.send(Phase::Up, 5.0, 7.0);

        assert!(changed);
        let points = stroke_points(f.only_shape());
        assert_eq!(points.len(), 1);
        assert!(points[0].approx_eq(Vec2::new(5.0, 7.0), 1e-4));
    }

    #[test]
    fn pen_points_are_in_world_coordinates() {
        let mut f = Fixture::new();
        f.camera = Camera::new(Vec2::new(100.0, 50.0), 2.0);

        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Up, 20.0, 10.0);

        let points = stroke_points(f.only_shape());
        assert!(points[0].approx_eq(f.camera.screen_to_world(Vec2::ZERO), 1e-4));
        assert!(
            points[points.len() - 1]
                .approx_eq(f.camera.screen_to_world(Vec2::new(20.0, 10.0)), 1e-4)
        );
    }

    #[test]
    fn pen_uses_style_color() {
        let mut f = Fixture::new();
        let color = ColorId::new(3).expect("palette has colour 3");
        f.style.color = color;

        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Up, 10.0, 0.0);

        assert_eq!(f.only_shape().style().color, color);
    }

    #[test]
    fn pen_width_is_zoom_independent_on_screen() {
        // Arrange
        let mut f = Fixture::new();
        f.camera = Camera::new(Vec2::ZERO, 2.0);
        f.style.width_px = 4.0;

        // Act
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Up, 10.0, 0.0);

        // Assert
        let width = f.only_shape().style().width;
        assert!(approx_eq(width, 2.0, 1e-5));
        assert!(approx_eq(f.camera.screen_len(width), 4.0, 1e-5));
    }

    #[test]
    fn pen_move_without_down_does_nothing() {
        let mut f = Fixture::new();

        assert!(!f.send(Phase::Move, 10.0, 10.0));
        assert!(f.overlay().is_empty());
        assert!(f.doc.is_empty());
    }

    #[test]
    fn pen_up_without_down_commits_nothing() {
        let mut f = Fixture::new();

        assert!(!f.send(Phase::Up, 10.0, 10.0));
        assert!(f.doc.is_empty());
        assert!(!f.history.can_undo());
    }

    #[test]
    fn cancel_discards_gesture() {
        // Arrange
        let mut f = Fixture::new();
        f.send(Phase::Down, 0.0, 0.0);
        f.send(Phase::Move, 10.0, 0.0);

        // Act
        let cancelled = cancel(&mut f.state);

        // Assert
        assert!(cancelled);
        assert!(f.overlay().is_empty());
        assert!(!f.send(Phase::Up, 20.0, 0.0));
        assert!(f.doc.is_empty());
        assert!(!f.history.can_undo());
    }

    #[test]
    fn cancel_when_idle_returns_false() {
        let mut f = Fixture::new();

        assert!(!cancel(&mut f.state));
    }

    #[test]
    fn one_gesture_one_undo() {
        // Arrange
        let mut f = Fixture::new();
        f.smoothing = SmoothingLevel::Medium;

        // Act
        f.send(Phase::Down, 0.0, 0.0);
        for i in 1..50_u8 {
            f.send(Phase::Move, f32::from(i) * 3.0, f32::from(i % 4));
        }
        f.send(Phase::Up, 150.0, 0.0);

        // Assert
        assert_eq!(f.history.undo_len(), 1);
        assert_eq!(f.doc.len(), 1);
        assert!(f.history.undo(&mut f.doc));
        assert!(f.doc.is_empty());
    }
}
