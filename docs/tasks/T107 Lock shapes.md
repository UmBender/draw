---
id: T107
title: Lock shapes
status: in-progress
wave: 59
branch: task/T107-lock
depends_on: []
adrs: [ADR-T107-1, ADR-T107-2]
feature: "[[Editing tools]]"
tutorial: ""
tags: [task]
---

# T107 Lock shapes

## Goal

`Ctrl+K` locks the selected shapes so a background grid or diagram can no longer be selected, moved, resized or erased by accident; `Ctrl+Shift+K` unlocks every shape.

## Known facts (2026-11-10)

- `Ctrl+K` and `Ctrl+Shift+K` are unbound.
- Shapes have no flags beyond style, fill, label and text; snapshots and the autosave store every shape field ([[Editor snapshots]], [[Crash recovery]]).
- Filling grid cells with the bucket and typing into shape text are how a background DP table is used.

## Spec

Owner decisions are marked (owner); the rest is settled in [[ADR-T107-1 Lock flag kept by the document]] (the document keeps a set of locked ids and a reversible `Edit::Lock`; removal of a locked shape is refused; clear canvas and the eraser leave locked shapes alone; snapshot version 2) and [[ADR-T107-2 Ctrl+K locks and Ctrl+Shift+K unlocks]]. No visible mark (owner).

- **AC-1** (owner) — `Ctrl+K` locks every selected shape and clears the selection; `Ctrl+Shift+K` unlocks every locked shape. Each is one undo step; nothing to lock or unlock records nothing.
  - keymap: `resolve_ctrl_k_locks_and_ctrl_shift_k_unlocks`
  - document: `lock_edit_sets_the_flag_and_its_inverse_clears_it`, `lock_edit_that_changes_nothing_is_a_mismatch`, `lock_edit_on_unknown_id_is_rejected`, `remove_of_locked_shape_is_rejected`, `tx_lock_skips_locked_unknown_and_repeated_ids`, `tx_unlock_all_unlocks_every_locked_shape`, `equality_counts_the_lock_flag`, `lock_edit_heap_bytes_is_the_edit_only`
  - editor: `lock_locks_selection_and_clears_it`, `lock_is_one_undo_step`, `lock_with_empty_selection_records_nothing`, `unlock_all_is_one_undo_step`, `unlock_all_without_locked_shapes_records_nothing`
- **AC-2** — A locked shape is skipped by click and marquee selection, `Ctrl+A`, the eraser, delete, cut and every move, resize, align and restack; copy never sees it because it cannot be selected.
  - document: `topmost_unlocked_where_skips_locked_shapes`, `tx_remove_and_tx_clear_skip_locked_shapes`, `tx_restack_leaves_locked_shapes_in_place`
  - select: `click_skips_locked_shape_and_picks_the_one_below`, `marquee_skips_locked_shapes`, `select_all_skips_locked_shapes`, `prune_drops_locked_ids`
  - eraser: `remove_drag_skips_locked_shapes`, `clear_drag_skips_locked_fills`
  - editor: `selection_commands_never_touch_locked_shapes`, `clear_all_keeps_locked_shapes`, `clear_all_with_only_locked_shapes_records_nothing`, `redo_of_lock_drops_it_from_selection`
- **AC-3** (owner) — A locked shape still takes bucket fills (cells and shapes) and its text or cell text can still be edited, so a locked grid keeps working as a table.
  - editor: `bucket_fills_locked_grid_cell`, `bucket_fills_locked_rect`, `text_tool_types_into_locked_grid_cell`, `emptying_locked_text_block_keeps_it`
- **AC-4** — Snap and smart guides still use locked shapes. The lock flag is kept by snapshots and the autosave (format version bumped).
  - document: `snap_targets_include_locked_shapes`
  - snapshot: `version_is_two`, `snapshot_round_trip_keeps_locks_and_lock_edits`, `snapshot_writes_locked_record_only_when_needed`, `snapshot_rejects_unknown_locked_id`, `snapshot_rejects_repeated_locked_id`, `snapshot_rejects_locked_selection`; the autosave writes the same text (`Snapshot::to_text`).
- **AC-5** — Fuzzed: lock and unlock mixed into editing sessions never let a locked shape move or vanish except through undo.
  - fuzz: `lock_sessions_never_move_or_remove_locked_shapes` (proptest), `lock_sessions_lock_shapes` (coverage), and `check_invariants` refuses a locked id in the selection.

## Out of scope

- Locking one shape at a time from a menu; a visible lock badge (the spec step may add a subtle outline style).
- Locking layers or groups.

## Files owned

- `src/core/shape.rs`, `src/core/document.rs`, `src/core/editor.rs`, `src/core/keymap.rs`, `src/core/command.rs`, `src/core/tools/select.rs`, `src/core/tools/eraser.rs`, `src/core/snapshot.rs`
- `docs/features/Editing tools.md`, `docs/features/Editor snapshots.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, snapshot fixtures
- `src/core/clipboard.rs` (exhaustive `Edit` match), `docs/architecture/Keymap.md` (the documented key table)
- this note, and new `ADR-T107-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T107): specify locking shapes`
- [ ] tests — `test(T107): add failing tests for locking shapes`
- [ ] models — `feat(T107): add the lock flag and commands`
- [ ] behaviour — `feat(T107): skip locked shapes in selection and the eraser`
- [ ] quality — `chore(T107): pass clippy and rustfmt`
- [ ] docs — `docs(T107): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from E8 of the [[Feature proposal]].
- 2026-11-10 — UNATTENDED CHOICE: the lock lives in the document (a set of ids plus `Edit::Lock`), not in each `Shape` variant; clear canvas and the eraser (both modes) leave locked shapes alone (ADR-T107-1).
- 2026-11-10 — UNATTENDED CHOICE: added `src/core/clipboard.rs` (it matches `Edit` exhaustively) and `docs/architecture/Keymap.md` (the documented key table the keymap tests mirror) to *Files owned*.
