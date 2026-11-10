---
id: T101
title: Align and distribute
status: in-progress
wave: 58
branch: task/T101-align-distribute
depends_on: [T100]
adrs: ["[[ADR-T101-1 Align and distribute keys]]"]
feature: "[[Editing tools]]"
tutorial: ""
tags: [task]
---

# T101 Align and distribute

## Goal

With two or more shapes selected, `Alt+←/→/↑/↓` align them to that side of the selection's box, and with three or more `Alt+H` / `Alt+V` space them with equal gaps, each one undo step.

## Known facts (2026-11-10)

- Smart snap aligns while drawing; moving a selection does not snap ([[Snapping#Limits]]).
- `Alt` + drag duplicates; plain arrows resize the one selected grid under the Select tool (ADR-T85-1). `Alt+arrow`, `Alt+H` and `Alt+V` are unbound.
- Branches from T100 only because both edit the key and command tables.

## Spec

Rules: [[ADR-T101-1 Align and distribute keys]].

- **AC-1** — `Alt+←` moves every selected shape so its left edge meets the selection box's left edge; `→`, `↑`, `↓` likewise for right, top and bottom. Each shape moves as a whole (by its bounds).
  - `keymap::tests::alt_arrows_align_and_alt_h_v_distribute`, `keymap::tests::every_documented_binding`
  - `editor::tests::alt_arrow_left_aligns_left_edges`
  - `editor::tests::alt_arrows_align_each_side`
  - `editor::tests::align_moves_grid_and_text_as_whole_shapes`
  - `select::tests::align_selection_moves_by_bounds`
- **AC-2** (owner) — `Alt+H` keeps the leftmost and rightmost shapes in place and moves the others so the horizontal gaps between neighbours (sorted by left edge) are equal; `Alt+V` does the same vertically. Overlapping shapes may get a negative gap.
  - `editor::tests::alt_h_spaces_shapes_with_equal_gaps`
  - `editor::tests::alt_v_spaces_shapes_with_equal_vertical_gaps`
  - `editor::tests::distribute_overlapping_shapes_gives_negative_gap`
  - `select::tests::distribute_sorts_by_left_edge_not_selection_order`
  - `select::tests::distribute_gaps_are_equal` (proptest)
- **AC-3** — Fewer than two shapes (align) or three (distribute), or nothing that would move, changes nothing and adds no undo step.
  - `editor::tests::align_with_one_shape_does_nothing`
  - `editor::tests::distribute_with_two_shapes_does_nothing`
  - `editor::tests::align_when_already_aligned_adds_no_undo_step`
  - `editor::tests::distribute_twice_adds_one_undo_step`
- **AC-4** — Each command is one undo step and keeps the selection; ignored during a gesture or text edit.
  - `editor::tests::align_is_one_undo_step_and_keeps_selection`
  - `editor::tests::align_during_gesture_does_nothing`
  - `editor::tests::alt_arrow_in_text_edit_moves_caret_not_shapes`
- **AC-5** — Fuzzed; results stay finite.
  - `select::tests::align_and_distribute_stay_finite` (proptest, huge coordinates)
  - `fuzz::align_sessions_stay_finite_and_undo` (proptest), and the new chords in `arb_command_burst` through `BINDINGS`
  - `fuzz::align_session_select_all_aligns` (coverage: the session's keys really move shapes)

## Out of scope

- Aligning to the canvas or to a key object; edges attached to moved graph nodes do not follow (they are separate shapes).
- Toolbar buttons.

## Files owned

- `src/core/tools/select.rs` (the align and distribute functions reuse its move helpers), `src/core/geom.rs`
- `docs/features/Editing tools.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T101-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T101): specify align and distribute`
- [ ] tests — `test(T101): add failing tests for align and distribute`
- [ ] models — `feat(T101): …`
- [ ] behaviour — `feat(T101): …`
- [ ] quality — `chore(T101): pass clippy and rustfmt`
- [ ] docs — `docs(T101): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
