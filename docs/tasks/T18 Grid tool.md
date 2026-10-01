---
id: T18
title: Grid tool
status: in-progress
wave: 8
branch: task/T18-grid-tool
depends_on: [T16]
adrs: ["[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T18-1 Grid drag reads live dims and snaps as a box]]"]
feature: "[[Grid tool]]"
tutorial: "[[18 A grid tool with live parameters]]"
tags: [task]
---

# T18 Grid tool

## Goal

Draw an `n × m` grid (DP tables, boards, matrices) in one drag, like a
rectangle, choosing the number of columns and rows with the arrow keys while
dragging.

## Spec

Decision: [[ADR-T18-1 Grid drag reads live dims and snaps as a box]] — dims
are read from `Helpers` at preview and release, never stored in the drag.

- **AC-1** — `G` selects the grid tool (T16 binding). A left drag spans the
  grid's box and the preview shows a `Shape::Grid` with the editor's current
  `cols × rows` (default 4 × 4) and the drawing style (width in world units).
  Release commits exactly one `Shape::Grid` as one undo step; drags shorter
  than `shape_tool::MIN_DRAG_PX` on screen commit nothing. The preview is
  empty when idle; events without a drag are ignored.
  *Tests (`grid::tests`):* `drag_commits_one_grid`,
  `commit_is_one_undo_step`, `short_drag_commits_nothing`,
  `preview_uses_current_dims`, `preview_empty_when_idle`,
  `move_without_drag_is_ignored`, `width_is_world_units_at_zoom`;
  `keymap::tests::helper_bindings` (existing, `G`).
- **AC-2** — Arrow keys change `cols`/`rows` during the drag (T16 commands,
  no gesture cancel); the preview follows the current values and the
  committed grid uses the values at release; the values stay for the next
  grid. *Tests:* `grid::tests::arrows_change_dims_live`,
  `grid::tests::dims_persist_for_next_grid`;
  `editor::tests::grid_dims_clamped`, `grid_dims_change_keeps_gesture`
  (existing, T16).
- **AC-3** — `Shift` makes cells square: the box becomes
  `side·cols × side·rows` with `side = max(|w|/cols, |h|/rows)`, keeping the
  drag direction; it stays exact when dims change mid-drag.
  *Tests:* `grid::tests::shift_makes_square_cells`,
  `shift_keeps_drag_direction`, property `shift_cells_always_square`.
- **AC-4** — Snapping as a box drag (ADR-T17-1): with grid snap on, both
  corners land on the world grid; alignment guides show in the preview.
  *Tests:* `grid::tests::grid_snap_snaps_corners`,
  `grid::tests::preview_shows_guides`.
- **AC-5** — `Esc` (cancel) discards the drag with no document change; a
  later `Up` commits nothing. The committed grid is erasable, selectable,
  movable and copyable through T16's shape model; the fuzz harness drives
  grid drags with arrow keys and checks the invariants.
  *Tests:* `grid::tests::cancel_keeps_document`,
  `grid::tests::cancel_when_idle_needs_no_redraw`, property
  `committed_grid_is_finite_and_in_range`;
  `fuzz::editor_never_panics_and_keeps_invariants` (existing).

## Out of scope

Per-cell fill or numbering, merged cells, resizing a committed grid.

## Files owned

`src/core/tools/grid.rs`, new `ADR-T18-*`.

## Subtasks (one commit each)

- [x] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 18 — requires steps 9 and 16.

## Log

- 2026-10-01 — spec: AC-1…AC-5 with test names; ADR-T18-1 records that the
  tool reads dims live from `Helpers` and snaps like a box drag. All wiring
  (routing, `G`, arrows, toolbar, fuzz) already exists from T16, so only
  `grid.rs` changes.
