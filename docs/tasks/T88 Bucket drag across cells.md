---
id: T88
title: Bucket drag across cells
status: todo
wave: 51
branch: task/T88-bucket-drag
depends_on: []
adrs: []
feature: "[[Cell fill]]"
tutorial: ""
tags: [task]
---

# T88 Bucket drag across cells

## Goal

Dragging the bucket across a grid fills every cell it passes over, in one undo step. (idea E3 of [[Feature proposal]])

## Known facts (2026-11-09)

- `core::tools::bucket::on_pointer` acts on the press only.
- In pen mode a tap is delivered as a click ([[ADR-T34-1 Tap slop for pen tablets detected at start-up]]).

## Spec

The spec step names the test for each criterion.

- **AC-1** — Pressing in a grid cell and dragging fills each cell passed over with the current colour, staying in that grid.
- **AC-2** — A fast drag leaves no gaps: the segment between two moves is walked cell by cell.
- **AC-3** — The whole drag is one undo step, and no step at all if no cell changed.
- **AC-4** — `Esc` during the drag restores the cells.
- **AC-5** — A press on a rectangle, ellipse or empty space behaves as today.

## Out of scope

- Filling enclosed regions.
- Clearing fills with the bucket.

## Files owned

- `src/core/tools/bucket.rs`
- `src/core/document.rs`
- `docs/features/Cell fill.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T88-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T88): specify bucket drag across cells`
- [ ] tests — `test(T88): add failing tests`
- [ ] models — `feat(T88): add types`
- [ ] behaviour — `feat(T88): implement`
- [ ] quality — `chore(T88): pass clippy and rustfmt`
- [ ] docs — `docs(T88): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
