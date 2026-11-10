---
id: T100
title: Z-order keys
status: todo
wave: 57
branch: task/T100-z-order-keys
depends_on: []
adrs: []
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

The spec step names the test functions of each AC.

- **AC-1** (owner) — Outside text edit mode, `Ctrl+]` raises the selection one step and `Ctrl+[` lowers it; in text edit mode they still resize text.
- **AC-2** — One step passes the nearest unselected shape above (or below) whose bounds overlap the selection's; with none, nothing changes and no undo step is added.
- **AC-3** — `Ctrl+Shift+]` / `Ctrl+Shift+[` move the selection to the top / bottom. Selected shapes keep their order among themselves.
- **AC-4** — Each press is one undo step; undo restores the order exactly. The selection stays selected. Nothing happens during a gesture.
- **AC-5** — The new commands are fuzzed; the document order stays a permutation (no shape lost or doubled).

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

- [ ] spec — `docs(T100): specify z-order keys`
- [ ] tests — `test(T100): add failing tests for z-order keys`
- [ ] models — `feat(T100): …`
- [ ] behaviour — `feat(T100): …`
- [ ] quality — `chore(T100): pass clippy and rustfmt`
- [ ] docs — `docs(T100): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
