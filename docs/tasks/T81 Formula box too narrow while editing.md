---
id: T81
title: Formula box too narrow while editing
status: review
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

## Cause (found 2026-11-09)

Confirmed by reading the code; the regression test below shows it.

- A click with the Select tool on the block that is the only selection
  re-opens it (`Editor::reopen_clicked_text`) and leaves it selected.
- `shell::app::draw_overlay` outlines `edit.bounds()`, the source box, and
  then `editor.selection_bounds()`, which it skips only during a gesture.
  An open text block is not a gesture.
- `selection_bounds` uses `Shape::bounds`, the typeset box. That is the
  short outline.

The block stays selected on purpose: when the edit ends, it is the
selection again. So the fix leaves `selection` alone and makes
`selection_bounds` skip the block being edited, whose outline is the edit
session's. A shape hosting the text is not skipped: its outline is its
frame, which the text is fitted into. No module boundary or decision
changes, so no ADR is needed.

## Spec

- **AC-1** — Re-opening a selected block with a formula leaves no outline
  of its typeset box: `selection_bounds` is `None` and the edit's box is
  wider than the typeset one. *Test:*
  `core::editor::tests::regression_t81_formula_box_while_editing`
  (lower-case id, as in T76 and T78), from
  `tests/snapshots/regressions/T81-formula-box-while-editing.snap`.
- **AC-2** — Once the edit ends, the block is still selected and its box
  is the typeset one. *Test:*
  `core::editor::tests::selection_bounds_after_text_edit_is_the_typeset_box`.
- **AC-3** — The other selected shapes keep their outline while one block
  is edited. *Test:*
  `core::editor::tests::selection_bounds_skips_only_the_block_being_edited`.
- **AC-4** — A shape whose text is edited keeps its outline. *Test:*
  `core::editor::tests::selection_bounds_keeps_the_host_of_the_text_being_edited`.

## Out of scope

- The size of the typeset box itself.
- The drift of long text inside a block with markup, which
  [[T78 Formula ink outside its text box]] left unchanged.

## Files owned

- `src/core/editor.rs`
- `tests/snapshots/regressions/T81-formula-box-while-editing.snap`

## Subtasks (one commit each)

- [x] spec — `docs(T81): specify the box of a formula being edited`
- [x] regression — `test(T81): add regression test for the box while editing`
- [x] behaviour — `fix(T81): keep the box of a formula on its source while editing`
- [x] quality — `chore(T81): pass clippy and rustfmt`
- [x] docs — `docs(T81): record the fix and close the task`

## Learning path

None planned.

## Log

- 2026-11-09 — created from a screenshot by the owner, while reviewing
  T78. Not reproduced yet.
- 2026-11-09 — cause found by reading the code (see *Cause*). Reproduced
  in core: the regression test was red on `selection_bounds`, which still
  gave the typeset box. Fixed there; no change in `shell`.
- 2026-11-09 — no models step: the fix adds no types. No quality commit:
  `scripts/check.sh` was green with no change. `scripts/e2e.sh` is green
  with no golden changed.
- No feature or tutorial note: the task has neither, it only removes a
  wrong outline. The two outlines have not been checked by eye in the
  running app; no e2e case covers this state.
