---
id: T21
title: Cell fill and fill eraser
status: review
wave: 10
branch: task/T21-cell-fill
depends_on: [T18]
adrs: ["[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T18-3 Grid axis indices]]", "[[ADR-T21-1 Cell fills and a two-mode eraser]]", "[[ADR-T21-2 Eraser mode chosen on press]]"]
feature: "[[Cell fill]]"
tutorial: "[[21 Per-cell fills and a two-mode eraser]]"
tags: [task]
---

# T21 Cell fill and fill eraser

## Goal

The bucket fills a single cell of a grid (DP tables: mark visited states,
the answer cell, a path). The eraser can take a fill away without deleting
the shape: rubbing the inside of a filled circle, rectangle or grid cell
clears its fill; touching an outline still deletes the whole shape.

Requested by the owner on 2026-10-01. Branched from
`task/T18-grid-tool` (T18 is in review, not merged yet).

## Spec

Decision: [[ADR-T21-1 Cell fills and a two-mode eraser]].

- **AC-1** — Model: `Shape::Grid` gains `fills: Vec<CellFill>`
  (`CellFill { col, row, color }`), kept sorted by `(row, col)` with at
  most one entry per cell and only cells inside `cols × rows`. Cells are
  counted from corner `a` toward `b`, like the axis indices.
  `Shape::grid_cell_at(p)` gives the cell under a world point (inside the
  box, boundary inclusive), `grid_cell_rect(a, b, cols, rows, col, row)` its
  world box, `Shape::cell_fill(col, row)` reads and
  `Shape::with_cell_fill(col, row, Option<ColorId>)` sets or clears a fill
  (no-op on other shapes or out-of-range cells). Fills survive `translate`
  and clone (move, copy, undo). `bounds`, `hit` and `contains` of a grid
  are unchanged. `Shape::hit_outline(p, tol)` is `hit` ignoring fills.
  *Tests (`shape::tests`):* `grid_cell_at_counts_from_start_corner`,
  `grid_cell_at_reversed_drag`, `grid_cell_at_outside_is_none`,
  `grid_cell_rect_matches_lines`, `with_cell_fill_sets_and_replaces`,
  `with_cell_fill_none_clears`, `with_cell_fill_out_of_range_is_noop`,
  `with_cell_fill_keeps_sorted_unique`, `grid_fills_survive_translate`,
  `hit_outline_ignores_fill`; property `cell_fills_stay_valid`.
- **AC-2** — Bucket: a press fills the cell under the pointer of the
  topmost shape that is either a closed shape containing the point (as
  before) or a grid whose box contains it; the current colour, one undo
  step. Refilling a cell with the same colour changes nothing.
  *Tests (`bucket::tests`):* `click_in_grid_cell_fills_it`,
  `refill_cell_same_color_is_noop`, `cell_fill_is_one_undo_step`,
  `topmost_of_rect_and_grid_wins`, `click_outside_grid_box_does_nothing`.
- **AC-3** — Renderer: filled cells are drawn under the grid lines; axis
  indices keep the outline colour (they are outside the cells). Manual
  check; geometry covered by `grid_cell_rect_matches_lines`.
- **AC-4** — Eraser (and right drag), mode chosen on press
  ([[ADR-T21-2 Eraser mode chosen on press]]): a press inside a filled
  `Rect`/`Ellipse` (not on its outline) or a filled grid cell (not on a
  line) starts **clear mode** — the drag clears every cell and closed-shape
  fill it passes over and never removes a shape. Any other press starts
  **remove mode**, unchanged from before. The preview hides affected shapes
  and, in clear mode, shows them without the cleared fills; release commits
  one undo step. *Tests (`eraser::tests`):* `inside_filled_rect_clears_fill`,
  `inside_filled_ellipse_clears_fill`, `press_on_outline_removes_filled_rect`,
  `unfilled_inside_is_untouched`, `filled_cell_clears_only_that_cell`,
  `clear_drag_crosses_lines_without_removing`,
  `clear_drag_also_clears_shape_fills`,
  `remove_drag_into_filled_rect_removes_it`, `preview_shows_cleared_fill`,
  `erase_fill_is_one_undo_step`.
- **AC-5** — Fuzz: grid fills stay inside `cols × rows`, sorted and
  unique, in every editor state.
  *Test:* `fuzz::editor_never_panics_and_keeps_invariants` (extended).

## Out of scope

Flood fill across cells, clearing with the bucket, filling a cell of a grid
under a closed shape that covers it (the topmost shape wins), changing a
grid's dimensions after creation, colouring the axis indices.

## Files owned

`src/core/shape.rs`, `src/core/tools/bucket.rs`,
`src/core/tools/eraser.rs`, `src/shell/render.rs`, `tests/fuzz.rs`;
`src/core/tools/grid.rs` and `src/core/snap.rs` (only `fills` in
`Shape::Grid` literals); `docs/architecture/Keymap.md` (eraser
description); new `ADR-T21-*`, feature and tutorial notes.

## Subtasks (one commit each)

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs

## Learning path

Step 21 — requires steps 10 and 18.

## Log

- 2026-10-01 — spec: owner asked for per-cell fill with the bucket and for
  the eraser to clear fills on cells, circles and rectangles; axis indices
  keep their colour. Task created on the T18 branch because it depends on
  the grid shape; `Task Board`/`Learning Path` rows are for the integrator.
- spec refined before the tests: per-shape outline/inside effects would
  delete a grid whenever a drag crosses a line; the mode is now chosen on
  press (ADR-T21-2, amends ADR-T21-1).
- tests: shape, bucket, eraser unit tests, `cell_fills_stay_valid`
  property, fuzz invariant on fills; red as compile errors.
- models: `CellFill`, `fills`, method signatures, eraser `Mode` and
  `cleared` state.
- behaviour: `hit` now delegates to `hit_outline`. All tests green; the
  behaviour commit kept an eager clone in `clear_at` (an edit missed after
  rustfmt reflowed the line), fixed in the quality commit.
- quality: `clear_at` returns `Option` without cloning; `grid_cell_at`
  uses a finite check instead of `a == b` (`clippy::float_cmp`).
  `PROPTEST_CASES=20000` fuzz green.
- Files touched: as owned, plus this note, two ADRs, the feature and
  tutorial notes. `clipboard.rs` testkit untouched (tests define their own
  grid helper). `Architecture.md` unchanged (module map row still fits).
- Not checked on screen: cell fill rendering under lines and the eraser
  preview need a manual look.
