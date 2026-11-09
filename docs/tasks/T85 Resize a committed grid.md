---
id: T85
title: Resize a committed grid
status: todo
wave: 51
branch: task/T85-resize-grid
depends_on: []
adrs: [ADR-T85-1]
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

The spec step names the test for each criterion.

- **AC-1** — With the Select tool and exactly one grid selected, `→`/`←` and `↓`/`↑` add or remove a column or row, clamped to 1–64, and `helpers` is unchanged.
- **AC-2** — Cell size stays; the box grows or shrinks at the right and bottom edges.
- **AC-3** — Text and fills of remaining cells stay; those of a removed cell go, and undo restores them.
- **AC-4** — Each press that changes the grid is one undo step; a press at the limit adds none.
- **AC-5** — With anything else selected the arrows behave as they do today.

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

- [ ] spec — `docs(T85): specify resize a committed grid`
- [ ] tests — `test(T85): add failing tests`
- [ ] models — `feat(T85): add types`
- [ ] behaviour — `feat(T85): implement`
- [ ] quality — `chore(T85): pass clippy and rustfmt`
- [ ] docs — `docs(T85): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
