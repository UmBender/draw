//! Eraser: removes whole shapes it touches (also right drag from any tool).
//!
//! A drag marks every shape within [`ERASER_TOLERANCE_PX`] of the pointer
//! path; the preview hides the marked shapes and the release removes them as
//! one undo step. Between two pointer events the path is sampled at most one
//! tolerance apart, so fast drags do not skip thin shapes.

use super::{Overlay, Phase, Pointer, ToolCtx, ToolView};
use crate::core::document::{ShapeId, tx_remove};
use crate::core::geom::Vec2;

/// Hit tolerance of the eraser in screen pixels.
pub const ERASER_TOLERANCE_PX: f32 = 6.0;

/// Most samples taken between two pointer events (bounds the work of a huge
/// jump; at 6 px spacing this covers over 6000 px).
const MAX_SAMPLES: usize = 1024;

/// Gesture state of this tool.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Last pointer position (screen) of the drag, `None` when idle.
    last: Option<Vec2>,
    /// Shapes marked for removal, in the order they were touched.
    marked: Vec<ShapeId>,
}

/// Marks every unmarked shape hit by the segment `from → to` (screen).
/// Returns whether anything new was marked.
fn mark_along(state: &mut State, ctx: &ToolCtx<'_>, from: Vec2, to: Vec2) -> bool {
    let tol = ctx.camera.world_len(ERASER_TOLERANCE_PX);
    // `as` saturates (NaN becomes 0), and the clamp keeps at least one step.
    let steps = ((from.distance(to) / ERASER_TOLERANCE_PX).ceil() as usize).clamp(1, MAX_SAMPLES);
    let before = state.marked.len();
    for step in 0..=steps {
        let world = ctx
            .camera
            .screen_to_world(from.lerp(to, step as f32 / steps as f32));
        for (id, shape) in ctx.doc.shapes() {
            if !state.marked.contains(&id) && shape.hit(world, tol) {
                state.marked.push(id);
            }
        }
    }
    state.marked.len() > before
}

/// Handles one pointer event of a gesture. Returns whether a redraw is needed.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    if pointer.phase == Phase::Down {
        state.marked.clear();
    }
    let from = state.last.unwrap_or(pointer.pos);
    let marked_more = mark_along(state, ctx, from, pointer.pos);
    match pointer.phase {
        Phase::Down | Phase::Move => {
            state.last = Some(pointer.pos);
            marked_more
        }
        Phase::Up => {
            // A rejected removal still needs a redraw to unhide the marks.
            let marked = std::mem::take(state).marked;
            let tx = tx_remove(ctx.doc, &marked);
            ctx.commit(tx) || !marked.is_empty()
        }
    }
}

/// What the gesture in progress draws on top of the document.
#[must_use]
pub fn preview(state: &State, view: &ToolView<'_>) -> Overlay {
    let _ = view;
    Overlay {
        hidden: state.marked.clone(),
        ..Overlay::default()
    }
}

/// Discards the gesture in progress without changing the document. Returns
/// whether a redraw is needed.
pub fn cancel(state: &mut State) -> bool {
    let active = state.last.is_some() || !state.marked.is_empty();
    *state = State::default();
    active
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::Camera;
    use crate::core::clipboard::testkit::{Fixture, line, ptr};
    use crate::core::command::Tool;
    use crate::core::geom::Vec2;
    use crate::core::tools::Phase;

    fn send(state: &mut State, fx: &mut Fixture, phase: Phase, x: f32, y: f32) -> bool {
        on_pointer(state, &mut fx.ctx(Tool::Eraser), ptr(phase, x, y))
    }

    fn three_lines() -> (Fixture, Vec<crate::core::document::ShapeId>) {
        Fixture::new(vec![
            line(0.0, 0.0, 100.0, 0.0),
            line(0.0, 50.0, 100.0, 50.0),
            line(0.0, 200.0, 100.0, 200.0),
        ])
    }

    #[test]
    fn down_on_shape_marks_it_hidden() {
        // Arrange
        let (mut fx, ids) = three_lines();
        let mut state = State::default();

        // Act
        let redraw = send(&mut state, &mut fx, Phase::Down, 50.0, 3.0);

        // Assert
        assert!(redraw);
        let overlay = preview(&state, &fx.view(Tool::Eraser));
        assert_eq!(overlay.hidden, vec![ids[0]]);
        assert!(overlay.shapes.is_empty());
        assert_eq!(fx.doc.len(), 3);
    }

    #[test]
    fn drag_marks_every_shape_touched() {
        let (mut fx, ids) = three_lines();
        let mut state = State::default();

        send(&mut state, &mut fx, Phase::Down, 10.0, 0.0);
        send(&mut state, &mut fx, Phase::Move, 10.0, 50.0);

        let hidden = preview(&state, &fx.view(Tool::Eraser)).hidden;
        assert_eq!(hidden.len(), 2);
        assert!(hidden.contains(&ids[0]) && hidden.contains(&ids[1]));
    }

    #[test]
    fn fast_drag_samples_between_events() {
        // Arrange: a vertical line between two far-apart pointer events.
        let (mut fx, ids) = Fixture::new(vec![line(50.0, 0.0, 50.0, 100.0)]);
        let mut state = State::default();

        // Act
        send(&mut state, &mut fx, Phase::Down, 0.0, 50.0);
        send(&mut state, &mut fx, Phase::Move, 100.0, 50.0);

        // Assert
        assert_eq!(preview(&state, &fx.view(Tool::Eraser)).hidden, ids);
    }

    #[test]
    fn release_removes_marked_as_one_step() {
        let (mut fx, ids) = three_lines();
        let mut state = State::default();

        send(&mut state, &mut fx, Phase::Down, 10.0, 0.0);
        send(&mut state, &mut fx, Phase::Move, 10.0, 50.0);
        let changed = send(&mut state, &mut fx, Phase::Up, 10.0, 50.0);

        assert!(changed);
        assert_eq!(fx.ids(), vec![ids[2]]);
        assert_eq!(fx.history.undo_len(), 1);
        assert!(preview(&state, &fx.view(Tool::Eraser)).is_empty());
        assert!(fx.history.undo(&mut fx.doc));
        assert_eq!(fx.ids(), ids);
    }

    #[test]
    fn drag_over_nothing_records_no_step() {
        let (mut fx, _) = three_lines();
        let mut state = State::default();

        send(&mut state, &mut fx, Phase::Down, 10.0, 100.0);
        send(&mut state, &mut fx, Phase::Move, 90.0, 120.0);
        let changed = send(&mut state, &mut fx, Phase::Up, 90.0, 120.0);

        assert!(!changed);
        assert_eq!(fx.doc.len(), 3);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn cancel_drops_marks_and_keeps_document() {
        let (mut fx, _) = three_lines();
        let mut state = State::default();
        send(&mut state, &mut fx, Phase::Down, 50.0, 0.0);

        let redraw = cancel(&mut state);

        assert!(redraw);
        assert!(preview(&state, &fx.view(Tool::Eraser)).is_empty());
        assert_eq!(fx.doc.len(), 3);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn cancel_when_idle_needs_no_redraw() {
        let mut state = State::default();

        assert!(!cancel(&mut state));
    }

    #[test]
    fn tolerance_is_in_screen_pixels() {
        // Arrange: at zoom 2 the tolerance is 3 world units; the line's
        // half width adds 0.5, so the reach is 3.5 world = 7 px.
        let (mut fx, ids) = Fixture::new(vec![line(0.0, 0.0, 100.0, 0.0)]);
        fx.camera = Camera::new(Vec2::ZERO, 2.0);
        let (mut near, mut far) = (State::default(), State::default());

        // Act
        send(&mut near, &mut fx, Phase::Down, 50.0, ERASER_TOLERANCE_PX);
        send(
            &mut far,
            &mut fx,
            Phase::Down,
            50.0,
            ERASER_TOLERANCE_PX + 3.0,
        );

        // Assert
        assert_eq!(preview(&near, &fx.view(Tool::Eraser)).hidden, ids);
        assert!(preview(&far, &fx.view(Tool::Eraser)).hidden.is_empty());
    }
}
