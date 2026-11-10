---
id: T109
title: Shape recognition
status: todo
wave: 59
branch: task/T109-shape-recognition
depends_on: []
adrs: []
feature: "[[Drawing tools]]"
tutorial: ""
tags: [task]
---

# T109 Shape recognition

## Goal

Right after drawing a pen stroke, pressing `Z` with the left hand replaces it with the best-fitting straight line, rectangle or ellipse, in one undo step.

## Known facts (2026-11-10)

- Plain `Z` is unbound (`Ctrl+Z` is undo). The proposal had `Q`, now the laser pointer (T106); the owner chose `Z`.
- Pen strokes are already smoothed by anti-tremor ([[Anti-tremor strokes]]); the line, rectangle and ellipse tools exist ([[Drawing tools]]).
- The proposal's risk: a wrong guess costs more than redrawing, so a stroke that fits nothing well must stay as it is.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `Z` acts on the last pen stroke drawn, as long as nothing else was done since (a new stroke, an edit, an undo); otherwise it does nothing.
- **AC-2** — The stroke becomes a `Line` when it is nearly straight, a `Rect` when it is a closed four-cornered loop, an `Ellipse` when it is a closed round loop, with the stroke's colour and width. Thresholds are set in the ADR and tested on drawn samples (shaky circles, squares, lines, and scribbles that must stay strokes).
- **AC-3** — One undo step: undo brings the original stroke back. No fit good enough: nothing changes and no undo step is recorded.
- **AC-4** — Rotated rectangles and ellipses fit their axis-aligned box (draw has no rotation); an open curve is never closed into a shape.
- **AC-5** — Fuzzed: any stroke, including degenerate ones (one point, all points equal, huge coordinates), never panics and gives finite shapes.

## Out of scope

- Triangles, arrows, polygons; recognising while drawing without a key.
- Recognising text or formulas.

## Files owned

- a new `src/core/recognize.rs`, `src/core/editor.rs`, `src/core/keymap.rs`, `src/core/command.rs`
- `docs/features/Drawing tools.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T109-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T109): specify shape recognition`
- [ ] tests — `test(T109): add failing tests for shape recognition`
- [ ] models — `feat(T109): fit lines, rectangles and ellipses to a stroke`
- [ ] behaviour — `feat(T109): replace the last stroke with its shape on Z`
- [ ] quality — `chore(T109): pass clippy and rustfmt`
- [ ] docs — `docs(T109): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from I2 of the [[Feature proposal]].
- Key changed from `Q` to `Z`: `Q` is the laser pointer (T106).
