---
id: T114
title: Export to SVG
status: todo
wave: 59
branch: task/T114-svg-export
depends_on: []
adrs: []
feature: "[[PNG export]]"
tutorial: ""
tags: [task]
---

# T114 Export to SVG

## Goal

`Ctrl+Shift+E` writes the drawing as `~/draw-<date>-<time>.svg`, a vector image that stays sharp at any size and looks the same as on screen, formulas included, without needing draw's fonts.

## Known facts (2026-11-10)

- `Ctrl+E` exports the view to PNG by reading back the document layer ([[PNG export]], ADR-T92-2); `Ctrl+Shift+E` is unbound.
- Text and formulas are drawn from two embedded fonts (the monospaced text font and the Latin Modern Math subset, ADR-T33-1, ADR-T55-1) with fontdue; an SVG viewer will not have them.
- The proposal's risk: a second rendering path that must match the first.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `Ctrl+Shift+E` writes every shape of the document (not only the view) to `~/draw-<date>-<time>.svg`, with the canvas background; the log and a notice name the file. The write is atomic and off the frame.
- **AC-2** — Every shape kind is exported with its colour, width, fill, label and texts: strokes as smoothed paths, lines, arrows (with heads), rectangles, ellipses, grids (cells, fills, index bands), text blocks and shape texts.
- **AC-3** — Text and formulas are exported as glyph outlines (`<path>`s taken from the embedded fonts), each glyph defined once and reused, so the file looks the same in any viewer and needs no font.
- **AC-4** — A golden test renders sample documents to SVG and compares them with committed files; the SVG parses as XML and every number is finite. An empty document writes a valid empty SVG.
- **AC-5** — Fuzzed: any document exports without panicking.

## Out of scope

- Exporting a selection only, choosing the file name, PDF.
- Selectable text in the SVG.

## Files owned

- a new `src/core/svg.rs`, `src/core/keymap.rs`, `src/core/command.rs`, `src/core/editor.rs`
- `src/shell/export.rs`, `src/shell/glyphs.rs` (glyph outlines)
- `docs/features/PNG export.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/` SVG goldens
- this note, and new `ADR-T114-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T114): specify SVG export`
- [ ] tests — `test(T114): add failing tests for SVG export`
- [ ] models — `feat(T114): write documents as SVG with glyph outlines`
- [ ] behaviour — `feat(T114): export to SVG on Ctrl+Shift+E`
- [ ] quality — `chore(T114): pass clippy and rustfmt`
- [ ] docs — `docs(T114): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from K5 of the [[Feature proposal]].
