//! Selection, marquee and move.
//!
//! - A press on a shape (topmost hit within [`HIT_TOLERANCE_PX`]) selects it
//!   unless it is already selected, then a drag moves the selection: the
//!   preview shows translated copies, the release commits one `Replace`
//!   transaction. With `Alt` held at the press the copies are inserted
//!   instead (duplicate-and-move) and become the selection.
//! - A press-release closer than [`CLICK_SLOP_PX`] is a click: it selects only
//!   the clicked shape; `Shift`-click toggles it instead.
//! - A press on empty space clears the selection (kept with `Shift`) and a
//!   drag draws a marquee; the release selects every shape whose bounds
//!   intersect it.
//!
//! Every entry point first drops selected ids missing from the document, so a
//! stale selection is never acted on.

use std::collections::HashSet;

use super::{Overlay, Phase, Pointer, ToolCtx, ToolView};
use crate::core::clipboard;
use crate::core::document::{Document, Edit, ShapeId, Transaction, tx_remove};
use crate::core::geom::{Aabb, Vec2};
use crate::core::shape::Shape;

/// Pointer travel (screen pixels) below which a press-release is a click.
pub const CLICK_SLOP_PX: f32 = 3.0;

/// Hit tolerance for picking shapes, in screen pixels.
pub const HIT_TOLERANCE_PX: f32 = 6.0;

/// Gesture state of this tool.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// The gesture in progress, if any.
    gesture: Option<Gesture>,
}

/// A select-tool gesture; positions are screen pixels.
#[derive(Debug, Clone, PartialEq)]
enum Gesture {
    /// Rubber-band selection started on empty space.
    Marquee {
        /// Press position.
        start: Vec2,
        /// Latest pointer position.
        current: Vec2,
        /// `Shift` was held: add to the selection instead of replacing it.
        additive: bool,
    },
    /// Dragging shapes started on a shape.
    Move {
        /// Press position.
        start: Vec2,
        /// Latest pointer position.
        current: Vec2,
        /// The shape under the press.
        clicked: ShapeId,
        /// Shapes being moved (the selection at press time).
        ids: Vec<ShapeId>,
        /// `Alt` was held: insert moved copies, keep the originals.
        duplicate: bool,
    },
}

/// Drops selected ids that are not in the document.
pub(crate) fn prune(selection: &mut Vec<ShapeId>, doc: &Document) {
    selection.retain(|id| doc.get(*id).is_some());
}

/// The shapes of `selection` with their ids, in z-order (bottom first);
/// unknown ids are skipped.
pub(crate) fn selected_shapes<'d>(
    doc: &'d Document,
    selection: &[ShapeId],
) -> Vec<(ShapeId, &'d Shape)> {
    let wanted: HashSet<ShapeId> = selection.iter().copied().collect();
    doc.shapes().filter(|(id, _)| wanted.contains(id)).collect()
}

/// Whether a press at `start` released at `end` (screen) is a click.
fn is_click(start: Vec2, end: Vec2) -> bool {
    start.distance(end) < CLICK_SLOP_PX
}

/// Clones of `shapes` translated by `delta`.
fn translated(shapes: &[(ShapeId, &Shape)], delta: Vec2) -> Vec<Shape> {
    shapes
        .iter()
        .map(|(_, shape)| {
            let mut shape = (*shape).clone();
            shape.translate(delta);
            shape
        })
        .collect()
}

/// Handles one pointer event of a gesture. Returns whether a redraw is needed.
pub fn on_pointer(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    match pointer.phase {
        Phase::Down => press(state, ctx, pointer),
        Phase::Move => drag_to(state, pointer.pos),
        Phase::Up => {
            drag_to(state, pointer.pos);
            release(state, ctx)
        }
    }
}

/// Starts a gesture: picks, toggles or clears, then arms a move or marquee.
fn press(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    prune(ctx.selection, ctx.doc);
    let before = ctx.selection.clone();
    let pos = pointer.pos;
    let world = ctx.camera.screen_to_world(pos);
    let tol = ctx.camera.world_len(HIT_TOLERANCE_PX);
    let hit = ctx.doc.topmost_where(|shape| shape.hit(world, tol));
    state.gesture = match hit {
        Some(id) if pointer.mods.shift => {
            match ctx.selection.iter().position(|selected| *selected == id) {
                Some(index) => {
                    ctx.selection.remove(index);
                }
                None => ctx.selection.push(id),
            }
            None
        }
        Some(id) => {
            if !ctx.selection.contains(&id) {
                *ctx.selection = vec![id];
            }
            Some(Gesture::Move {
                start: pos,
                current: pos,
                clicked: id,
                ids: ctx.selection.clone(),
                duplicate: pointer.mods.alt,
            })
        }
        None => {
            if !pointer.mods.shift {
                ctx.selection.clear();
            }
            Some(Gesture::Marquee {
                start: pos,
                current: pos,
                additive: pointer.mods.shift,
            })
        }
    };
    *ctx.selection != before
}

/// Moves the pointer of the gesture in progress. Returns whether it moved.
fn drag_to(state: &mut State, pos: Vec2) -> bool {
    match &mut state.gesture {
        Some(Gesture::Marquee { current, .. } | Gesture::Move { current, .. }) => {
            let moved = current.distance(pos) > 0.0;
            *current = pos;
            moved
        }
        None => false,
    }
}

/// Ends the gesture: applies the marquee, the click or the move.
fn release(state: &mut State, ctx: &mut ToolCtx<'_>) -> bool {
    let Some(gesture) = state.gesture.take() else {
        return false;
    };
    prune(ctx.selection, ctx.doc);
    let camera = ctx.camera;
    match gesture {
        Gesture::Marquee { start, current, .. } => {
            if is_click(start, current) {
                return start.distance(current) > 0.0;
            }
            let rect = Aabb::from_corners(
                camera.screen_to_world(start),
                camera.screen_to_world(current),
            );
            let hits: Vec<ShapeId> = ctx
                .doc
                .shapes()
                .filter(|(_, shape)| shape.bounds().intersects(&rect))
                .map(|(id, _)| id)
                .collect();
            for id in hits {
                if !ctx.selection.contains(&id) {
                    ctx.selection.push(id);
                }
            }
            true
        }
        Gesture::Move {
            start,
            current,
            clicked,
            ids,
            duplicate,
        } => {
            if is_click(start, current) {
                let changed = ctx.selection.as_slice() != [clicked];
                *ctx.selection = vec![clicked];
                prune(ctx.selection, ctx.doc);
                return changed || start.distance(current) > 0.0;
            }
            let delta = camera.screen_to_world(current) - camera.screen_to_world(start);
            let moving = selected_shapes(ctx.doc, &ids);
            if duplicate {
                let copies = moving.iter().map(|(_, shape)| (*shape).clone()).collect();
                clipboard::insert_copies(ctx, copies, delta);
            } else {
                let after = translated(&moving, delta);
                let edits: Vec<Edit> = moving
                    .iter()
                    .zip(after)
                    .map(|((id, before), after)| Edit::Replace {
                        id: *id,
                        before: (*before).clone(),
                        after,
                    })
                    .collect();
                ctx.commit(Transaction::from(edits));
            }
            // The preview disappears even if the edit was rejected.
            true
        }
    }
}

/// What the gesture in progress draws on top of the document.
#[must_use]
pub fn preview(state: &State, view: &ToolView<'_>) -> Overlay {
    let camera = view.camera;
    match &state.gesture {
        None => Overlay::default(),
        Some(Gesture::Marquee { start, current, .. }) => Overlay {
            marquee: Some(Aabb::from_corners(
                camera.screen_to_world(*start),
                camera.screen_to_world(*current),
            )),
            ..Overlay::default()
        },
        Some(Gesture::Move {
            start,
            current,
            ids,
            duplicate,
            ..
        }) => {
            if is_click(*start, *current) {
                return Overlay::default();
            }
            let delta = camera.screen_to_world(*current) - camera.screen_to_world(*start);
            let moving = selected_shapes(view.doc, ids);
            Overlay {
                shapes: translated(&moving, delta),
                hidden: if *duplicate {
                    Vec::new()
                } else {
                    moving.iter().map(|(id, _)| *id).collect()
                },
                ..Overlay::default()
            }
        }
    }
}

/// Discards the gesture in progress without changing the document. Returns
/// whether a redraw is needed.
pub fn cancel(state: &mut State) -> bool {
    state.gesture.take().is_some()
}

/// Selects every shape. Returns whether the selection changed.
pub fn select_all(ctx: &mut ToolCtx<'_>) -> bool {
    let all: Vec<ShapeId> = ctx.doc.shapes().map(|(id, _)| id).collect();
    let changed = *ctx.selection != all;
    *ctx.selection = all;
    changed
}

/// Deletes the selected shapes as one undo step and clears the selection.
/// Returns whether the document changed.
pub fn delete_selection(ctx: &mut ToolCtx<'_>) -> bool {
    prune(ctx.selection, ctx.doc);
    if ctx.selection.is_empty() {
        return false;
    }
    let tx = tx_remove(ctx.doc, ctx.selection);
    let changed = ctx.commit(tx);
    ctx.selection.clear();
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::Camera;
    use crate::core::clipboard::{self, testkit::*};
    use crate::core::command::Tool;
    use crate::core::document::ShapeId;
    use crate::core::geom::{Aabb, Vec2};
    use crate::core::input::Modifiers;
    use crate::core::shape::Shape;
    use crate::core::tools::Phase;

    fn send(
        state: &mut State,
        fx: &mut Fixture,
        phase: Phase,
        x: f32,
        y: f32,
        mods: Modifiers,
    ) -> bool {
        on_pointer(
            state,
            &mut fx.ctx(Tool::Select),
            ptr_with(phase, x, y, mods),
        )
    }

    fn click(state: &mut State, fx: &mut Fixture, x: f32, y: f32, mods: Modifiers) {
        send(state, fx, Phase::Down, x, y, mods);
        send(state, fx, Phase::Up, x, y, mods);
    }

    fn drag(
        state: &mut State,
        fx: &mut Fixture,
        from: (f32, f32),
        to: (f32, f32),
        mods: Modifiers,
    ) -> bool {
        send(state, fx, Phase::Down, from.0, from.1, mods);
        send(
            state,
            fx,
            Phase::Move,
            f32::midpoint(from.0, to.0),
            f32::midpoint(from.1, to.1),
            mods,
        );
        send(state, fx, Phase::Move, to.0, to.1, mods);
        send(state, fx, Phase::Up, to.0, to.1, mods)
    }

    fn corner(shape: &Shape) -> Vec2 {
        match shape {
            Shape::Rect { a, .. } => *a,
            other => panic!("expected a rect, got {other:?}"),
        }
    }

    /// Three filled 10×10 squares at x = 0, 50 and 200 (y = 0).
    fn squares() -> (Fixture, Vec<ShapeId>) {
        Fixture::new(vec![
            filled_rect(0.0, 0.0, 10.0, 10.0),
            filled_rect(50.0, 0.0, 10.0, 10.0),
            filled_rect(200.0, 0.0, 10.0, 10.0),
        ])
    }

    const NONE: Modifiers = Modifiers::NONE;

    #[test]
    fn click_selects_topmost_shape() {
        // Arrange: two overlapping filled squares.
        let (mut fx, ids) = Fixture::new(vec![
            filled_rect(0.0, 0.0, 10.0, 10.0),
            filled_rect(5.0, 5.0, 10.0, 10.0),
        ]);
        let mut state = State::default();

        // Act
        click(&mut state, &mut fx, 7.0, 7.0, NONE);

        // Assert
        assert_eq!(fx.selection, vec![ids[1]]);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn click_on_empty_clears_selection() {
        let (mut fx, ids) = squares();
        fx.selection.clone_from(&ids);
        let mut state = State::default();

        click(&mut state, &mut fx, 100.0, 100.0, NONE);

        assert!(fx.selection.is_empty());
    }

    #[test]
    fn click_on_selected_in_group_selects_only_it() {
        let (mut fx, ids) = squares();
        fx.selection.clone_from(&ids);
        let mut state = State::default();

        click(&mut state, &mut fx, 55.0, 5.0, NONE);

        assert_eq!(fx.selection, vec![ids[1]]);
        assert!(corner(&fx.shape(ids[1])).approx_eq(Vec2::new(50.0, 0.0), 1e-6));
    }

    #[test]
    fn click_shift_toggles_shape() {
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0]];
        let mut state = State::default();

        click(&mut state, &mut fx, 55.0, 5.0, SHIFT);
        assert_eq!(fx.selection, vec![ids[0], ids[1]]);

        click(&mut state, &mut fx, 5.0, 5.0, SHIFT);
        assert_eq!(fx.selection, vec![ids[1]]);
    }

    #[test]
    fn marquee_selects_intersecting_shapes() {
        let (mut fx, ids) = squares();
        let mut state = State::default();

        let changed = drag(&mut state, &mut fx, (-5.0, -5.0), (52.0, 3.0), NONE);

        assert!(changed);
        assert_eq!(fx.selection, vec![ids[0], ids[1]]);
        assert!(preview(&state, &fx.view(Tool::Select)).is_empty());
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn marquee_preview_shows_world_rect() {
        // Arrange: at zoom 2, screen (0,0)..(100,50) is world (0,0)..(50,25).
        let (mut fx, _) = squares();
        fx.camera = Camera::new(Vec2::ZERO, 2.0);
        let mut state = State::default();

        // Act
        send(&mut state, &mut fx, Phase::Down, 100.0, 50.0, NONE);
        send(&mut state, &mut fx, Phase::Move, 0.0, 0.0, NONE);

        // Assert
        let marquee = preview(&state, &fx.view(Tool::Select)).marquee;
        let expected = Aabb::from_corners(Vec2::ZERO, Vec2::new(50.0, 25.0));
        assert!(marquee.is_some_and(
            |m| m.min.approx_eq(expected.min, 1e-4) && m.max.approx_eq(expected.max, 1e-4)
        ));
    }

    #[test]
    fn marquee_with_shift_adds_to_selection() {
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[2]];
        let mut state = State::default();

        drag(&mut state, &mut fx, (-5.0, -5.0), (5.0, 5.0), SHIFT);

        assert_eq!(fx.selection, vec![ids[2], ids[0]]);
    }

    #[test]
    fn marquee_without_shift_replaces_selection() {
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[2]];
        let mut state = State::default();

        drag(&mut state, &mut fx, (-5.0, -5.0), (5.0, 5.0), NONE);

        assert_eq!(fx.selection, vec![ids[0]]);
    }

    #[test]
    fn move_drag_translates_selection() {
        // Arrange
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0], ids[1]];
        let mut state = State::default();

        // Act
        let changed = drag(&mut state, &mut fx, (5.0, 5.0), (25.0, 15.0), NONE);

        // Assert
        assert!(changed);
        assert!(corner(&fx.shape(ids[0])).approx_eq(Vec2::new(20.0, 10.0), 1e-4));
        assert!(corner(&fx.shape(ids[1])).approx_eq(Vec2::new(70.0, 10.0), 1e-4));
        assert!(corner(&fx.shape(ids[2])).approx_eq(Vec2::new(200.0, 0.0), 1e-6));
        assert_eq!(fx.selection, vec![ids[0], ids[1]]);
    }

    #[test]
    fn move_drag_uses_world_delta() {
        let (mut fx, ids) = squares();
        fx.camera = Camera::new(Vec2::ZERO, 2.0);
        fx.selection = vec![ids[0]];
        let mut state = State::default();

        drag(&mut state, &mut fx, (10.0, 10.0), (50.0, 30.0), NONE);

        assert!(corner(&fx.shape(ids[0])).approx_eq(Vec2::new(20.0, 10.0), 1e-4));
    }

    #[test]
    fn move_is_one_undo_step() {
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0], ids[1]];
        let mut state = State::default();

        drag(&mut state, &mut fx, (5.0, 5.0), (25.0, 15.0), NONE);

        assert_eq!(fx.history.undo_len(), 1);
        assert!(fx.history.undo(&mut fx.doc));
        assert!(corner(&fx.shape(ids[0])).approx_eq(Vec2::ZERO, 1e-6));
        assert!(corner(&fx.shape(ids[1])).approx_eq(Vec2::new(50.0, 0.0), 1e-6));
    }

    #[test]
    fn move_preview_shows_translated_and_hides_originals() {
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0]];
        let mut state = State::default();

        send(&mut state, &mut fx, Phase::Down, 5.0, 5.0, NONE);
        send(&mut state, &mut fx, Phase::Move, 15.0, 5.0, NONE);

        let overlay = preview(&state, &fx.view(Tool::Select));
        assert_eq!(overlay.hidden, vec![ids[0]]);
        assert_eq!(overlay.shapes.len(), 1);
        assert!(corner(&overlay.shapes[0]).approx_eq(Vec2::new(10.0, 0.0), 1e-4));
        assert!(corner(&fx.shape(ids[0])).approx_eq(Vec2::ZERO, 1e-6));
    }

    #[test]
    fn move_drag_on_unselected_shape_moves_it() {
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[2]];
        let mut state = State::default();

        drag(&mut state, &mut fx, (55.0, 5.0), (65.0, 5.0), NONE);

        assert_eq!(fx.selection, vec![ids[1]]);
        assert!(corner(&fx.shape(ids[1])).approx_eq(Vec2::new(60.0, 0.0), 1e-4));
        assert!(corner(&fx.shape(ids[2])).approx_eq(Vec2::new(200.0, 0.0), 1e-6));
    }

    #[test]
    fn move_cancel_keeps_document() {
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0]];
        let mut state = State::default();
        send(&mut state, &mut fx, Phase::Down, 5.0, 5.0, NONE);
        send(&mut state, &mut fx, Phase::Move, 25.0, 5.0, NONE);

        assert!(cancel(&mut state));

        assert!(preview(&state, &fx.view(Tool::Select)).is_empty());
        assert!(corner(&fx.shape(ids[0])).approx_eq(Vec2::ZERO, 1e-6));
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn alt_drag_duplicates() {
        // Arrange
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0]];
        let mut state = State::default();

        // Act
        let changed = drag(&mut state, &mut fx, (5.0, 5.0), (25.0, 5.0), ALT);

        // Assert
        assert!(changed);
        let all = fx.ids();
        assert_eq!(all.len(), 4);
        let copy_id = all[3];
        assert!(corner(&fx.shape(ids[0])).approx_eq(Vec2::ZERO, 1e-6));
        assert!(corner(&fx.shape(copy_id)).approx_eq(Vec2::new(20.0, 0.0), 1e-4));
        assert_eq!(fx.selection, vec![copy_id]);
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn alt_drag_preview_keeps_originals_visible() {
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0]];
        let mut state = State::default();

        send(&mut state, &mut fx, Phase::Down, 5.0, 5.0, ALT);
        send(&mut state, &mut fx, Phase::Move, 25.0, 5.0, ALT);

        let overlay = preview(&state, &fx.view(Tool::Select));
        assert!(overlay.hidden.is_empty());
        assert_eq!(overlay.shapes.len(), 1);
    }

    #[test]
    fn select_all_selects_every_shape() {
        let (mut fx, ids) = squares();

        assert!(select_all(&mut fx.ctx(Tool::Select)));

        assert_eq!(fx.selection, ids);
    }

    #[test]
    fn select_all_twice_reports_no_change() {
        let (mut fx, _) = squares();
        select_all(&mut fx.ctx(Tool::Select));

        assert!(!select_all(&mut fx.ctx(Tool::Select)));
    }

    #[test]
    fn delete_removes_selection_as_one_step() {
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[2], ids[0]];

        assert!(delete_selection(&mut fx.ctx(Tool::Select)));

        assert_eq!(fx.ids(), vec![ids[1]]);
        assert!(fx.selection.is_empty());
        assert_eq!(fx.history.undo_len(), 1);
        assert!(fx.history.undo(&mut fx.doc));
        assert_eq!(fx.ids(), ids);
    }

    #[test]
    fn delete_with_empty_selection_is_noop() {
        let (mut fx, _) = squares();

        assert!(!delete_selection(&mut fx.ctx(Tool::Select)));

        assert_eq!(fx.doc.len(), 3);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn selection_pruned_after_undo() {
        // Arrange: duplicate selects the copy; undoing it leaves a stale id.
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0]];
        clipboard::duplicate(&mut fx.ctx(Tool::Select));
        assert!(fx.history.undo(&mut fx.doc));
        let stale_ids = fx.selection.clone();
        assert!(stale_ids.iter().all(|id| fx.doc.get(*id).is_none()));

        // Act: every entry point drops stale ids before acting.
        let copied = clipboard::copy(&mut fx.ctx(Tool::Select));
        let deleted = delete_selection(&mut fx.ctx(Tool::Select));

        // Assert
        assert!(!copied && !deleted);
        assert!(fx.clipboard.shapes().is_empty());
        assert!(fx.selection.iter().all(|id| fx.doc.get(*id).is_some()));
        assert_eq!(fx.doc.len(), 3);

        // A drag on empty space after a stale selection selects nothing stale.
        fx.selection = stale_ids;
        let mut state = State::default();
        drag(&mut state, &mut fx, (100.0, 100.0), (110.0, 110.0), SHIFT);
        assert!(fx.selection.iter().all(|id| fx.doc.get(*id).is_some()));
    }
}
