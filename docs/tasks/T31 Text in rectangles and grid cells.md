---
id: T31
title: Text in rectangles and grid cells
status: ready
wave: 19
branch: task/T31-shape-text
depends_on: [T29]
adrs: []
feature: "[[Text in shapes (experimental)]]"
tutorial: "[[31 Prototyping an interaction behind a switch]]"
tags: [task, experimental]
---

# T31 Text in rectangles and grid cells

## Goal

An **experimental** prototype for typing text inside rectangles,
ellipses and grid cells — e.g. filling a DP table cell by cell, or
naming a node — to find out whether it can be fast to use without
getting in the way. It is off by default and turned on with
`DRAW_EXPERIMENTAL=shape-text`. The result is a decision, recorded in an
ADR: promote it (follow-up task to make it default), rework it, or
remove it (rollback ADR + `git revert`, per [[Decision Protocol]]).

## Open questions (owner decides in the spec step)

1. **How to start typing.** Double-click a shape or cell with any tool?
   `Enter` on a selected shape? Text tool click inside a shape (T29's
   tool, so no new key)? Typing while hovering is rejected up front:
   it would turn every shortcut into a misfire.
2. **Moving between cells.** `Tab` / `Shift+Tab` and arrows move to the
   next cell while editing a grid, spreadsheet-style? Arrows already
   resize grids (T18) — only outside edit mode, but is that confusing?
3. **Numbers vs text.** A numbered shape (T19) has a label. Does text
   replace the number, sit beside it, or is a typed number just text
   (and numbering only the default content)? Same for grid indices
   (T22).
4. **Fit.** Shrink the font to fit the shape (like labels today), keep
   the block size and clip, or grow the shape?
5. **Attachment.** Text moves, copies and is erased with its shape;
   does the eraser on text clear only the text (like the fill eraser of
   T21)?
6. **Formulas.** If [[T30 Basic TeX formulas]] is done, `$…$` works in
   cells for free — keep or disable?

## Spec

To be written after the questions above; every AC names its tests.
ADRs expected: **ADR-T31-1 — Shape text prototype** (chosen
interaction, switch, what is measured) and, at the end,
**ADR-T31-2 — Verdict** (promote / rework / remove).

- **AC-1 — Off by default.** Without the variable nothing changes:
  same keys, same shapes, same rendering; the switch is a pure function
  over the environment, unit-tested. Documents never contain shape text
  unless the switch is on.
- **AC-2 — Type into shapes and cells.** With the switch on, the chosen
  interaction (Q1) opens edit mode on a rectangle, ellipse or grid cell,
  reusing T29's edit mode (same keys, same undo rule: one entry per
  finished edit).
- **AC-3 — Grid flow.** The chosen cell movement (Q2) fills a 5 × 5 grid
  without the mouse; undo restores cells one edit at a time.
- **AC-4 — Shape behaviour.** Move, copy, duplicate, erase and
  numbering behave as decided in Q3–Q5; fuzz harness covers shape text
  with the switch on.
- **AC-5 — Trial.** Owner on the target: fill a 5 × 5 DP table and label
  a 6-node graph, with and without the feature (T29 free text only);
  note times and every misfire or annoyance in the *Log*.
- **AC-6 — Verdict.** ADR-T31-2 records the decision from AC-5. At
  integration the integrator adds it under `[Unreleased]` → `Added` in
  `CHANGELOG.md`, marked `(experimental)`; if it is later removed, the
  rollback adds a `Removed` line.

## Out of scope

- Making it default (a follow-up task if promoted).
- Text in lines, arrows and strokes.
- Spreadsheet features (formulas over cells, copy a column, etc.).

## Files owned

- `src/core/shape.rs`, `src/core/editor.rs`, `src/core/tools/text.rs`,
  `src/core/tools/grid.rs`, `src/core/tools/select.rs`,
  `src/core/tools/eraser.rs`, `src/core/numbering.rs`,
  `src/core/text_layout.rs`, `src/core/experimental.rs` (new)
- `src/shell/render.rs`, `src/shell/app.rs` (reading the switch)
- `tests/fuzz.rs`, `README.md` (*Experimental* section)
- new ADRs `ADR-T31-*`, feature note
  `docs/features/Text in shapes (experimental).md`, tutorial
  `docs/tutorials/31 Prototyping an interaction behind a switch.md`

## Subtasks (one commit each)

- [ ] spec — `docs(T31): …`
- [ ] tests — `test(T31): …`
- [ ] models — `feat(T31): …`
- [ ] behaviour — `feat(T31): …`
- [ ] quality — `chore(T31): …`
- [ ] docs — `docs(T31): …`

## Learning path

Step 31 — requires step 29 (text blocks), step 18 (grid tool) and
step 19 (numbering).

## Log

- 2026-10-14 — Created as experimental at the owner's request; the
  interaction is still open (questions above). Depends only on T29 but
  runs after T30, which owns the same text layout and render files.
