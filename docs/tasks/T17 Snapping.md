---
id: T17
title: Snapping
status: in-progress
wave: 8
branch: task/T17-snapping
depends_on: [T16]
adrs: ["[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T16-3 Helper settings and hooks]]", "[[ADR-T17-1 Snapping order and tolerances]]"]
feature: "[[Snapping]]"
tutorial: "[[17 Snapping and alignment guides]]"
tags: [task]
---

# T17 Snapping

## Goal

Shapes come out neat without effort: almost-circles become circles, boxes
match the size of boxes already drawn, corners land on a grid, and edges line
up with nearby shapes, with guides showing what snapped.

## Spec

Decisions: [[ADR-T17-1 Snapping order and tolerances]]. Tolerances are screen
pixels converted to world units with the zoom at gesture time
(`camera.world_len`). The *drag* is the shape tool's `start` (on `Down`) and
moving `end`; a **box** drag is `Rect`/`Ellipse`, a **point** drag is
`Line`/`Arrow`. Snap targets are the document shapes: `Rect`, `Ellipse` and
`Grid` contribute their box (edges and centre) and sizes; `Line`/`Arrow`
contribute their endpoints to alignment; freehand strokes contribute nothing.

- **AC-1** — Round: with smart snap on, a box drag whose
  `min(|w|,|h|) / max(|w|,|h|) ≥ 1 − ROUND_TOLERANCE` (0.1) becomes square:
  both sides take the larger length, signs kept. *Tests:*
  `snap::tests::near_circle_becomes_circle`,
  `snap::tests::far_from_round_is_unchanged`,
  `snap::tests::round_keeps_drag_direction`.
- **AC-2** — Sizes: with smart snap on, each of a box drag's `|w|` and `|h|`
  snaps to `k · c` (`k` in `1..=MAX_SIZE_MULTIPLE` = 8) for any size `c`
  (width or height of a `Rect`/`Ellipse`/`Grid` box, or of a grid cell) when
  within `SIZE_TOLERANCE_PX` (6 px); the nearest wins, smaller `k` on ties.
  *Tests:* `snap::tests::size_snaps_to_existing_shape`,
  `snap::tests::size_snaps_to_multiple`, `snap::tests::size_snaps_to_grid_cell`,
  `snap::tests::size_out_of_tolerance_is_unchanged`.
- **AC-3** — Align: with smart snap on, the start point and the moving end
  snap per axis to a target coordinate within `ALIGN_TOLERANCE_PX` (6 px);
  for a box drag the end may instead put the box centre on a target. The
  result carries one guide (world segment along the target line, spanning
  the target and the dragged box) for each x/y line of the final shape —
  box edges and centre, or the line's endpoints — that lies on a target.
  *Tests:* `snap::tests::edge_aligns_to_neighbour`, `snap::tests::centre_aligns`,
  `snap::tests::line_end_aligns_to_endpoint`,
  `snap::tests::align_out_of_tolerance_is_unchanged`,
  `snap::tests::guides_listed_in_overlay` (shape tool preview carries them).
- **AC-4** — Grid: with grid snap on, the start and the end snap to the
  nearest multiple of `GRID_STEP` (20 world units) on each axis, before the
  smart snaps. With grid snap on the renderer draws a faint dot grid on the
  multiples of the grid step, coarsened by powers of two so dots stay at
  least `DOT_GRID_MIN_PX` apart. *Tests:* `snap::tests::point_snaps_to_grid`,
  `snap::tests::grid_runs_before_smart_snaps`,
  `render::tests::dot_grid_spacing_at_unit_zoom_is_grid_step`,
  `render::tests::dot_grid_spacing_coarsens_when_zoomed_out`,
  `render::tests::dot_grid_spacing_non_finite_is_none`,
  `render::tests::dot_grid_points_cover_view_on_multiples`.
- **AC-5** — Order and override: the end goes grid → size → align → round,
  each step only when its toggle is on (size and round only for box drags);
  the start goes grid → align. `Alt` held on an event disables all snapping
  for that event (sampled per event, like `Shift`). With `Shift` held the end
  only gets grid snap, then the Shift constraint, so the constraint always
  holds exactly; no guides are shown. *Tests:*
  `shape_tool::tests::snap_off_by_default_is_unchanged`,
  `shape_tool::tests::snap_grid_applies_to_line_endpoints`,
  `shape_tool::tests::snap_smart_rect_matches_existing_size`,
  `shape_tool::tests::snap_alt_disables_snapping`,
  `shape_tool::tests::snap_shift_constraint_wins`,
  `shape_tool::tests::snap_line_is_not_size_snapped`.
- **AC-6** — Snap functions are pure and total: for finite inputs they
  return finite points and guides; a step whose result would not be finite
  is skipped. *Tests:* `snap::tests::snap_is_finite_for_any_input`,
  `snap::tests::grid_snap_is_finite_for_any_input`,
  `shape_tool::tests::shape_gesture_never_panics_and_is_finite` (snapping on).

## Out of scope

Snapping while moving a selection, snapping freehand strokes, rotation snaps.

## Files owned

`src/core/snap.rs`, `src/core/tools/shape_tool.rs`, `src/shell/render.rs`
(guides and dot grid only), new `ADR-T17-*`.

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 17 — requires steps 9 and 16.

## Log
