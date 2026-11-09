---
id: T86
title: Zero-based numbering
status: todo
wave: 51
branch: task/T86-zero-based
depends_on: []
adrs: [ADR-T86-1]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T86 Zero-based numbering

## Goal

Node numbers can start at 0, to match 0-indexed code. (idea E2 of [[Feature proposal]])

## Known facts (2026-11-09)

- `core::numbering` has `FIRST_NUMBER` and `roll_back`, which never goes below it.
- `Shift+N` restarts at 1; grid indices are already 0-based.
- `Ctrl+N` is unbound.

## Spec

The spec step names the test for each criterion.

- **AC-1** — `Ctrl+N` switches the first number between 1 and 0 and restarts the counter there.
- **AC-2** — `Shift+N` restarts at the current first number.
- **AC-3** — Undo never rolls the counter below the first number.
- **AC-4** — The setting survives a snapshot round trip.
- **AC-5** — Number 0 draws and hit-tests like any other label.

## Out of scope

- Letters or any other start value.
- Editing labels.

## Files owned

- `src/core/numbering.rs`, `src/core/snapshot.rs`, `src/core/shape.rs`
- `docs/features/Numbered nodes.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T86-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T86): specify zero-based numbering`
- [ ] tests — `test(T86): add failing tests`
- [ ] models — `feat(T86): add types`
- [ ] behaviour — `feat(T86): implement`
- [ ] quality — `chore(T86): pass clippy and rustfmt`
- [ ] docs — `docs(T86): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
