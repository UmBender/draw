---
id: T84
title: Grid from text
status: todo
wave: 52
branch: task/T84-grid-from-text
depends_on: [T83]
adrs: [ADR-T84-1]
feature: "[[Grid tool]]"
tutorial: ""
tags: [task]
---

# T84 Grid from text

## Goal

A selected text block becomes a grid in one key: lines are rows, whitespace separates cells. The same works straight from the clipboard, so a DP table or sample matrix appears at once. (idea C1 of [[Feature proposal]])

## Known facts (2026-11-09)

- `Shape::Grid` has `texts: Vec<CellText>`, sorted, and at most 64 columns and rows (`src/core/shape.rs`).
- New keys need an ADR ([[Keymap]]). `Ctrl+T` and `Ctrl+Shift+V` are unbound outside text edit.

## Spec

The spec step names the test for each criterion.

- **AC-1** — With exactly one text block selected, `Ctrl+T` replaces it with a grid at the block's top-left, one undo step.
- **AC-2** — Lines are rows and runs of whitespace separate cells; blank lines are ignored; short rows leave empty cells.
- **AC-3** — One line gives a 1 × n strip. Axis indices follow the `N` toggle, as for any new grid.
- **AC-4** — `Ctrl+Shift+V` builds the same grid from the clipboard at the cursor, using the event from [[T83 Paste text from the clipboard]].
- **AC-5** — More than 64 columns or rows: nothing changes (no truncation of the input) and the reason goes to the diagnostics log.
- **AC-6** — With no text block selected, or only blank text, the key does nothing and adds no undo step.

## Out of scope

- Formulas that contain spaces are split like any text.
- Editing a grid as text again.

## Files owned

- `src/core/tools/grid.rs`, `src/core/shape.rs`
- `src/shell/input_map.rs`, `src/shell/app.rs`
- `docs/features/Grid tool.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T84-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T84): specify grid from text`
- [ ] tests — `test(T84): add failing tests`
- [ ] models — `feat(T84): add types`
- [ ] behaviour — `feat(T84): implement`
- [ ] quality — `chore(T84): pass clippy and rustfmt`
- [ ] docs — `docs(T84): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
