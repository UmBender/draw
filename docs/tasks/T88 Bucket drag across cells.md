---
id: T88
title: Bucket drag across cells
status: review
wave: 51
branch: task/T88-bucket-drag
depends_on: []
adrs: ["[[ADR-T88-1 Bucket drag previews cells and commits on release]]"]
feature: "[[Cell fill]]"
tutorial: ""
tags: [task]
---

# T88 Bucket drag across cells

## Goal

Dragging the bucket across a grid fills every cell it passes over, in one undo step. (idea E3 of [[Feature proposal]])

## Known facts (2026-11-09)

- `core::tools::bucket::on_pointer` acts on the press only.
- In pen mode a tap is delivered as a click ([[ADR-T34-1 Tap slop for pen tablets detected at start-up]]).

## Spec

Tests are in `src/core/tools/bucket.rs` unless marked *(editor)*, which are
in `src/core/editor.rs`. Design: [[ADR-T88-1 Bucket drag previews cells and commits on release]].

- **AC-1** — Pressing in a grid cell and dragging fills each cell passed over with the current colour, staying in that grid.
  `drag_fills_each_cell_passed_over`, `drag_stays_in_the_pressed_grid`,
  `drag_leaving_and_reentering_fills_only_inside`,
  `drag_on_reversed_grid_counts_from_the_start_corner`,
  `drag_uses_the_current_colour`,
  `drag_preview_shows_the_painted_grid_over_the_hidden_one`,
  `bucket_drag_survives_a_snapshot_round_trip` *(editor)*.
- **AC-2** — A fast drag leaves no gaps: the segment between two moves is walked cell by cell.
  `fast_move_fills_every_cell_on_the_segment`,
  `fast_diagonal_move_leaves_no_gaps`,
  `release_far_from_the_last_move_walks_the_rest`,
  `cells_along_is_connected_and_inside` (property).
- **AC-3** — The whole drag is one undo step, and no step at all if no cell changed.
  `drag_is_one_undo_step`,
  `drag_changes_the_document_only_on_release`,
  `drag_over_cells_of_the_same_colour_records_no_step`,
  `bucket_drag_moves_keep_the_document_revision` *(editor)*.
- **AC-4** — `Esc` during the drag restores the cells.
  `cancel_discards_the_drag`,
  `bucket_escape_during_drag_restores_the_cells` *(editor)*.
- **AC-5** — A press on a rectangle, ellipse or empty space behaves as today.
  `press_on_rect_fills_at_once_and_ignores_the_drag`,
  `press_on_ellipse_fills_at_once_and_ignores_the_drag`,
  `press_on_empty_space_then_drag_into_grid_fills_nothing`,
  and the unchanged click tests of T10 and T21.
  A tap in pen mode fills the one pressed cell:
  `bucket_pen_tap_fills_the_pressed_cell_only` *(editor)*.

## Out of scope

- Filling enclosed regions.
- Clearing fills with the bucket.

## Files owned

- `src/core/tools/bucket.rs`
- `src/core/document.rs`
- `docs/features/Cell fill.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- `src/core/tools/mod.rs` (the `[tools]` snapshot section now holds the bucket's drag)
- `docs/features/Pen tablets.md` (its limit about the Bucket fill on the tip's press)
- this note, and new `ADR-T88-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T88): specify bucket drag across cells`
- [x] tests — `test(T88): add failing tests`
- [x] models — `feat(T88): add types`
- [x] behaviour — `feat(T88): implement`
- [x] quality — `chore(T88): pass clippy and rustfmt`
- [x] docs — `docs(T88): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — the spec step took two commits: the first held only the ADR because the script editing this note was refused, and history is not rewritten.
- 2026-11-09 — UNATTENDED CHOICE: added `src/core/tools/mod.rs` to *Files owned*: the bucket now has gesture state, and the `[tools]` snapshot section is written there.
- 2026-11-09 — UNATTENDED CHOICE: added `docs/features/Pen tablets.md` to *Files owned*: its limit says a Bucket fill happens on the tip's press, which is no longer true for grid cells.
- 2026-11-09 — UNATTENDED CHOICE: a grid press commits on release, with the cells shown in the overlay meanwhile, as the eraser's clear drag does (ADR-T88-1).
- 2026-11-09 — no fuzz or fixture change: no new event or command, and the snapshot's `bucket` record is only written during a drag, so `VERSION` stays 1.
- 2026-11-09 — manual check left for the owner: drag a path across a DP table in the running app (not viewed here).
