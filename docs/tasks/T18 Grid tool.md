---
id: T18
title: Grid tool
status: ready
wave: 8
branch: task/T18-grid-tool
depends_on: [T16]
adrs: ["[[ADR-T16-1 Grid shape and shape labels]]"]
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

To be refined at the spec step; starting point:

- **AC-1** — `G` selects the grid tool; a left drag spans the grid's box and
  the preview shows the grid with the editor's current `cols × rows`
  (default 4 × 4). Release commits one `Shape::Grid` (one undo step); drags
  shorter than `MIN_DRAG_PX` commit nothing.
  *Tests:* `grid::tests::drag_commits_one_grid`, `short_drag_commits_nothing`,
  `preview_uses_current_dims`.
- **AC-2** — `←`/`→` change columns and `↑`/`↓` change rows during the drag;
  the preview updates live and the committed grid uses the final values; the
  values persist for the next grid. *Tests:* `grid::tests::arrows_change_dims_live`,
  `editor::tests` from T16 for the clamping.
- **AC-3** — `Shift` makes cells square (box adjusted to `cols × rows` square
  cells). *Test:* `grid::tests::shift_makes_square_cells`.
- **AC-4** — `Esc` cancels with no change; the grid is erasable, selectable,
  movable, copyable like any shape (via T16's shape model).
  *Tests:* `grid::tests::cancel_keeps_document`, fuzz.

## Out of scope

Per-cell fill or numbering, merged cells, resizing a committed grid.

## Files owned

`src/core/tools/grid.rs`, new `ADR-T18-*`.

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 18 — requires steps 9 and 16.

## Log
