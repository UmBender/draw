---
id: T93
title: Coordinate plane from points
status: todo
wave: 54
branch: task/T93-coordinate-plane
depends_on: [T87]
adrs: [ADR-T93-1]
feature: "[[Shapes]]"
tutorial: ""
tags: [task]
---

# T93 Coordinate plane from points

## Goal

A selected text block of points becomes axes with ticks and plotted points. (idea C4 of [[Feature proposal]])

## Known facts (2026-11-09)

- It uses the same select-a-block-and-press-a-key pattern as T84 and T87.
- `Ctrl+P` is unbound.

## Spec

The spec step names the test for each criterion.

- **AC-1** — With one text block of `x y` lines selected, `Ctrl+P` replaces it with two axes, numbered ticks and one small filled circle per point, one undo step.
- **AC-2** — Tick steps are 1, 2 or 5 × 10ⁿ, with both axes on the same scale.
- **AC-3** — Axes are always drawn through the origin when it is in range, and at the edge otherwise.
- **AC-4** — Integers and decimals are accepted, with negatives; non-finite values or text that does not parse leave everything unchanged, with the reason in the diagnostics log.
- **AC-5** — More than 1 000 points: nothing changes. A single point or all-equal coordinates still gives a valid plot.

## Out of scope

- Plotting functions.
- Polygons joined in order.

## Files owned

- `src/core/tools/layout.rs`, `src/core/tools/mod.rs`, `src/core/shape.rs`
- `docs/features/Shapes.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T93-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T93): specify coordinate plane from points`
- [ ] tests — `test(T93): add failing tests`
- [ ] models — `feat(T93): add types`
- [ ] behaviour — `feat(T93): implement`
- [ ] quality — `chore(T93): pass clippy and rustfmt`
- [ ] docs — `docs(T93): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
