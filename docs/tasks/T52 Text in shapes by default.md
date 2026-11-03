---
id: T52
title: Text in shapes by default
status: todo
wave: 32
branch: task/T52-shape-text-default
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task]
---

# T52 Text in shapes by default

## Goal

Writing inside rectangles, ellipses (circles) and grid cells is a normal
feature, on without `DRAW_EXPERIMENTAL=shape-text`. The one exception:
a **numbered** rectangle or ellipse (one with a label from the numbering
helper, [[T19 Auto-numbering]]) cannot hold text. Its number is its
content. Grids, including numbered grids ([[T22 Numbering drives grid
indices]]), keep text in their cells.

## Known facts (2026-11-02)

- The feature came from [[T31 Text in rectangles and grid cells]]
  behind the switch of [[ADR-T31-1 Shape text prototype behind a
  switch]]: `core::experimental::{Experimental, SHAPE_TEXT,
  EXPERIMENTAL_ENV}`, read once in `shell::app::run` and carried in
  `Helpers::experimental`.
- Code that checks the switch: `core::editor`, `core::tools::{text,
  select}`, `core::shape`, `shell::render`, `shell::app`, and
  `tests/fuzz.rs` (which fuzzes both settings).
- `Shape::Rect` and `Shape::Ellipse` have `label: Option<u32>` and
  `text: Option<ShapeText>`. Today the text is "drawn centred inside
  *instead of* the label", so a numbered shape with text hides its
  number.
- Feature note: [[Text in shapes (experimental)]], tutorial step 31.
  The README lists `shape-text` under `DRAW_EXPERIMENTAL`.
- There is no file format, so no saved documents have to keep working.

## Proposal (settle in the spec step)

1. **Promote**: a new ADR supersedes ADR-T31-1 (`supersedes:
   [ADR-T31-1]`, which becomes `superseded`). Text in shapes is always
   on.
2. **Remove the switch**: decide whether `core::experimental` stays as
   an empty, reusable mechanism (`DRAW_EXPERIMENTAL` accepted but
   naming nothing), or goes away. Since 2.0.0 adds no features
   ([[T50 Version 2.0.0 refactor]]), removing it is the likely choice,
   with `DRAW_EXPERIMENTAL=shape-text` ignored silently or with a
   one-line notice for one release.
3. **Numbered shapes refuse text**, covering every way in:
   - opening the text editor on a numbered rect/ellipse (double click,
     typing with it selected, the Text tool clicking inside it) does
     nothing there; the Text tool creates a free text block instead,
     as on empty canvas;
   - pasting or duplicating keeps the rule (a numbered copy has no
     text);
   - numbering a rect/ellipse that already holds text: the spec
     decides between leaving it unnumbered (text wins, the counter
     does not advance) and refusing silently. The rule: text is never
     lost without an undo step.
   - The invariant `label.is_some() ⇒ text.is_none()` for Rect/Ellipse
     is asserted in the fuzz harness after every event.
4. **Docs**: the feature note loses "(experimental)" (rename and link
   updates), the README moves it from the experimental list to the
   features, and the cheat sheet lists how to write in a shape.

## Spec

To be written in the spec step (ADR required: superseding ADR-T31-1,
the numbered-shape rule).

## Out of scope

- New text features (alignment, wrapping changes, text on lines).
- Text in numbered grids' index labels.

## Files owned

- `src/core/experimental.rs`, `src/core/mod.rs`, `src/core/editor.rs`,
  `src/core/shape.rs`, `src/core/tools/text.rs`,
  `src/core/tools/select.rs`, `src/core/numbering.rs`
- `src/shell/app.rs`, `src/shell/render.rs`
- `tests/fuzz.rs`
- `README.md`, `docs/features/Text in shapes (experimental).md`
  (renamed), notes linking to it
- new ADR `ADR-T52-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T52): …`
- [ ] tests — `test(T52): …`
- [ ] models — `feat(T52): …`
- [ ] behaviour — `feat(T52): …`
- [ ] quality — `chore(T52): …`
- [ ] docs — `docs(T52): …`

## Learning path

Extends step 31 (*Prototyping an interaction behind a switch*) with how
a prototype graduates: removing a switch and keeping an invariant.

## Log

- 2026-11-02 — note written at the owner's request; ships in 1.5.0
  ([[T53 Release 1.5.0]]).
