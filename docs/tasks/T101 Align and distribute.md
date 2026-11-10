---
id: T101
title: Align and distribute
status: review
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

- [x] spec — `docs(T101): specify align and distribute`
- [x] tests — `test(T101): add failing tests for align and distribute`
- [x] models — `feat(T101): add align and distribute commands and bindings`
- [x] behaviour — `feat(T101): implement aligning and distributing the selection`
- [x] quality — skipped: `scripts/check.sh --fuzz` was green on the behaviour commit with nothing to change (see *Log*)
- [x] docs — `docs(T101): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
- 2026-11-10 — implemented in a batch run. `Command::Align(Side)` on `Alt+←/→/↑/↓` and `Command::Distribute(Axis)` on `Alt+H` / `Alt+V` (`Side`, `Axis` in `core::command`); `select::align_selection` / `select::distribute_selection` build one `Replace` transaction; the editor's `arrange` ignores them during a gesture. The shell already passes `Alt` with every key (as for `Alt+]`), so `Alt+arrow` reaches the core; no alternative key was needed. `scripts/check.sh --fuzz` and `scripts/e2e.sh` green; no golden changed.
- 2026-11-10 — quality step skipped: `cargo fmt` ran in every step and `scripts/check.sh --fuzz` (fmt, clippy `-D warnings`, tests, docs, 20 000 fuzz cases) passed on the behaviour commit with no change.
- 2026-11-10 — UNATTENDED CHOICE: distribute sorts by near edge, then far edge, then stacking order, and keeps the first and last of that order fixed; the gap is `(last.min − first.max − Σ middle sizes) / (n − 1)` (ADR-T101-1). Alternative: equal centre spacing.
- 2026-11-10 — UNATTENDED CHOICE: the logic lives in `tools/select.rs` (listed as owned when a move helper is shared): it reuses `selected_shapes`, `prune` and the `Replace` transaction of moves and scaling.
- 2026-11-10 — UNATTENDED CHOICE: the no-op tolerance (`1e-5` of the box's magnitude, at least `1e-5` world units) applies to the command as a whole: when every offset is within it nothing is recorded, otherwise every shape with a non-zero offset moves by its exact offset, so the gaps come out equal. ADR-T101-1 words it per shape; the two only differ below that tolerance.
- 2026-11-10 — found while fuzzing (no bug): a second click on the only selected rectangle opens its text (ADR-T31-1), so a following `Alt+←` moves the caret and `Undo` cancels the text. The fuzz property closes the text with `Esc` before its final undos, as the T100 one does.
- 2026-11-10 — manual checks for the owner: align a few uneven graph nodes with `Alt+arrows`, space them with `Alt+H` / `Alt+V`, undo; `Alt+arrow` on X11 and Wayland (some desktops bind `Alt+arrow` themselves).
