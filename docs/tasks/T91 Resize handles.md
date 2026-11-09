---
id: T91
title: Resize handles
status: todo
wave: 51
branch: task/T91-resize-handles
depends_on: []
adrs: [ADR-T91-1]
feature: "[[Editing tools]]"
tutorial: ""
tags: [task]
---

# T91 Resize handles

## Goal

A selection can be resized by dragging a corner or with the keyboard. (idea E4 of [[Feature proposal]])

## Known facts (2026-11-09)

- Moving is the only transform today.
- Selection moves were made linear in [[T25 Linear-time selection with large documents]], and the overlay budget is under 2 ms.

## Spec

The spec step names the test for each criterion.

- **AC-1** — With a selection, four corner handles show at a constant screen size; dragging one scales about the opposite corner, one undo step.
- **AC-2** — `Alt+[` and `Alt+]` scale the selection by 10 % about its centre, one undo step each.
- **AC-3** — `Shift` while dragging keeps the proportions.
- **AC-4** — Every shape kind scales: points of strokes, lines and arrows, boxes of rectangles, ellipses and grids, and the size of text. Stroke widths do not change.
- **AC-5** — A drag cannot flip a shape, or make it smaller than 4 world units; non-finite scales are ignored.
- **AC-6** — Handles take priority over move only on the handle itself; `Alt`-drag duplicate still works.

## Out of scope

- Rotation.
- Edge-only handles.
- Changing a grid's cell count (T85).

## Files owned

- `src/core/tools/select.rs`, `src/core/shape.rs`, `src/core/document.rs`, `src/core/history.rs`
- `src/shell/render.rs`
- `docs/features/Editing tools.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T91-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T91): specify resize handles`
- [ ] tests — `test(T91): add failing tests`
- [ ] models — `feat(T91): add types`
- [ ] behaviour — `feat(T91): implement`
- [ ] quality — `chore(T91): pass clippy and rustfmt`
- [ ] docs — `docs(T91): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
