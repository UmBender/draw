---
id: T85
title: Resize a committed grid
status: review
wave: 51
branch: task/T85-resize-grid
depends_on: []
adrs: ["[[ADR-T85-1 Arrows resize the one selected grid]]"]
feature: "[[Grid tool]]"
tutorial: ""
tags: [task]
---

# T85 Resize a committed grid

## Goal

With one grid selected, the arrow keys add or remove its columns and rows, keeping its contents. (idea E1 of [[Feature proposal]])

## Known facts (2026-11-09)

- `Command::GridCols` and `GridRows` only change `helpers` today (`src/core/editor.rs`).
- A committed grid stores `fills` and `texts` by `(row, col)`.

## Spec

The decision and its edge cases are in [[ADR-T85-1 Arrows resize the one selected grid]].

- **AC-1** — With the Select tool and exactly one grid selected, `→`/`←` and `↓`/`↑` add or remove a column or row, clamped to 1–64, and `helpers` is unchanged.
  Tests: `editor::tests::grid_resize_arrows_change_selected_grid_not_helpers`, `editor::tests::grid_resize_clamps_to_cell_limits`, `shape::tests::grid_stepped_clamps_large_deltas`.
- **AC-2** — Cell size stays; the box grows or shrinks at the right and bottom edges, on screen, whatever the drag direction.
  Tests: `shape::tests::grid_stepped_moves_right_and_bottom_edges`, `shape::tests::grid_stepped_reversed_grid_moves_right_and_bottom_edges`.
- **AC-3** — Text and fills of remaining cells stay; those of a removed cell go, and undo restores them.
  Tests: `shape::tests::grid_stepped_keeps_remaining_cells_drops_removed`, `shape::tests::grid_stepped_reversed_grid_keeps_cells_in_place`, `editor::tests::grid_resize_undo_restores_removed_cells`, property `shape::tests::grid_stepped_stays_valid`.
- **AC-4** — Each press that changes the grid is one undo step; a press at the limit adds none.
  Tests: `editor::tests::grid_resize_each_press_is_one_undo_step`, `editor::tests::grid_resize_at_limit_adds_no_undo_step`, `editor::tests::grid_resize_marks_the_document_changed`.
- **AC-5** — With anything else selected the arrows behave as they do today.
  Tests: `editor::tests::grid_arrows_change_helpers_unless_one_grid_is_selected`, `editor::tests::grid_arrows_change_helpers_with_another_tool`.
- **Robustness** — A coordinate that would stop being finite leaves the grid alone; a change during a gesture cancels it first; random arrow, undo and redo presses on a filled grid keep every invariant.
  Tests: `shape::tests::grid_stepped_refuses_non_finite_result`, `editor::tests::grid_resize_during_a_move_cancels_it`, `editor::tests::grid_resize_at_limit_keeps_the_gesture`, fuzz `grid_resize_sessions_keep_invariants`.

## Out of scope

- Inserting a row or column in the middle.
- Drag handles (T91).

## Files owned

- `src/core/shape.rs`, `src/core/tools/select.rs`
- `docs/features/Grid tool.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T85-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T85): specify resize a committed grid`
- [x] tests — `test(T85): add failing tests`
- [x] models — `feat(T85): add types`
- [x] behaviour — `feat(T85): implement`
- [x] quality — `chore(T85): pass clippy and rustfmt`
- [x] docs — `docs(T85): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — UNATTENDED CHOICE: on a grid drawn leftwards or upwards the right or bottom edge is corner `a`, so that corner moves and the cell indices shift, which keeps AC-2 and AC-3 true on screen ([[ADR-T85-1 Arrows resize the one selected grid]]). On such a grid with axis indices the numbers of the existing cells change by the step.
- 2026-11-09 — UNATTENDED CHOICE: a step that changes the grid cancels a gesture in progress, like the other document-editing commands; a press at the limit leaves it alone.
- 2026-11-09 — UNATTENDED CHOICE: `Editor::clear_all` was split out of `execute` to stay under the clippy line limit, with no behaviour change. `src/core/tools/select.rs` and `src/core/keymap.rs` did not need changes.
- 2026-11-09 — no new command or event, so `arb_session` is unchanged; `grid_resize_sessions_keep_invariants` was added to `tests/fuzz.rs`. Snapshot format unchanged.
- 2026-11-09 — manual check left for the owner: add a column to a filled table and undo it.
