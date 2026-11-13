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
//! - A press on a corner handle of the selection, without `Alt`, scales it
//!   about the opposite corner: the preview shows scaled copies, the release
//!   commits one `Replace` transaction; `Shift` keeps the proportions
//!   (ADR-T91-1).
//! - [`align_selection`] and [`distribute_selection`] translate the
//!   selected shapes by their bounds as one `Replace` transaction
//!   (ADR-T101-1).
//!
//! Every entry point first drops selected ids missing from the document, so a
//! stale selection is never acted on.

use std::collections::HashSet;

use super::{Overlay, Phase, Pointer, ToolCtx, ToolView};
use crate::core::camera::Camera;
use crate::core::clipboard;
use crate::core::command::{Axis, Side};
use crate::core::document::{Document, Edit, ShapeId, Transaction, tx_remove};
use crate::core::geom::{Aabb, Vec2, approx_eq};
use crate::core::shape::Shape;
use crate::core::snapshot::{Reader, SnapshotError, Writer, check_unlocked_ids};
use crate::machine::select::{MIN_FACTOR, ResizeFrame, Select};

/// Pointer travel (screen pixels) below which a press-release is a click.
pub const CLICK_SLOP_PX: f32 = 3.0;

/// Hit tolerance for picking shapes, in screen pixels.
pub const HIT_TOLERANCE_PX: f32 = 6.0;

/// Gap in pixels between the selected shapes' bounds and the selection
/// outline, whose corners carry the handles.
pub const SELECTION_PAD_PX: f32 = 4.0;

/// Side of a resize handle in screen pixels, independent of zoom (ADR-T91-1).
pub const HANDLE_SIZE_PX: f32 = 10.0;

/// Factor of [`Command::ScaleUp`](crate::core::command::Command::ScaleUp).
pub const SCALE_UP_FACTOR: f32 = 1.1;

/// Factor of [`Command::ScaleDown`](crate::core::command::Command::ScaleDown).
pub const SCALE_DOWN_FACTOR: f32 = 0.9;

/// How close to 1 a factor counts as no change.
const IDENTITY_EPS: f32 = 1e-6;

/// Gesture state of this tool.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    /// The gesture in progress ([`Select::Idle`] when none).
    gesture: Select,
}

/// The corners of `b`, clockwise from the top-left one.
fn corners(b: Aabb) -> [Vec2; 4] {
    [
        b.min,
        Vec2::new(b.max.x, b.min.y),
        b.max,
        Vec2::new(b.min.x, b.max.y),
    ]
}

/// The joint geometry box of `selection` (see [`Shape::geometry_bounds`]);
/// unknown ids are skipped.
fn selection_geometry(doc: &Document, selection: &[ShapeId]) -> Option<Aabb> {
    selection
        .iter()
        .filter_map(|id| doc.get(*id))
        .map(Shape::geometry_bounds)
        .reduce(|a, b| a.union(&b))
}

/// The smallest factor per axis that every shape of `selection` allows.
fn selection_floor(doc: &Document, selection: &[ShapeId]) -> Vec2 {
    selection
        .iter()
        .filter_map(|id| doc.get(*id))
        .map(Shape::min_scale)
        .fold(Vec2::ZERO, |a, b| Vec2::new(a.x.max(b.x), a.y.max(b.y)))
}

/// Screen centres of the four handles of `selection`, clockwise from the
/// top-left corner, or `None` when it has nothing to scale (ADR-T91-1).
#[must_use]
pub fn handle_centers(doc: &Document, selection: &[ShapeId], camera: &Camera) -> Option<[Vec2; 4]> {
    let geometry = selection_geometry(doc, selection)?;
    if geometry.width() <= 0.0 && geometry.height() <= 0.0 {
        return None;
    }
    let bounds = selection
        .iter()
        .filter_map(|id| doc.get(*id))
        .map(Shape::bounds)
        .reduce(|a, b| a.union(&b))?;
    let outline = Aabb::from_corners(
        camera.world_to_screen(bounds.min),
        camera.world_to_screen(bounds.max),
    )
    .expand(SELECTION_PAD_PX);
    let centers = corners(outline);
    centers.iter().all(|c| c.is_finite()).then_some(centers)
}

/// The handle of `selection` under the screen position `pos`: the nearest
/// one whose square holds it.
fn handle_at(doc: &Document, selection: &[ShapeId], camera: &Camera, pos: Vec2) -> Option<usize> {
    let reach = HANDLE_SIZE_PX * 0.5;
    handle_centers(doc, selection, camera)?
        .iter()
        .enumerate()
        .filter(|(_, c)| (c.x - pos.x).abs() <= reach && (c.y - pos.y).abs() <= reach)
        .min_by(|(_, a), (_, b)| a.distance(pos).total_cmp(&b.distance(pos)))
        .map(|(index, _)| index)
}

/// The frame for a drag of handle `index` of `selection`.
fn resize_frame(doc: &Document, selection: &[ShapeId], index: usize) -> Option<ResizeFrame> {
    let box_corners = corners(selection_geometry(doc, selection)?);
    Some(ResizeFrame {
        pivot: *box_corners.get((index + 2) % box_corners.len())?,
        corner: *box_corners.get(index)?,
        floor: selection_floor(doc, selection),
    })
}

/// Copies of `shapes` scaled about `pivot`; `None` if any cannot be
/// (the whole operation is then ignored).
fn scaled(shapes: &[(ShapeId, &Shape)], pivot: Vec2, sx: f32, sy: f32) -> Option<Vec<Shape>> {
    shapes
        .iter()
        .map(|(_, shape)| shape.scaled_about(pivot, sx, sy))
        .collect()
}

/// `Replace` edits turning `before` into `after`, pairwise.
fn replace_edits(before: &[(ShapeId, &Shape)], after: Vec<Shape>) -> Transaction {
    let edits: Vec<Edit> = before
        .iter()
        .zip(after)
        .map(|((id, before), after)| Edit::Replace {
            id: *id,
            before: (*before).clone(),
            after,
        })
        .collect();
    Transaction::from(edits)
}

/// Scales `ids` about `pivot` as one undo step; factors of 1 record
/// nothing. Returns whether the document changed.
fn commit_scale(ctx: &mut ToolCtx<'_>, ids: &[ShapeId], pivot: Vec2, sx: f32, sy: f32) -> bool {
    if approx_eq(sx, 1.0, IDENTITY_EPS) && approx_eq(sy, 1.0, IDENTITY_EPS) {
        return false;
    }
    let before = selected_shapes(ctx.doc, ids);
    let Some(after) = scaled(&before, pivot, sx, sy) else {
        return false;
    };
    let tx = replace_edits(&before, after);
    ctx.commit(tx)
}

/// Scales the selection by `factor` about its centre, within its limits.
fn scale_selection(ctx: &mut ToolCtx<'_>, factor: f32) -> bool {
    prune(ctx.selection, ctx.doc);
    let Some(geometry) = selection_geometry(ctx.doc, ctx.selection) else {
        return false;
    };
    let floor = selection_floor(ctx.doc, ctx.selection);
    let s = factor.max(floor.x.max(floor.y)).max(MIN_FACTOR);
    let ids = ctx.selection.clone();
    commit_scale(ctx, &ids, geometry.center(), s, s)
}

/// Grows the selection by 10 % about its centre as one undo step. Returns
/// whether the document changed.
pub fn scale_selection_up(ctx: &mut ToolCtx<'_>) -> bool {
    scale_selection(ctx, SCALE_UP_FACTOR)
}

/// Shrinks the selection by 10 % about its centre as one undo step, down to
/// its limits. Returns whether the document changed.
pub fn scale_selection_down(ctx: &mut ToolCtx<'_>) -> bool {
    scale_selection(ctx, SCALE_DOWN_FACTOR)
}

/// Moves each selected shape so its bounds touch `side` of the selection's
/// box, as one undo step (ADR-T101-1). Needs two or more selected shapes;
/// records nothing when no shape would move or a moved shape would not be
/// finite. Returns whether the document changed.
pub fn align_selection(ctx: &mut ToolCtx<'_>, side: Side) -> bool {
    prune(ctx.selection, ctx.doc);
    let boxes = selected_bounds(ctx.doc, ctx.selection);
    if boxes.len() < 2 {
        return false;
    }
    let Some(frame) = boxes.iter().map(|(_, b)| *b).reduce(|a, b| a.union(&b)) else {
        return false;
    };
    let moves: Vec<(ShapeId, Vec2)> = boxes
        .iter()
        .map(|&(id, b)| {
            let offset = match side {
                Side::Left => Vec2::new(frame.min.x - b.min.x, 0.0),
                Side::Right => Vec2::new(frame.max.x - b.max.x, 0.0),
                Side::Top => Vec2::new(0.0, frame.min.y - b.min.y),
                Side::Bottom => Vec2::new(0.0, frame.max.y - b.max.y),
            };
            (id, offset)
        })
        .collect();
    commit_moves(ctx, &moves, frame)
}

/// Lays two or more selected shapes out as a row (`Horizontal`) or column
/// (`Vertical`) without overlap, as one undo step (ADR-T101-2). Sorted by
/// near edge, the first shape stays put and the others are centred on its
/// line across `axis`; neighbours get equal gaps of a quarter of the
/// largest size along `axis` (ADR-T101-3). Records nothing when no shape
/// would move or a moved shape would not be finite. Returns whether the
/// document changed.
pub fn distribute_selection(ctx: &mut ToolCtx<'_>, axis: Axis) -> bool {
    prune(ctx.selection, ctx.doc);
    let mut order = selected_bounds(ctx.doc, ctx.selection);
    if order.len() < 2 {
        return false;
    }
    let Some(frame) = order.iter().map(|(_, b)| *b).reduce(|a, b| a.union(&b)) else {
        return false;
    };
    let near = |b: &Aabb| match axis {
        Axis::Horizontal => b.min.x,
        Axis::Vertical => b.min.y,
    };
    let far = |b: &Aabb| match axis {
        Axis::Horizontal => b.max.x,
        Axis::Vertical => b.max.y,
    };
    let across = |b: &Aabb| match axis {
        Axis::Horizontal => b.center().y,
        Axis::Vertical => b.center().x,
    };
    let offset = |along: f32, across: f32| match axis {
        Axis::Horizontal => Vec2::new(along, across),
        Axis::Vertical => Vec2::new(across, along),
    };
    // Near edge, then far edge, then stacking order (the sort is stable).
    order.sort_by(|(_, p), (_, q)| near(p).total_cmp(&near(q)).then(far(p).total_cmp(&far(q))));
    let Some(&(_, first_box)) = order.first() else {
        return false;
    };
    let size = |b: &Aabb| far(b) - near(b);
    let gap = order.iter().map(|(_, b)| size(b)).fold(0.0, f32::max) * GAP_RATIO;
    let line = across(&first_box);
    let mut cursor = far(&first_box) + gap;
    let moves: Vec<(ShapeId, Vec2)> = order
        .iter()
        .skip(1)
        .map(|(id, b)| {
            let shift = offset(cursor - near(b), line - across(b));
            cursor += size(b) + gap;
            (*id, shift)
        })
        .collect();
    commit_moves(ctx, &moves, frame)
}

/// Gap of a row or column, as a share of the largest selected size along
/// its axis (ADR-T101-3).
const GAP_RATIO: f32 = 0.25;

/// Relative tolerance under which a whole align or distribute is a no-op
/// (ADR-T101-1).
const ARRANGE_EPS: f32 = 1e-5;

/// The selected shapes' ids and bounds, in stacking order.
fn selected_bounds(doc: &Document, selection: &[ShapeId]) -> Vec<(ShapeId, Aabb)> {
    selected_shapes(doc, selection)
        .into_iter()
        .map(|(id, shape)| (id, shape.bounds()))
        .collect()
}

/// Translates each shape of `moves` by its offset as one undo step
/// (ADR-T101-1). Records nothing when every offset is within
/// [`ARRANGE_EPS`] of `frame`'s magnitude, or when an offset or a moved
/// shape is not finite. Returns whether the document changed.
fn commit_moves(ctx: &mut ToolCtx<'_>, moves: &[(ShapeId, Vec2)], frame: Aabb) -> bool {
    let magnitude = [frame.min.x, frame.min.y, frame.max.x, frame.max.y]
        .into_iter()
        .fold(1.0f32, |m, c| m.max(c.abs()));
    let tolerance = ARRANGE_EPS * magnitude;
    if !tolerance.is_finite() {
        return false;
    }
    let negligible = |d: Vec2| d.x.abs() <= tolerance && d.y.abs() <= tolerance;
    if moves.iter().all(|&(_, offset)| negligible(offset)) {
        return false;
    }
    let mut edits = Vec::with_capacity(moves.len());
    for &(id, offset) in moves {
        if !offset.is_finite() {
            return false;
        }
        if offset == Vec2::ZERO {
            continue;
        }
        let Some(before) = ctx.doc.get(id) else {
            continue;
        };
        let mut after = before.clone();
        after.translate(offset);
        let bounds = after.bounds();
        if !after.is_finite() || !bounds.min.is_finite() || !bounds.max.is_finite() {
            return false;
        }
        if after != *before {
            edits.push(Edit::Replace {
                id,
                before: before.clone(),
                after,
            });
        }
    }
    ctx.commit(Transaction::from(edits))
}

/// Drops selected ids that are not in the document or are locked
/// (ADR-T107-1).
pub(crate) fn prune(selection: &mut Vec<ShapeId>, doc: &Document) {
    selection.retain(|id| doc.get(*id).is_some() && !doc.is_locked(*id));
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
        Phase::Move => drag_to(state, pointer.pos, pointer.mods.shift),
        Phase::Up => {
            drag_to(state, pointer.pos, pointer.mods.shift);
            release(state, ctx)
        }
    }
}

/// Starts a gesture: takes a handle, or picks, toggles or clears and arms a
/// move or marquee.
fn press(state: &mut State, ctx: &mut ToolCtx<'_>, pointer: Pointer) -> bool {
    prune(ctx.selection, ctx.doc);
    let before = ctx.selection.clone();
    let pos = pointer.pos;
    // ADR-T91-1: a handle wins only on its own square, and never with Alt,
    // which duplicates.
    if !pointer.mods.alt
        && let Some(frame) = handle_at(ctx.doc, ctx.selection, ctx.camera, pos)
            .and_then(|index| resize_frame(ctx.doc, ctx.selection, index))
    {
        state.gesture = Select::Resize {
            start: pos,
            current: pos,
            proportional: pointer.mods.shift,
            ids: ctx.selection.clone(),
            frame,
        };
        return *ctx.selection != before;
    }
    let world = ctx.camera.screen_to_world(pos);
    let tol = ctx.camera.world_len(HIT_TOLERANCE_PX);
    // The inside of the only selected shape grabs it when it can host
    // text, so a second click there can open its text (ADR-T31-1,
    // ADR-T52-1). Locked shapes are never picked (ADR-T107-1).
    let hit = ctx
        .doc
        .topmost_unlocked_where(|shape| shape.hit(world, tol))
        .or_else(|| match ctx.selection.as_slice() {
            &[id] => ctx
                .doc
                .get(id)
                .and_then(|shape| shape.text_host_at(world))
                .map(|_| id),
            _ => None,
        });
    state.gesture = match hit {
        Some(id) if pointer.mods.shift => {
            match ctx.selection.iter().position(|selected| *selected == id) {
                Some(index) => {
                    ctx.selection.remove(index);
                }
                None => ctx.selection.push(id),
            }
            Select::Idle
        }
        Some(id) => {
            if !ctx.selection.contains(&id) {
                *ctx.selection = vec![id];
            }
            Select::Move {
                start: pos,
                current: pos,
                clicked: id,
                ids: ctx.selection.clone(),
                duplicate: pointer.mods.alt,
            }
        }
        None => {
            if !pointer.mods.shift {
                ctx.selection.clear();
            }
            Select::Marquee {
                start: pos,
                current: pos,
                additive: pointer.mods.shift,
            }
        }
    };
    *ctx.selection != before
}

/// Moves the pointer of the gesture in progress, with `shift` as now held.
/// Returns whether anything it draws changed.
fn drag_to(state: &mut State, pos: Vec2, shift: bool) -> bool {
    state.gesture.drag(pos, shift)
}

/// Ends the gesture: applies the marquee, the click or the move.
fn release(state: &mut State, ctx: &mut ToolCtx<'_>) -> bool {
    let gesture = state.gesture.take();
    if gesture == Select::Idle {
        return false;
    }
    prune(ctx.selection, ctx.doc);
    let camera = ctx.camera;
    match gesture {
        Select::Idle => false,
        Select::Marquee { start, current, .. } => {
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
                .filter(|(id, shape)| !ctx.doc.is_locked(*id) && shape.bounds().intersects(&rect))
                .map(|(id, _)| id)
                .collect();
            for id in hits {
                if !ctx.selection.contains(&id) {
                    ctx.selection.push(id);
                }
            }
            true
        }
        Select::Move {
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
                ctx.commit(replace_edits(&moving, after));
            }
            // The preview disappears even if the edit was rejected.
            true
        }
        Select::Resize {
            start,
            current,
            proportional,
            ids,
            frame,
        } => {
            if is_click(start, current) {
                return start.distance(current) > 0.0;
            }
            let delta = camera.screen_to_world(current) - camera.screen_to_world(start);
            if let Some((sx, sy)) = frame.factors(delta, proportional) {
                commit_scale(ctx, &ids, frame.pivot, sx, sy);
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
        Select::Idle => Overlay::default(),
        Select::Marquee { start, current, .. } => Overlay {
            marquee: Some(Aabb::from_corners(
                camera.screen_to_world(*start),
                camera.screen_to_world(*current),
            )),
            ..Overlay::default()
        },
        Select::Move {
            start,
            current,
            ids,
            duplicate,
            ..
        } => {
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
        Select::Resize {
            start,
            current,
            proportional,
            ids,
            frame,
        } => {
            if is_click(*start, *current) {
                return Overlay::default();
            }
            let delta = camera.screen_to_world(*current) - camera.screen_to_world(*start);
            let moving = selected_shapes(view.doc, ids);
            let after = frame
                .factors(delta, *proportional)
                .and_then(|(sx, sy)| scaled(&moving, frame.pivot, sx, sy));
            after.map_or_else(Overlay::default, |shapes| Overlay {
                shapes,
                hidden: moving.iter().map(|(id, _)| *id).collect(),
                ..Overlay::default()
            })
        }
    }
}

/// Discards the gesture in progress without changing the document. Returns
/// whether a redraw is needed.
pub fn cancel(state: &mut State) -> bool {
    state.gesture.cancel()
}

/// Selects every unlocked shape (ADR-T107-1). Returns whether the
/// selection changed.
pub fn select_all(ctx: &mut ToolCtx<'_>) -> bool {
    let all: Vec<ShapeId> = ctx
        .doc
        .shapes()
        .map(|(id, _)| id)
        .filter(|id| !ctx.doc.is_locked(*id))
        .collect();
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

impl State {
    /// `select -`, `select marquee <start> <current> <additive>`,
    /// `select move <start> <current> <clicked> <duplicate> <ids>` or
    /// `select resize <start> <current> <proportional> <pivot> <corner>
    /// <floor> <ids>` (snapshots, ADR-T74-1).
    pub(crate) fn write_snapshot(&self, w: &mut Writer) {
        w.record("select", |l| match &self.gesture {
            Select::Idle => {
                l.none();
            }
            Select::Marquee {
                start,
                current,
                additive,
            } => {
                l.word("marquee").put(start).put(current).bool(*additive);
            }
            Select::Move {
                start,
                current,
                clicked,
                ids,
                duplicate,
            } => {
                l.word("move")
                    .put(start)
                    .put(current)
                    .put(clicked)
                    .bool(*duplicate)
                    .put(ids);
            }
            Select::Resize {
                start,
                current,
                proportional,
                ids,
                frame,
            } => {
                l.word("resize")
                    .put(start)
                    .put(current)
                    .bool(*proportional)
                    .put(&frame.pivot)
                    .put(&frame.corner)
                    .put(&frame.floor)
                    .put(ids);
            }
        });
    }

    /// Reads what [`State::write_snapshot`] wrote: moved ids in `doc`.
    pub(crate) fn read_snapshot(r: &mut Reader<'_>, doc: &Document) -> Result<Self, SnapshotError> {
        let mut t = r.record("select")?;
        let gesture = if t.none() {
            Select::Idle
        } else {
            match t.word("select gesture")? {
                "marquee" => Select::Marquee {
                    start: t.loose_vec2()?,
                    current: t.loose_vec2()?,
                    additive: t.bool("additive")?,
                },
                "move" => {
                    let start = t.loose_vec2()?;
                    let current = t.loose_vec2()?;
                    let clicked: ShapeId = t.get()?;
                    let duplicate = t.bool("duplicate")?;
                    let ids: Vec<ShapeId> = t.get()?;
                    check_unlocked_ids(&t, doc, &[clicked])?;
                    check_unlocked_ids(&t, doc, &ids)?;
                    Select::Move {
                        start,
                        current,
                        clicked,
                        ids,
                        duplicate,
                    }
                }
                "resize" => {
                    let start = t.loose_vec2()?;
                    let current = t.loose_vec2()?;
                    let proportional = t.bool("proportional")?;
                    let frame = ResizeFrame {
                        pivot: t.get()?,
                        corner: t.get()?,
                        floor: t.get()?,
                    };
                    let ids: Vec<ShapeId> = t.get()?;
                    check_unlocked_ids(&t, doc, &ids)?;
                    Select::Resize {
                        start,
                        current,
                        proportional,
                        ids,
                        frame,
                    }
                }
                other => return Err(t.error(format!("unknown select gesture `{other}`"))),
            }
        };
        t.end()?;
        Ok(Self { gesture })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::camera::Camera;
    use crate::core::clipboard::{self, testkit::*};
    use crate::core::command::Tool;
    use crate::core::document::ShapeId;
    use crate::core::geom::{Aabb, Vec2, approx_eq};
    use crate::core::input::Modifiers;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
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

        assert_eq!(fx.selection, []);
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
        assert_eq!(overlay.hidden, []);
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
        assert_eq!(fx.selection, []);
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
        assert_eq!(fx.clipboard.shapes(), []);
        assert!(fx.selection.iter().all(|id| fx.doc.get(*id).is_some()));
        assert_eq!(fx.doc.len(), 3);

        // A drag on empty space after a stale selection selects nothing stale.
        fx.selection = stale_ids;
        let mut state = State::default();
        drag(&mut state, &mut fx, (100.0, 100.0), (110.0, 110.0), SHIFT);
        assert!(fx.selection.iter().all(|id| fx.doc.get(*id).is_some()));
    }

    // ---- T107 locks ---------------------------------------------------------

    /// Locks `ids` in the fixture's document (not in the history).
    fn lock(fx: &mut Fixture, ids: &[ShapeId]) {
        let tx = crate::core::document::tx_lock(&fx.doc, ids);
        assert!(fx.doc.apply(&tx).is_ok());
    }

    #[test]
    fn click_skips_locked_shape_and_picks_the_one_below() {
        // Arrange: two overlapping filled squares, the top one locked.
        let (mut fx, ids) = Fixture::new(vec![
            filled_rect(0.0, 0.0, 10.0, 10.0),
            filled_rect(5.0, 5.0, 30.0, 30.0),
        ]);
        lock(&mut fx, &[ids[1]]);
        let mut state = State::default();

        // Act: on both, then on the locked one only.
        click(&mut state, &mut fx, 7.0, 7.0, NONE);
        let below = fx.selection.clone();
        click(&mut state, &mut fx, 25.0, 25.0, NONE);

        // Assert
        assert_eq!(below, vec![ids[0]]);
        assert_eq!(fx.selection, []);
    }

    #[test]
    fn drag_on_locked_shape_moves_nothing() {
        // Arrange
        let (mut fx, ids) = squares();
        lock(&mut fx, &[ids[1]]);
        let before = fx.shape(ids[1]);
        let mut state = State::default();

        // Act: a drag starting on the locked square is a marquee.
        drag(&mut state, &mut fx, (55.0, 5.0), (95.0, 45.0), NONE);

        // Assert
        assert_eq!(fx.shape(ids[1]), before);
        assert_eq!(fx.selection, []);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn marquee_skips_locked_shapes() {
        // Arrange
        let (mut fx, ids) = squares();
        lock(&mut fx, &[ids[0]]);
        let mut state = State::default();

        // Act
        drag(&mut state, &mut fx, (-5.0, -5.0), (205.0, 3.0), NONE);

        // Assert
        assert_eq!(fx.selection, vec![ids[1], ids[2]]);
    }

    #[test]
    fn select_all_skips_locked_shapes() {
        // Arrange
        let (mut fx, ids) = squares();
        lock(&mut fx, &[ids[1]]);

        // Act
        let changed = select_all(&mut fx.ctx(Tool::Select));

        // Assert
        assert!(changed);
        assert_eq!(fx.selection, vec![ids[0], ids[2]]);
    }

    #[test]
    fn prune_drops_locked_ids() {
        // Arrange
        let (mut fx, ids) = squares();
        lock(&mut fx, &[ids[2]]);
        let mut selection = ids.clone();

        // Act
        prune(&mut selection, &fx.doc);

        // Assert
        assert_eq!(selection, vec![ids[0], ids[1]]);
    }

    // ---- T31 shape text ---------------------------------------------------

    #[test]
    fn shape_text_press_inside_only_selected_rect_keeps_it() {
        // Arrange: an unfilled 100 × 100 rectangle, selected.
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 100.0, 100.0)]);
        fx.selection = vec![ids[0]];
        let mut state = State::default();

        // Act: click in the middle, away from the outline.
        click(&mut state, &mut fx, 50.0, 50.0, Modifiers::NONE);

        // Assert: still selected; a drag from inside moves it.
        assert_eq!(fx.selection, ids);
        drag(
            &mut state,
            &mut fx,
            (50.0, 50.0),
            (60.0, 55.0),
            Modifiers::NONE,
        );
        assert_eq!(corner(&fx.shape(ids[0])), Vec2::new(10.0, 5.0));
    }

    #[test]
    fn press_inside_only_selected_numbered_rect_clears_selection() {
        // Arrange: T52, a numbered rectangle holds no text, so its inside
        // is not a handle.
        let numbered = rect(0.0, 0.0, 100.0, 100.0).with_label(Some(1));
        let (mut fx, ids) = Fixture::new(vec![numbered]);
        fx.selection = vec![ids[0]];
        let mut state = State::default();

        // Act
        click(&mut state, &mut fx, 50.0, 50.0, Modifiers::NONE);

        // Assert
        assert_eq!(fx.selection, []);
    }

    #[test]
    fn shape_text_press_inside_unselected_rect_does_not_pick_it() {
        let (mut fx, _) = Fixture::new(vec![rect(0.0, 0.0, 100.0, 100.0)]);
        let mut state = State::default();

        click(&mut state, &mut fx, 50.0, 50.0, Modifiers::NONE);

        assert_eq!(fx.selection, []);
    }

    // ---- T91 resize handles -----------------------------------------------

    /// Indices into [`handle_centers`].
    const TOP_LEFT: usize = 0;
    const BOTTOM_RIGHT: usize = 2;

    /// Screen centre of handle `index` of the current selection.
    fn handle(fx: &Fixture, index: usize) -> (f32, f32) {
        match handle_centers(&fx.doc, &fx.selection, &fx.camera) {
            Some(centers) => (centers[index].x, centers[index].y),
            None => panic!("the selection has no handles"),
        }
    }

    /// Drags handle `index` by the screen delta `(dx, dy)`.
    fn drag_handle(
        state: &mut State,
        fx: &mut Fixture,
        index: usize,
        (dx, dy): (f32, f32),
        mods: Modifiers,
    ) -> bool {
        let from = handle(fx, index);
        drag(state, fx, from, (from.0 + dx, from.1 + dy), mods)
    }

    /// The `(a, b)` corners of a box shape.
    fn box_of(shape: &Shape) -> (Vec2, Vec2) {
        match shape {
            Shape::Rect { a, b, .. } | Shape::Ellipse { a, b, .. } | Shape::Grid { a, b, .. } => {
                (*a, *b)
            }
            other => panic!("expected a box, got {other:?}"),
        }
    }

    fn assert_box(shape: &Shape, a: (f32, f32), b: (f32, f32)) {
        let (got_a, got_b) = box_of(shape);
        assert!(
            got_a.approx_eq(Vec2::new(a.0, a.1), 1e-3)
                && got_b.approx_eq(Vec2::new(b.0, b.1), 1e-3),
            "got {got_a:?} {got_b:?}, want {a:?} {b:?}"
        );
    }

    /// One selected 100 × 100 filled square at the origin.
    fn selected_square() -> (Fixture, Vec<ShapeId>) {
        let (mut fx, ids) = Fixture::new(vec![filled_rect(0.0, 0.0, 100.0, 100.0)]);
        fx.selection = vec![ids[0]];
        (fx, ids)
    }

    #[test]
    fn resize_drag_scales_about_opposite_corner() {
        // Arrange
        let (mut fx, ids) = selected_square();
        let mut state = State::default();

        // Act: bottom-right by (50, 100), then top-left of the result.
        let changed = drag_handle(
            &mut state,
            &mut fx,
            BOTTOM_RIGHT,
            (50.0, 100.0),
            Modifiers::NONE,
        );

        // Assert
        assert!(changed);
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (150.0, 200.0));
        assert_eq!(fx.selection, vec![ids[0]]);

        drag_handle(
            &mut state,
            &mut fx,
            TOP_LEFT,
            (75.0, 100.0),
            Modifiers::NONE,
        );
        assert_box(&fx.shape(ids[0]), (75.0, 100.0), (150.0, 200.0));
    }

    #[test]
    fn resize_uses_world_delta() {
        // Arrange: at zoom 2, 40 screen pixels are 20 world units.
        let (mut fx, ids) = selected_square();
        fx.camera = Camera::new(Vec2::ZERO, 2.0);
        let mut state = State::default();

        // Act
        drag_handle(
            &mut state,
            &mut fx,
            BOTTOM_RIGHT,
            (40.0, 0.0),
            Modifiers::NONE,
        );

        // Assert
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (120.0, 100.0));
    }

    #[test]
    fn resize_is_one_undo_step() {
        // Arrange: two shapes selected.
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0], ids[1]];
        let mut state = State::default();

        // Act
        drag_handle(
            &mut state,
            &mut fx,
            BOTTOM_RIGHT,
            (60.0, 10.0),
            Modifiers::NONE,
        );

        // Assert
        assert_eq!(fx.history.undo_len(), 1);
        assert!(fx.history.undo(&mut fx.doc));
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (10.0, 10.0));
        assert_box(&fx.shape(ids[1]), (50.0, 0.0), (60.0, 10.0));
    }

    #[test]
    fn resize_preview_shows_scaled_and_hides_originals() {
        // Arrange
        let (mut fx, ids) = selected_square();
        let mut state = State::default();
        let (x, y) = handle(&fx, BOTTOM_RIGHT);

        // Act
        send(&mut state, &mut fx, Phase::Down, x, y, NONE);
        send(&mut state, &mut fx, Phase::Move, x + 100.0, y, NONE);

        // Assert: the document only changes on release.
        let overlay = preview(&state, &fx.view(Tool::Select));
        assert_eq!(overlay.hidden, vec![ids[0]]);
        assert_eq!(overlay.shapes.len(), 1);
        assert_box(&overlay.shapes[0], (0.0, 0.0), (200.0, 100.0));
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (100.0, 100.0));
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn resize_cancel_keeps_document() {
        // Arrange
        let (mut fx, ids) = selected_square();
        let mut state = State::default();
        let (x, y) = handle(&fx, BOTTOM_RIGHT);
        send(&mut state, &mut fx, Phase::Down, x, y, NONE);
        send(&mut state, &mut fx, Phase::Move, x + 50.0, y, NONE);

        // Act
        assert!(cancel(&mut state));

        // Assert
        assert!(preview(&state, &fx.view(Tool::Select)).is_empty());
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (100.0, 100.0));
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn resize_click_on_handle_changes_nothing() {
        // Arrange
        let (mut fx, ids) = selected_square();
        let mut state = State::default();
        let (x, y) = handle(&fx, BOTTOM_RIGHT);

        // Act
        click(&mut state, &mut fx, x, y, NONE);

        // Assert
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (100.0, 100.0));
        assert_eq!(fx.history.undo_len(), 0);
        assert_eq!(fx.selection, vec![ids[0]]);
    }

    #[test]
    fn resize_dragged_back_to_the_start_records_nothing() {
        // Arrange
        let (mut fx, _) = selected_square();
        let mut state = State::default();
        let (x, y) = handle(&fx, BOTTOM_RIGHT);

        // Act
        send(&mut state, &mut fx, Phase::Down, x, y, NONE);
        send(&mut state, &mut fx, Phase::Move, x + 80.0, y, NONE);
        send(&mut state, &mut fx, Phase::Move, x + 20.0, y, NONE);
        send(&mut state, &mut fx, Phase::Move, x, y, NONE);
        send(&mut state, &mut fx, Phase::Up, x, y, NONE);

        // Assert
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn handle_centers_keep_their_screen_offset_at_any_zoom() {
        // Arrange: the unit-wide outline adds half a world unit per side.
        let (mut fx, _) = selected_square();

        for zoom in [0.5_f32, 1.0, 4.0] {
            // Act
            fx.camera = Camera::new(Vec2::ZERO, zoom);
            let centers = handle_centers(&fx.doc, &fx.selection, &fx.camera);

            // Assert
            let Some(centers) = centers else {
                panic!("no handles at zoom {zoom}");
            };
            let (near, far) = (-0.5 * zoom, 100.5 * zoom);
            let pad = SELECTION_PAD_PX;
            let expected = [
                (near - pad, near - pad),
                (far + pad, near - pad),
                (far + pad, far + pad),
                (near - pad, far + pad),
            ];
            for (center, (x, y)) in centers.iter().zip(expected) {
                assert!(
                    center.approx_eq(Vec2::new(x, y), 1e-3),
                    "zoom {zoom}: {center:?}, want ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn handle_centers_none_without_anything_to_scale() {
        // Arrange: a line with both ends at one point.
        let (mut fx, ids) = Fixture::new(vec![line(5.0, 5.0, 5.0, 5.0)]);

        // Act / Assert
        assert!(handle_centers(&fx.doc, &fx.selection, &fx.camera).is_none());
        fx.selection = vec![ids[0]];
        assert!(handle_centers(&fx.doc, &fx.selection, &fx.camera).is_none());
    }

    #[test]
    fn scale_up_grows_ten_percent_about_centre() {
        // Arrange
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 100.0, 50.0)]);
        fx.selection = vec![ids[0]];

        // Act
        let changed = scale_selection_up(&mut fx.ctx(Tool::Select));

        // Assert
        assert!(changed);
        assert_box(&fx.shape(ids[0]), (-5.0, -2.5), (105.0, 52.5));
    }

    #[test]
    fn scale_down_shrinks_ten_percent_about_centre() {
        // Arrange
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 100.0, 50.0)]);
        fx.selection = vec![ids[0]];

        // Act
        let changed = scale_selection_down(&mut fx.ctx(Tool::Select));

        // Assert
        assert!(changed);
        assert_box(&fx.shape(ids[0]), (5.0, 2.5), (95.0, 47.5));
    }

    #[test]
    fn scale_key_is_one_undo_step() {
        // Arrange: two shapes selected.
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[0], ids[1]];

        // Act
        scale_selection_up(&mut fx.ctx(Tool::Select));
        scale_selection_down(&mut fx.ctx(Tool::Select));

        // Assert
        assert_eq!(fx.history.undo_len(), 2);
        assert!(fx.history.undo(&mut fx.doc));
        assert!(fx.history.undo(&mut fx.doc));
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (10.0, 10.0));
    }

    #[test]
    fn scale_key_without_selection_does_nothing() {
        // Arrange
        let (mut fx, _) = squares();

        // Act / Assert
        assert!(!scale_selection_up(&mut fx.ctx(Tool::Select)));
        assert!(!scale_selection_down(&mut fx.ctx(Tool::Select)));
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn resize_shift_keeps_proportions() {
        // Arrange: 100 × 50, pulled mostly to the right.
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 100.0, 50.0)]);
        fx.selection = vec![ids[0]];
        let mut state = State::default();

        // Act: the larger factor, 2, wins for both axes.
        drag_handle(&mut state, &mut fx, BOTTOM_RIGHT, (100.0, 10.0), SHIFT);

        // Assert
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (200.0, 100.0));
    }

    #[test]
    fn resize_shift_pressed_mid_drag_applies() {
        // Arrange
        let (mut fx, ids) = selected_square();
        let mut state = State::default();
        let (x, y) = handle(&fx, BOTTOM_RIGHT);
        send(&mut state, &mut fx, Phase::Down, x, y, NONE);
        send(&mut state, &mut fx, Phase::Move, x + 50.0, y, NONE);

        // Act: Shift goes down during the drag and the preview follows.
        send(&mut state, &mut fx, Phase::Move, x + 50.0, y, SHIFT);
        let overlay = preview(&state, &fx.view(Tool::Select));
        send(&mut state, &mut fx, Phase::Up, x + 50.0, y, SHIFT);

        // Assert
        assert_box(&overlay.shapes[0], (0.0, 0.0), (150.0, 150.0));
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (150.0, 150.0));
    }

    #[test]
    fn resize_scales_a_mixed_selection() {
        // Arrange: one of each kind inside the box (0,0)–(100,100).
        let style = Style {
            color: ColorId::INK,
            width: 3.0,
        };
        let stroke = Shape::Stroke {
            points: vec![Vec2::new(10.0, 10.0), Vec2::new(20.0, 30.0)],
            widths: Some(vec![1.0, 2.0]),
            style,
        };
        let arrow = Shape::Arrow {
            a: Vec2::new(10.0, 50.0),
            b: Vec2::new(40.0, 50.0),
            style,
        };
        let text = Shape::Text {
            origin: Vec2::new(50.0, 50.0),
            text: "ab".to_owned(),
            size: 10.0,
            style,
        };
        let grid = Shape::Grid {
            a: Vec2::new(60.0, 60.0),
            b: Vec2::new(90.0, 90.0),
            cols: 3,
            rows: 2,
            style,
            axes: false,
            fills: Vec::new(),
            texts: Vec::new(),
        };
        let (mut fx, ids) = Fixture::new(vec![
            rect(0.0, 0.0, 100.0, 100.0),
            ellipse(10.0, 10.0, 20.0, 20.0),
            line(0.0, 90.0, 10.0, 100.0),
            stroke,
            arrow,
            text,
            grid,
        ]);
        fx.selection.clone_from(&ids);
        let mut state = State::default();

        // Act: doubles everything about the origin.
        drag_handle(
            &mut state,
            &mut fx,
            BOTTOM_RIGHT,
            (100.0, 100.0),
            Modifiers::NONE,
        );

        // Assert
        assert_eq!(fx.history.undo_len(), 1);
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (200.0, 200.0));
        assert_box(&fx.shape(ids[1]), (20.0, 20.0), (60.0, 60.0));
        assert_box(&fx.shape(ids[6]), (120.0, 120.0), (180.0, 180.0));
        match fx.shape(ids[3]) {
            Shape::Stroke {
                points,
                widths,
                style,
            } => {
                assert!(points[1].approx_eq(Vec2::new(40.0, 60.0), 1e-3));
                assert_eq!(widths, Some(vec![1.0, 2.0]));
                assert!(approx_eq(style.width, 3.0, 1e-6));
            }
            other => panic!("expected a stroke, got {other:?}"),
        }
        match fx.shape(ids[5]) {
            Shape::Text { origin, size, .. } => {
                assert!(origin.approx_eq(Vec2::new(100.0, 100.0), 1e-3));
                assert!(approx_eq(size, 20.0, 1e-3));
            }
            other => panic!("expected text, got {other:?}"),
        }
    }

    #[test]
    fn resize_past_opposite_corner_stops_at_minimum() {
        // Arrange
        let (mut fx, ids) = selected_square();
        let mut state = State::default();

        // Act: far beyond the top-left corner, which would flip the box.
        drag_handle(
            &mut state,
            &mut fx,
            BOTTOM_RIGHT,
            (-500.0, -500.0),
            Modifiers::NONE,
        );

        // Assert
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (4.0, 4.0));
    }

    #[test]
    fn resize_minimum_holds_for_the_smallest_selected_shape() {
        // Arrange: a small square inside a large one.
        let (mut fx, ids) =
            Fixture::new(vec![rect(0.0, 0.0, 100.0, 100.0), rect(0.0, 0.0, 8.0, 8.0)]);
        fx.selection.clone_from(&ids);
        let mut state = State::default();

        // Act
        drag_handle(
            &mut state,
            &mut fx,
            BOTTOM_RIGHT,
            (-150.0, -150.0),
            Modifiers::NONE,
        );

        // Assert: the group stopped together, at factor 0.5.
        assert_box(&fx.shape(ids[1]), (0.0, 0.0), (4.0, 4.0));
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (50.0, 50.0));
    }

    #[test]
    fn scale_down_stops_at_minimum() {
        // Arrange
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 4.2, 4.2)]);
        fx.selection = vec![ids[0]];

        // Act: the first press reaches the minimum, the next finds no room.
        assert!(scale_selection_down(&mut fx.ctx(Tool::Select)));
        assert!(!scale_selection_down(&mut fx.ctx(Tool::Select)));

        // Assert
        assert_box(&fx.shape(ids[0]), (0.1, 0.1), (4.1, 4.1));
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn scale_up_overflow_changes_nothing() {
        // Arrange: a box so wide that 10 % more leaves the finite range.
        let mut huge = rect(0.0, 0.0, 1.0, 1.0);
        if let Shape::Rect { a, b, .. } = &mut huge {
            *a = Vec2::new(-3.3e38, -3.3e38);
            *b = Vec2::new(3.3e38, 3.3e38);
        }
        let (mut fx, ids) = Fixture::new(vec![huge.clone()]);
        fx.selection = vec![ids[0]];

        // Act
        let changed = scale_selection_up(&mut fx.ctx(Tool::Select));

        // Assert
        assert!(!changed);
        assert_eq!(fx.shape(ids[0]), huge);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn press_on_handle_wins_over_shape_below() {
        // Arrange: another shape lies under the selected square's handle.
        let (mut fx, ids) = Fixture::new(vec![
            filled_rect(0.0, 0.0, 100.0, 100.0),
            filled_rect(100.0, 100.0, 50.0, 50.0),
        ]);
        fx.selection = vec![ids[0]];
        let mut state = State::default();

        // Act
        drag_handle(
            &mut state,
            &mut fx,
            BOTTOM_RIGHT,
            (10.0, 10.0),
            Modifiers::NONE,
        );

        // Assert
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (110.0, 110.0));
        assert_box(&fx.shape(ids[1]), (100.0, 100.0), (150.0, 150.0));
        assert_eq!(fx.selection, vec![ids[0]]);
    }

    #[test]
    fn press_beside_handle_moves_the_shape() {
        // Arrange
        let (mut fx, ids) = selected_square();
        let mut state = State::default();

        // Act: inside the filled square, just off the handle.
        drag(&mut state, &mut fx, (95.0, 95.0), (105.0, 95.0), NONE);

        // Assert
        assert_box(&fx.shape(ids[0]), (10.0, 0.0), (110.0, 100.0));
    }

    #[test]
    fn alt_press_on_handle_duplicates() {
        // Arrange
        let (mut fx, ids) = selected_square();
        let mut state = State::default();
        let (x, y) = handle(&fx, BOTTOM_RIGHT);
        let inside = (x - 4.5, y - 4.5);

        // Act: on the handle and on the outline of the square.
        drag(
            &mut state,
            &mut fx,
            inside,
            (inside.0 + 20.0, inside.1),
            ALT,
        );

        // Assert
        let all = fx.ids();
        assert_eq!(all.len(), 2);
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (100.0, 100.0));
        assert_box(&fx.shape(all[1]), (20.0, 0.0), (120.0, 100.0));
        assert_eq!(fx.selection, vec![all[1]]);
    }

    #[test]
    fn shift_press_on_handle_resizes() {
        // Arrange
        let (mut fx, ids) = selected_square();
        let mut state = State::default();

        // Act: Shift would toggle the shape anywhere else.
        drag_handle(&mut state, &mut fx, BOTTOM_RIGHT, (50.0, 0.0), SHIFT);

        // Assert
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (150.0, 150.0));
        assert_eq!(fx.selection, vec![ids[0]]);
    }

    // ---- T101 align and distribute -----------------------------------------

    use crate::core::command::{Axis, Side};
    use proptest::prelude::*;

    /// Unfilled rectangle with corners `a` and `b` and outline `width`.
    fn outlined(a: Vec2, b: Vec2, width: f32) -> Shape {
        Shape::Rect {
            a,
            b,
            style: Style {
                color: ColorId::INK,
                width,
            },
            fill: None,
            label: None,
            text: None,
        }
    }

    #[test]
    fn align_selection_moves_by_bounds() {
        // Arrange: outlines 1 and 5 wide; bounds start at -0.5 and 37.5.
        let (mut fx, ids) = Fixture::new(vec![
            rect(0.0, 0.0, 10.0, 10.0),
            outlined(Vec2::new(40.0, 25.0), Vec2::new(60.0, 30.0), 5.0),
        ]);
        fx.selection = ids.clone();

        // Act
        let changed = align_selection(&mut fx.ctx(Tool::Select), Side::Left);

        // Assert: the bounds meet, so the thick rectangle's corner is
        // 2 further right than the thin one's.
        assert!(changed);
        assert!(approx_eq(fx.shape(ids[1]).bounds().min.x, -0.5, 1e-4));
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (10.0, 10.0));
        assert_box(&fx.shape(ids[1]), (2.0, 25.0), (22.0, 30.0));
        assert_eq!(fx.history.undo_len(), 1);
        assert_eq!(fx.selection, ids);
    }

    #[test]
    fn distribute_sorts_by_left_edge_not_selection_order() {
        // Arrange: x = 0, 100, 30 bottom first; selection order mixed.
        // gap = 11 / 4 = 2.75, so the bounds start at 13.25 and 27 (corners
        // 13.75 and 27.5); all centres meet the first one's y 5.
        let (mut fx, ids) = Fixture::new(vec![
            rect(0.0, 0.0, 10.0, 10.0),
            rect(100.0, 5.0, 10.0, 10.0),
            rect(30.0, 70.0, 10.0, 10.0),
        ]);
        fx.selection = vec![ids[1], ids[2], ids[0]];

        // Act
        let changed = distribute_selection(&mut fx.ctx(Tool::Select), Axis::Horizontal);

        // Assert
        assert!(changed);
        assert_box(&fx.shape(ids[0]), (0.0, 0.0), (10.0, 10.0));
        assert_box(&fx.shape(ids[1]), (27.5, 0.0), (37.5, 10.0));
        assert_box(&fx.shape(ids[2]), (13.75, 0.0), (23.75, 10.0));
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn arrange_without_enough_shapes_records_nothing() {
        // Arrange
        let (mut fx, ids) = squares();
        fx.selection = vec![ids[1]];

        // Act / Assert: one shape neither aligns nor distributes.
        for axis in [Axis::Horizontal, Axis::Vertical] {
            assert!(
                !distribute_selection(&mut fx.ctx(Tool::Select), axis),
                "{axis:?}"
            );
        }
        for side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
            assert!(
                !align_selection(&mut fx.ctx(Tool::Select), side),
                "{side:?}"
            );
        }
        assert_eq!(fx.history.undo_len(), 0);
    }

    /// Rectangles from `(x, y, w, h)` tuples.
    fn boxes(specs: &[(f32, f32, f32, f32)]) -> Vec<Shape> {
        specs.iter().map(|&(x, y, w, h)| rect(x, y, w, h)).collect()
    }

    /// Any coordinate, huge ones included.
    fn any_coord() -> impl Strategy<Value = f32> {
        prop_oneof![
            4 => -1_000.0f32..1_000.0,
            1 => -f32::MAX..f32::MAX,
            1 => prop_oneof![Just(f32::MAX), Just(-f32::MAX), Just(0.0f32)],
        ]
    }

    proptest! {
        #[test]
        fn distribute_lays_out_a_compact_row(
            specs in prop::collection::vec(
                (0.0f32..1_000.0, 0.0f32..1_000.0, 1.0f32..100.0, 1.0f32..100.0),
                2..8,
            ),
            vertical: bool,
        ) {
            // Arrange
            let (mut fx, ids) = Fixture::new(boxes(&specs));
            fx.selection = ids.clone();
            let axis = if vertical { Axis::Vertical } else { Axis::Horizontal };
            let lo = |b: &Aabb| if vertical { b.min.y } else { b.min.x };
            let hi = |b: &Aabb| if vertical { b.max.y } else { b.max.x };
            let mid = |b: &Aabb| if vertical {
                f32::midpoint(b.min.x, b.max.x)
            } else {
                f32::midpoint(b.min.y, b.max.y)
            };
            let before: Vec<Aabb> = ids.iter().map(|&i| fx.shape(i).bounds()).collect();
            let mut order: Vec<usize> = (0..ids.len()).collect();
            order.sort_by(|&p, &q| {
                lo(&before[p])
                    .total_cmp(&lo(&before[q]))
                    .then(hi(&before[p]).total_cmp(&hi(&before[q])))
                    .then(p.cmp(&q))
            });
            let first = order[0];
            let size = |b: &Aabb| hi(b) - lo(b);
            let min_gap = before.iter().map(size).fold(0.0f32, f32::max) / 4.0;

            // Act
            distribute_selection(&mut fx.ctx(Tool::Select), axis);

            // Assert: first fixed, one line, every gap a quarter of the
            // largest size, sizes kept.
            let after: Vec<Aabb> = ids.iter().map(|&i| fx.shape(i).bounds()).collect();
            prop_assert_eq!(after[first], before[first]);
            let gaps: Vec<f32> = order
                .windows(2)
                .map(|w| lo(&after[w[1]]) - hi(&after[w[0]]))
                .collect();
            for gap in &gaps {
                prop_assert!(approx_eq(*gap, min_gap, 1e-2), "{:?} != {}", gaps, min_gap);
            }
            for k in 0..ids.len() {
                prop_assert!(approx_eq(mid(&after[k]), mid(&before[first]), 1e-2));
                prop_assert!(approx_eq(size(&after[k]), size(&before[k]), 1e-2));
            }
            prop_assert!(fx.history.undo_len() <= 1);
        }

        #[test]
        fn align_and_distribute_stay_finite(
            corners in prop::collection::vec((any_coord(), any_coord(), any_coord(), any_coord()), 1..6),
        ) {
            // Arrange
            let shapes: Vec<Shape> = corners
                .iter()
                .map(|&(ax, ay, bx, by)| outlined(Vec2::new(ax, ay), Vec2::new(bx, by), 1.0))
                .collect();
            let (mut fx, ids) = Fixture::new(shapes);
            fx.selection = ids.clone();

            // Act
            for side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
                align_selection(&mut fx.ctx(Tool::Select), side);
            }
            for axis in [Axis::Horizontal, Axis::Vertical] {
                distribute_selection(&mut fx.ctx(Tool::Select), axis);
            }

            // Assert: every shape is still there and finite; undo restores.
            prop_assert_eq!(fx.ids(), ids.clone());
            for &i in &ids {
                let shape = fx.shape(i);
                prop_assert!(shape.is_finite(), "{:?}", shape);
                prop_assert!(shape.bounds().min.is_finite(), "{:?}", shape);
                prop_assert!(shape.bounds().max.is_finite(), "{:?}", shape);
            }
            prop_assert_eq!(&fx.selection, &ids);
            prop_assert!(fx.history.undo_len() <= 6);
            while fx.history.undo(&mut fx.doc) {}
            for (&i, &(ax, ay, bx, by)) in ids.iter().zip(&corners) {
                prop_assert_eq!(
                    fx.shape(i),
                    outlined(Vec2::new(ax, ay), Vec2::new(bx, by), 1.0)
                );
            }
        }
    }
}
