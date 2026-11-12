---
id: T136
title: Automaton gestures and text editing per tool
status: in-progress
wave: 67
branch: task/T136-automaton-gestures-and-text-editing-per-
depends_on: [T133, T134]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T136 Automaton gestures and text editing per tool

## Goal

Each tool's gesture is its own sub-automaton, so a half-finished gesture cannot meet a command that does not expect it.

## Known facts (2026-11-12)

- Gestures today: pen, line, arrow, rectangle, ellipse, grid, text, eraser, bucket, select (marquee, move, resize), hand, laser.

## Spec

See [[ADR-T136-1 One gesture variant per tool in the machine]].

- **AC-1** — `Gesture` (in `machine/gesture`) has one variant per tool, each with a `Stage` (`Down -> Active`).
  Tests: `gesture_for_tool_has_one_variant_per_tool`, `gesture_tool_roundtrips_for_every_tool`, `gesture_stage_moves_from_down_to_active`, `gesture_yields_only_when_down_and_left`; the tool tests of `core/tools/**` pass unchanged.
- **AC-2** — A gesture ends in a commit or a cancel, never in a third state: `gesture_always_ends` (proptest over event sequences ending with the release of every button), `gesture_end_commit_and_cancel_clear_the_gesture`.
- **AC-3** — The undo history sees exactly one transaction per committed gesture: `one_undo_per_committed_gesture` (every creation tool), `cancelled_gesture_leaves_no_transaction`; the history tests and `undo_redo_symmetry` fuzz.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/machine/gesture/**`, `crates/draw-core/src/core/tools/**`, `crates/draw-core/src/core/editor.rs`
- this note, `ADR-T136-1`

## Subtasks (one commit each)

- [ ] spec — `docs(T136): specify the gesture automata`
- [ ] tests — `test(T136): add failing tests for the gestures`
- [ ] behaviour — `refactor(T136): move the gestures into the machine`
- [ ] quality — `chore(T136): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — BLOCKED (batch): the gesture state (`Gesture`, `Editor.gesture`, press/move/release handling) is private to `crates/draw-core/src/core/editor.rs`, which is not in *Files owned*; the owned path `crates/draw-core/src/tools/**` also does not exist (the code is in `core/tools/**`). Moving the gestures into `machine/gesture/` needs edits to `editor.rs`. Recommended fix: add `crates/draw-core/src/core/editor.rs` and `crates/draw-core/src/core/tools/**` to *Files owned* (as T135 did), then re-run T136.
- 2026-11-12 — owner decision (triage): Files owned amended to `crates/draw-core/src/core/tools/**`, `core/editor.rs` and `machine/gesture/**` (the `tools/**` path does not exist); the blocker above is resolved. Spec: ADR-T136-1.
