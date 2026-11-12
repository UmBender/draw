---
id: T136
title: Automaton gestures and text editing per tool
status: todo
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

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `Gesture` has one variant per tool with its own states (`Pen: Down -> Drawing -> Commit`, `Select: Idle -> Marquee | Moving | Resizing(Handle)`, `Text: Placing -> Editing`).
  Tests: the tool tests of `core/tools/**` pass unchanged.
- **AC-2** — A gesture ends in a commit or a cancel, never in a third state: `gesture_always_ends` (proptest over event sequences).
- **AC-3** — The undo history sees exactly one transaction per committed gesture.
  Tests: the history tests and `undo_redo_symmetry` fuzz.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/machine/gesture/**`, `crates/draw-core/src/tools/**`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T136): specify the gesture automata`
- [ ] tests — `test(T136): add failing tests for the gestures`
- [ ] behaviour — `refactor(T136): move the gestures into the machine`
- [ ] quality — `chore(T136): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
