---
id: T18
title: Grid tool
status: review
wave: 8
branch: task/T18-grid-tool
depends_on: [T16]
adrs: ["[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T18-1 Grid drag reads live dims and snaps as a box]]", "[[ADR-T18-2 Grid size flyout in the toolbar]]"]
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

- **AC-6** (added on owner request, [[ADR-T18-2 Grid size flyout in the toolbar]])
  — While the grid tool is active, the toolbar shows a flyout right of the
  strip, aligned with the `G` button: rows `cols` and `rows`, each with
  `-` and `+` buttons that run `GridCols(∓1)`/`GridRows(∓1)`, and the
  current values. Presses inside the flyout never reach the canvas; with
  another tool or a hidden toolbar it is absent and presses there are
  forwarded.
  *Tests (`toolbar::tests`):* `flyout_has_four_buttons_in_order`,
  `flyout_is_right_of_strip_aligned_with_grid_button`,
  `flyout_buttons_inside_panel_and_disjoint`,
  `flyout_buttons_return_their_commands`, `route_flyout_click_applies`,
  `route_flyout_gap_is_swallowed`, `route_without_flyout_forwards`;
  existing `route_*` tests updated for the new parameter.

## Out of scope

Per-cell fill or numbering, merged cells, resizing a committed grid.

## Files owned

`src/core/tools/grid.rs`, new `ADR-T18-*`.

Amended for AC-6 (approved by the owner): `src/shell/toolbar.rs` (flyout
layout, routing, drawing) and `src/shell/app.rs` (the `dispatch` call
passes whether the grid tool is active).

## Subtasks (one commit each)

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs
- AC-6 flyout: [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs

## Learning path

Step 18 — requires steps 9 and 16.

## Log

- 2026-10-01 — spec: AC-1…AC-5 with test names; ADR-T18-1 records that the
  tool reads dims live from `Helpers` and snaps like a box drag. All wiring
  (routing, `G`, arrows, toolbar, fuzz) already exists from T16, so only
  `grid.rs` changes.
- tests: 15 unit tests and 2 properties in `grid::tests`; 6 of them
  (idle/cancel/short-drag cases and the properties) pass vacuously
  against the stub, the other 11 fail as expected.
- models: `Drag` and `square_cells` signatures with `todo!()` bodies.
- behaviour: drag, snap (box), preview and commit with live dims, Shift
  square cells, cancel. All tests green at the first run.
- quality: rustfmt reflow of one line and a redundant rustdoc link target.
  `PROPTEST_CASES=20000 cargo test --release --test fuzz` green.
- AC-5 fuzz needed no new test: T16 already drives grid drags with arrow
  keys (`grid_drags_press_arrows_while_the_grid_tool_drags`) and now they
  create grids.
- Files touched: `src/core/tools/grid.rs`, plus this note, ADR-T18-1, the
  feature and tutorial notes and `Architecture.md` (module map row for
  `tools::grid`, was "stub until T18").
- Not checked on screen: grid rendering and arrow-key feel need a manual
  look in the running app.
- AC-6 spec: owner asked for a toolbar counter of columns and rows; the
  strip has no vertical room, so a flyout next to `G` shown only with the
  grid tool ([[ADR-T18-2 Grid size flyout in the toolbar]]). Files owned
  amended with `toolbar.rs` and `app.rs`.
- AC-6 tests: 7 new `toolbar::tests`, existing `route_*` tests pass
  `flyout = false`; red as a compile error (missing API).
- AC-6 models/behaviour: `ButtonKind::GridCols`/`GridRows`,
  `flyout_panel`, `flyout_layout`, `draw_flyout`, `route(visible, flyout,
  …)`; `app::dispatch` passes `editor.tool() == Tool::Grid`.
- AC-6 quality: `scripts/check.sh` green with no changes, so no quality
  commit.
- AC-6 not checked on screen: flyout look (label fit at 16 px, alignment)
  needs a manual look with the grid tool active. `docs/features/Toolbar.md`
  is not owned by this task and does not mention the flyout yet.
