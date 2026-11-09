---
id: T81
title: Formula box too narrow while editing
status: todo
wave: 34
branch: fix/formula-box-while-editing
depends_on: [T74]
adrs: []
feature: ""
tutorial: ""
tags: [task, text]
---

# T81 Formula box too narrow while editing

## Goal

While a block with a formula is being edited, no outline drawn around it
ends in the middle of its source text.

## Known facts (2026-11-09)

- **Symptom:** editing a block that starts `asdf$f(x) = \frac{a}{b}x \t…`,
  with the caret after `$f`, shows two outlines. One ends in the middle of
  `\frac`, about half way along the source. The other runs on past the
  right edge of the window, with the source.
- Not the bug of [[T78 Formula ink outside its text box]]: that one was a
  rounding drift in very long text without markup. This text has markup
  and is short, and the overflow is hundreds of pixels.
- `TextEdit::bounds` (`src/core/tools/text.rs`) uses
  `text_layout::source_bounds`, and `shell::app::draw_overlay` outlines it
  with `render::draw_selection`. That is the box of the source as typed
  ([[ADR-T30-1 TeX subset, box layout and metrics]]), so it is the long
  outline and it is right.
- `text_layout::bounds` is the box of the *typeset* formula, which is much
  narrower than its source.

## Cause (not confirmed)

The short outline is probably the block's `text_layout::bounds` drawn
during the edit, where the source is on screen instead of the formula.
What draws it has not been found. Start from whatever outlines a selected
block, and from how the block being edited is kept out of the selection.

## Spec

To be written in the spec step. Expected:

- A regression case with the state above, red before the fix.
- While a block is edited, every outline around it fits the source shown.
- Once the edit ends, the block's box is again the typeset one.

## Out of scope

- The size of the typeset box itself.
- The drift of long text inside a block with markup, which
  [[T78 Formula ink outside its text box]] left unchanged.

## Files owned

To be set in the spec step, once the cause is known.

## Subtasks (one commit each)

- [ ] spec — `docs(T81): specify the box of a formula being edited`
- [ ] regression — `test(T81): add regression test for the box while editing`
- [ ] behaviour — `fix(T81): keep the box of a formula on its source while editing`
- [ ] quality — `chore(T81): pass clippy and rustfmt`
- [ ] docs — `docs(T81): record the fix and close the task`

## Learning path

None planned.

## Log

- 2026-11-09 — created from a screenshot by the owner, while reviewing
  T78. Not reproduced yet.
