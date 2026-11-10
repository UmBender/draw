---
id: T107
title: Lock shapes
status: todo
wave: 59
branch: task/T107-lock
depends_on: []
adrs: []
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

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `Ctrl+K` locks every selected shape and clears the selection; `Ctrl+Shift+K` unlocks every locked shape. Each is one undo step; nothing to lock or unlock records nothing.
- **AC-2** — A locked shape is skipped by click and marquee selection, `Ctrl+A`, the eraser, delete, cut and every move, resize, align and restack; copy never sees it because it cannot be selected.
- **AC-3** (owner) — A locked shape still takes bucket fills (cells and shapes) and its text or cell text can still be edited, so a locked grid keeps working as a table.
- **AC-4** — Snap and smart guides still use locked shapes. The lock flag is kept by snapshots and the autosave (format version bumped).
- **AC-5** — Fuzzed: lock and unlock mixed into editing sessions never let a locked shape move or vanish except through undo.

## Out of scope

- Locking one shape at a time from a menu; a visible lock badge (the spec step may add a subtle outline style).
- Locking layers or groups.

## Files owned

- `src/core/shape.rs`, `src/core/document.rs`, `src/core/editor.rs`, `src/core/keymap.rs`, `src/core/command.rs`, `src/core/tools/select.rs`, `src/core/tools/eraser.rs`, `src/core/snapshot.rs`
- `docs/features/Editing tools.md`, `docs/features/Editor snapshots.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, snapshot fixtures
- this note, and new `ADR-T107-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T107): specify locking shapes`
- [ ] tests — `test(T107): add failing tests for locking shapes`
- [ ] models — `feat(T107): add the lock flag and commands`
- [ ] behaviour — `feat(T107): skip locked shapes in selection and the eraser`
- [ ] quality — `chore(T107): pass clippy and rustfmt`
- [ ] docs — `docs(T107): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from E8 of the [[Feature proposal]].
