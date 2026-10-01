//! Eraser: removes whole shapes or clears fills (also right drag from any
//! tool).
//!
//! The mode is chosen on `Down` (ADR-T21-2). A press inside a fill — a
//! filled rectangle or ellipse, or a filled grid cell — away from every
//! outline starts a *clear* drag: every fill the path passes over is
//! cleared and nothing is removed. Any other press starts a *remove* drag:
//! every shape within [`ERASER_TOLERANCE_PX`] of the path is marked. The
//! preview hides the affected shapes (showing cleared ones without their
//! fills) and the release applies everything as one undo step. Between two
//! pointer events the path is sampled at most one tolerance apart, so fast
//! drags do not skip thin shapes.

use super::{Overlay, Phase, Pointer, ToolCtx, ToolView};
use crate::core::document::{ShapeId, Transaction, tx_remove, tx_replace};
use crate::core::geom::Vec2;
use crate::core::shape::Shape;

/// Hit tolerance of the eraser in screen pixels.
pub const ERASER_TOLERANCE_PX: f32 = 6.0;

/// Most samples taken between two pointer events (bounds the work of a huge
/// jump; at 6 px spacing this covers over 6000 px).
const MAX_SAMPLES: usize = 1024;

/// What a drag does, chosen on `Down` (ADR-T21-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    /// Removes every shape the path hits.
    #[default]
    Remove,
    /// Clears the fills the path passes over; removes nothing.
    Clear,
}

/// Gesture state of this tool.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Last pointer position (screen) of the drag, `None` when idle.
    last: Option<Vec2>,
    /// What this drag does.
    mode: Mode,
    /// Shapes marked for removal, in the order they were touched.
    marked: Vec<ShapeId>,
    /// Shapes whose fills this drag cleared, with their new versions, in
    /// the order they were touched.
    cleared: Vec<(ShapeId, Shape)>,
}

/// `shape` with the fill under the world point `p` cleared: its own fill if
/// `p` is inside a filled closed shape, or the fill of the grid cell under
/// `p`. Unchanged if there is no fill at `p`.
fn clear_at(shape: &Shape, p: Vec2) -> Shape {
    let mut cleared = shape.clone();
    if cleared.fill().is_some() && cleared.contains(p) {
        cleared = cleared.with_fill(None);
    }
    if let Some((col, row)) = cleared.grid_cell_at(p) {
        cleared = cleared.with_cell_fill(col, row, None);
    }
    cleared
}

/// The mode of a drag pressed at the world point `p`: [`Mode::Clear`] if
/// no outline is within `tol` of `p` and some fill lies under it.
fn mode_at(ctx: &ToolCtx<'_>, p: Vec2, tol: f32) -> Mode {
    let on_outline = ctx.doc.shapes().any(|(_, shape)| shape.hit_outline(p, tol));
    if !on_outline
        && ctx
            .doc
            .shapes()
            .any(|(_, shape)| clear_at(shape, p) != *shape)
    {
        Mode::Clear
    } else {
        Mode::Remove
    }
}

/// Applies the drag's mode to every shape touched by the segment
/// `from → to` (screen). Returns whether anything new was marked or cleared.
fn mark_along(state: &mut State, ctx: &ToolCtx<'_>, from: Vec2, to: Vec2) -> bool {
    let tol = ctx.camera.world_len(ERASER_TOLERANCE_PX);
    // `as` saturates (NaN becomes 0), and the clamp keeps at least one step.
    let steps = ((from.distance(to) / ERASER_TOLERANCE_PX).ceil() as usize).clamp(1, MAX_SAMPLES);
    let mut changed = false;
    for step in 0..=steps {
        let world = ctx
            .camera
            .screen_to_world(from.lerp(to, step as f32 / steps as f32));
        for (id, shape) in ctx.doc.shapes() {
            match state.mode {
                Mode::Remove => {
                    if !state.marked.contains(&id) && shape.hit(world, tol) {
                        state.marked.push(id);
                        changed = true;
                    }
                }
                Mode::Clear => changed |= clear_into(&mut state.cleared, id, shape, world),
            }
        }
    }
    changed
}

/// Clears the fill at `p` of shape `id` (its cleared version so far, or
/// `shape`) into `cleared`. Returns whether a fill was cleared.
fn clear_into(cleared: &mut Vec<(ShapeId, Shape)>, id: ShapeId, shape: &Shape, p: Vec2) -> bool {
    let slot = cleared.iter().position(|(c, _)| *c == id);
    let current = slot.map_or(shape, |i| &cleared[i].1);
    let next = clear_at(current, p);
    if next == *current {
        return false;
    }
    match slot {
        Some(i) => cleared[i].1 = next,
        None => cleared.push((id, next)),
    }
    true
}

/// Handles one pointer event of a gesture. Returns whether a redraw is needed.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    if pointer.phase == Phase::Down {
        *state = State::default();
        let tol = ctx.camera.world_len(ERASER_TOLERANCE_PX);
        state.mode = mode_at(ctx, ctx.camera.screen_to_world(pointer.pos), tol);
    }
    let from = state.last.unwrap_or(pointer.pos);
    let marked_more = mark_along(state, ctx, from, pointer.pos);
    match pointer.phase {
        Phase::Down | Phase::Move => {
            state.last = Some(pointer.pos);
            marked_more
        }
        Phase::Up => {
            // A rejected edit still needs a redraw to unhide the marks.
            let State {
                marked, cleared, ..
            } = std::mem::take(state);
            let touched = !marked.is_empty() || !cleared.is_empty();
            let mut edits = Vec::new();
            for (id, shape) in cleared {
                edits.extend(tx_replace(ctx.doc, id, shape).0);
            }
            edits.extend(tx_remove(ctx.doc, &marked).0);
            ctx.commit(Transaction(edits)) || touched
        }
    }
}

/// What the gesture in progress draws on top of the document.
#[must_use]
pub fn preview(state: &State, view: &ToolView<'_>) -> Overlay {
    let _ = view;
    Overlay {
        shapes: state
            .cleared
            .iter()
            .map(|(_, shape)| shape.clone())
            .collect(),
        hidden: (state.marked.iter().copied())
            .chain(state.cleared.iter().map(|(id, _)| *id))
            .collect(),
        ..Overlay::default()
    }
}

/// Discards the gesture in progress without changing the document. Returns
/// whether a redraw is needed.
pub fn cancel(state: &mut State) -> bool {
    let active = state.last.is_some() || !state.marked.is_empty() || !state.cleared.is_empty();
    *state = State::default();
    active
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::Camera;
    use crate::core::clipboard::testkit::{Fixture, ellipse, filled_rect, line, ptr, rect};
    use crate::core::command::Tool;
    use crate::core::geom::Vec2;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use crate::core::tools::Phase;

    /// Unfilled `cols × rows` grid dragged from `(x, y)` to `(x + w, y + h)`.
    fn grid(x: f32, y: f32, w: f32, h: f32, cols: u32, rows: u32) -> Shape {
        Shape::Grid {
            a: Vec2::new(x, y),
            b: Vec2::new(x + w, y + h),
            cols,
            rows,
            style: Style {
                color: ColorId::INK,
                width: 1.0,
            },
            axes: false,
            fills: Vec::new(),
        }
    }

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

    // ---- T21 clearing fills -------------------------------------------------

    fn red() -> ColorId {
        ColorId::new(2).unwrap_or(ColorId::INK)
    }

    /// 4 × 4 grid of 25-unit cells with the given cells filled red.
    fn filled_grid(cells: &[(u32, u32)]) -> Shape {
        cells
            .iter()
            .fold(grid(0.0, 0.0, 100.0, 100.0, 4, 4), |g, &(c, r)| {
                g.with_cell_fill(c, r, Some(red()))
            })
    }

    /// A full erase gesture through `points` (screen).
    fn rub(fx: &mut Fixture, points: &[(f32, f32)]) -> bool {
        let mut state = State::default();
        let mut changed = false;
        for (i, &(x, y)) in points.iter().enumerate() {
            let phase = if i == 0 { Phase::Down } else { Phase::Move };
            changed |= send(&mut state, fx, phase, x, y);
        }
        if let Some(&(x, y)) = points.last() {
            changed |= send(&mut state, fx, Phase::Up, x, y);
        }
        changed
    }

    #[test]
    fn inside_filled_rect_clears_fill() {
        // Arrange
        let (mut fx, ids) = Fixture::new(vec![filled_rect(0.0, 0.0, 100.0, 100.0)]);

        // Act
        let changed = rub(&mut fx, &[(50.0, 50.0)]);

        // Assert
        assert!(changed);
        assert_eq!(fx.doc.len(), 1, "the rectangle stays");
        assert_eq!(fx.shape(ids[0]).fill(), None);
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn inside_filled_ellipse_clears_fill() {
        let filled = ellipse(0.0, 0.0, 100.0, 100.0).with_fill(Some(red()));
        let (mut fx, ids) = Fixture::new(vec![filled]);

        assert!(rub(&mut fx, &[(50.0, 50.0)]));

        assert_eq!(fx.doc.len(), 1);
        assert_eq!(fx.shape(ids[0]).fill(), None);
    }

    #[test]
    fn press_on_outline_removes_filled_rect() {
        let (mut fx, _) = Fixture::new(vec![filled_rect(0.0, 0.0, 100.0, 100.0)]);

        assert!(rub(&mut fx, &[(0.0, 50.0)]));

        assert!(fx.doc.is_empty());
    }

    #[test]
    fn unfilled_inside_is_untouched() {
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 100.0, 100.0)]);

        assert!(!rub(&mut fx, &[(50.0, 50.0), (60.0, 60.0)]));

        assert_eq!(fx.shape(ids[0]), rect(0.0, 0.0, 100.0, 100.0));
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn filled_cell_clears_only_that_cell() {
        // Arrange
        let (mut fx, ids) = Fixture::new(vec![filled_grid(&[(1, 1), (2, 1)])]);

        // Act: the centre of cell (1, 1), 12.5 from every line.
        rub(&mut fx, &[(37.5, 37.5)]);

        // Assert
        let g = fx.shape(ids[0]);
        assert_eq!(g.cell_fill(1, 1), None);
        assert_eq!(g.cell_fill(2, 1), Some(red()));
    }

    #[test]
    fn clear_drag_crosses_lines_without_removing() {
        // Arrange: row 0 filled in cells 0..3.
        let (mut fx, ids) = Fixture::new(vec![filled_grid(&[(0, 0), (1, 0), (2, 0), (3, 0)])]);

        // Act: start in cell 0, cross two lines, stop in cell 2.
        rub(&mut fx, &[(12.0, 12.0), (40.0, 12.0), (62.0, 12.0)]);

        // Assert
        assert_eq!(fx.doc.len(), 1, "the grid stays");
        let g = fx.shape(ids[0]);
        let left: Vec<(u32, u32)> = g.cell_fills().iter().map(|f| (f.col, f.row)).collect();
        assert_eq!(left, [(3, 0)]);
    }

    #[test]
    fn clear_drag_also_clears_shape_fills() {
        // Arrange: a filled cell and, to its right, a filled rectangle.
        let (mut fx, ids) = Fixture::new(vec![
            filled_grid(&[(3, 0)]),
            filled_rect(120.0, 0.0, 60.0, 30.0),
        ]);

        // Act: from the cell into the rectangle, across its outline.
        rub(&mut fx, &[(90.0, 12.0), (150.0, 12.0)]);

        // Assert: both fills gone, both shapes kept.
        assert_eq!(fx.doc.len(), 2);
        assert!(fx.shape(ids[0]).cell_fills().is_empty());
        assert_eq!(fx.shape(ids[1]).fill(), None);
    }

    #[test]
    fn remove_drag_into_filled_rect_removes_it() {
        let (mut fx, _) = Fixture::new(vec![filled_rect(100.0, 0.0, 100.0, 100.0)]);

        rub(&mut fx, &[(50.0, 50.0), (150.0, 50.0)]);

        assert!(fx.doc.is_empty(), "pre-T21 behaviour off a fill");
    }

    #[test]
    fn preview_shows_cleared_fill() {
        // Arrange
        let (mut fx, ids) = Fixture::new(vec![filled_rect(0.0, 0.0, 100.0, 100.0)]);
        let mut state = State::default();

        // Act
        send(&mut state, &mut fx, Phase::Down, 50.0, 50.0);
        let overlay = preview(&state, &fx.view(Tool::Eraser));

        // Assert: original hidden, an unfilled copy drawn, document intact.
        assert_eq!(overlay.hidden, vec![ids[0]]);
        assert_eq!(overlay.shapes, vec![fx.shape(ids[0]).with_fill(None)]);
        assert_eq!(fx.shape(ids[0]).fill(), Some(ColorId::INK));
    }

    #[test]
    fn erase_fill_is_one_undo_step() {
        // Arrange
        let before = filled_grid(&[(0, 0), (1, 0)]);
        let (mut fx, ids) = Fixture::new(vec![before.clone()]);

        // Act
        rub(&mut fx, &[(12.0, 12.0), (37.0, 12.0)]);
        assert_eq!(fx.history.undo_len(), 1);
        assert!(fx.history.undo(&mut fx.doc));

        // Assert
        assert_eq!(fx.shape(ids[0]), before);
    }
}
