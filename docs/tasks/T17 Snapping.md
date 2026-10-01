---
id: T17
title: Snapping
status: ready
wave: 8
branch: task/T17-snapping
depends_on: [T16]
adrs: ["[[ADR-T16-1 Grid shape and shape labels]]"]
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

To be refined at the spec step; tolerances are screen pixels converted with
the zoom at gesture time. Starting point:

- **AC-1** — Near-round: when smart snap is on, an ellipse/rect whose
  width/height ratio is within `ROUND_TOLERANCE` (≈ 10 %) of 1 becomes an exact
  circle/square. *Tests:* `snap::tests::near_circle_becomes_circle`,
  `far_from_round_is_unchanged`.
- **AC-2** — Match sizes: the dragged width and height each snap to the width
  or height of an existing `Rect`/`Ellipse` (or a grid cell), or an integer
  multiple of it, when within `SIZE_TOLERANCE_PX`. *Tests:*
  `snap::tests::size_snaps_to_existing_shape`, `size_snaps_to_multiple`.
- **AC-3** — Align: dragged corners/edges/centre snap to the edges and centres
  of shapes on screen within `ALIGN_TOLERANCE_PX`; the overlay shows a guide
  line for each active alignment. *Tests:* `snap::tests::edge_aligns_to_neighbour`,
  `centre_aligns`, `guides_listed_in_overlay`.
- **AC-4** — Grid snap: when grid snap is on, points snap to a world grid of
  `GRID_STEP` units and the renderer draws a faint dot grid. Grid snap
  applies before the smart snaps. *Tests:* `snap::tests::point_snaps_to_grid`,
  `render::tests::dot_grid_*`.
- **AC-5** — Order and override: grid → size → align → round, each step only
  when its toggle is on; holding `Alt` while dragging disables all snapping
  for that drag; `Shift` constraints still win. Applies to Line/Arrow
  endpoints (grid + align) and Rect/Ellipse. *Tests:* `shape_tool::tests::snap_*`.
- **AC-6** — All snap functions are pure, total and never return non-finite
  points (proptest). *Test:* `snap::tests::snap_is_finite_for_any_input`.

ADR to write at the spec step: `ADR-T17-1 Snapping order and tolerances`.

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
