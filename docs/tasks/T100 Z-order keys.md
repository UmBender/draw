---
id: T100
title: Z-order keys
status: done
wave: 57
branch: task/T100-z-order-keys
depends_on: []
adrs: ["[[ADR-T100-1 Z-order keys and restack transactions]]"]
feature: "[[Editing tools]]"
tutorial: ""
tags: [task]
---

# T100 Z-order keys

## Goal

With a selection and no text being edited, `Ctrl+]` / `Ctrl+[` move it one step up or down the stacking order, and `Ctrl+Shift+]` / `Ctrl+Shift+[` to the top or bottom, each one undo step.

## Known facts (2026-11-10)

- Stacking is drawing order; the top shape takes the bucket click ([[Cell fill#Limits]]).
- `Ctrl+]` / `Ctrl+[` change text size only in text edit mode ([[ADR-T29-3 Resize text while typing]]); `Alt+]` / `Alt+[` resize the selection (ADR-T91-1).
- `Document` keeps an order; `index_of` reports it.

## Spec

Rules: [[ADR-T100-1 Z-order keys and restack transactions]].

- **AC-1** (owner) — Outside text edit mode, `Ctrl+]` raises the selection one step and `Ctrl+[` lowers it; in text edit mode they still resize text.
  - `keymap::tests::ctrl_brackets_restack_the_selection`, `keymap::tests::every_documented_binding`
  - `editor::tests::ctrl_bracket_right_raises_selection_one_step`
  - `editor::tests::ctrl_bracket_left_lowers_selection_one_step`
  - `editor::tests::ctrl_bracket_in_text_edit_resizes_text_not_order`
- **AC-2** — One step passes the nearest unselected shape above (or below) whose bounds overlap the selection's; with none, nothing changes and no undo step is added.
  - `editor::tests::raise_passes_only_the_nearest_overlapping_shape`
  - `editor::tests::lower_passes_only_the_nearest_overlapping_shape`
  - `editor::tests::raise_with_no_overlapping_shape_above_adds_no_undo_step`
  - `editor::tests::lower_with_no_overlapping_shape_below_adds_no_undo_step`
- **AC-3** — `Ctrl+Shift+]` / `Ctrl+Shift+[` move the selection to the top / bottom. Selected shapes keep their order among themselves.
  - `editor::tests::ctrl_shift_bracket_right_moves_selection_to_top_keeping_order`
  - `editor::tests::ctrl_shift_bracket_left_moves_selection_to_bottom_keeping_order`
  - `editor::tests::to_top_when_already_on_top_adds_no_undo_step`
  - `document::tests::tx_restack_moves_block_keeping_its_order`
  - `document::tests::tx_restack_same_order_is_empty`
  - `document::tests::tx_restack_clamps_below_and_skips_unknown_ids`
- **AC-4** — Each press is one undo step; undo restores the order exactly. The selection stays selected. Nothing happens during a gesture.
  - `editor::tests::restack_is_one_undo_step_and_keeps_selection`
  - `editor::tests::restack_during_gesture_does_nothing`
  - `editor::tests::restack_with_empty_selection_does_nothing`
  - `document::tests::tx_restack_inverse_restores_order_exactly`
  - `history::tests::undo_redo_restack_restores_order`
- **AC-5** — The new commands are fuzzed; the document order stays a permutation (no shape lost or doubled).
  - `document::tests::tx_restack_keeps_a_permutation_and_undoes_exactly` (proptest)
  - `fuzz::z_order_sessions_keep_a_permutation` (proptest), and the new chords in `arb_command_burst` through `BINDINGS`
  - `fuzz::z_order_session_click_selects_and_raises` (coverage: the session's clicks really restack)

## Out of scope

- Z-order of grid cells or text inside shapes.
- Toolbar buttons.

## Files owned

- `src/core/document.rs`, `src/core/history.rs`
- `docs/features/Editing tools.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T100-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T100): specify z-order keys`
- [x] tests — `test(T100): add failing tests for z-order keys`
- [x] models — `feat(T100): add restack commands and bindings`
- [x] behaviour — `feat(T100): implement restacking the selection`
- [x] quality — `chore(T100): pass clippy and rustfmt`
- [x] docs — `docs(T100): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
- 2026-11-10 — implemented in a batch run. `Command::{RaiseSelection, LowerSelection, SelectionToTop, SelectionToBottom}` bound to `Ctrl+]` / `Ctrl+[` with and without `Shift`; `document::tx_restack` builds the move from `Remove` + `Insert` edits, so no new edit kind and no snapshot format change. `scripts/check.sh --fuzz` and `scripts/e2e.sh` green; no golden changed.
- 2026-11-10 — UNATTENDED CHOICE: "the selection's bounds" is the union box of the selected shapes (the box the Select tool draws), and one step passes the nearest overlapping unselected shape above the *topmost* (below the *bottommost*) selected shape, the selection moving as one block (ADR-T100-1). Alternative: per-shape overlap.
- 2026-11-10 — UNATTENDED CHOICE: the restack logic lives in `editor.rs` (owned), not in `tools/select.rs` (not owned).
- 2026-11-10 — UNATTENDED CHOICE: macroquad has no brace key code, so the core binds `BracketRight` / `BracketLeft` with `Ctrl+Shift`. Whether every backend (X11, Wayland) reports the bracket key code while `Shift` is held cannot be checked headless — owner manual check: `Ctrl+Shift+]` on a selected shape in the running app.
- 2026-11-10 — the coverage test `z_order_session_click_selects_and_raises` was added in the behaviour commit (it passes by construction once the code exists); it confirms the fuzz session's clicks select a shape that `Ctrl+]` raises.
- 2026-11-10 — manual checks for the owner: fill a cell under a shape, raise and lower it, undo; `Ctrl+Shift+]` / `Ctrl+Shift+[` on X11 and Wayland.
- 2026-11-10 — integrated; the owner checked it by hand.
