---
id: T78
title: Formula ink outside its text box
status: in-progress
wave: 34
branch: fix/formula-ink-outside-box
depends_on: [T74]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T78 Formula ink outside its text box

## Goal

Every glyph and rule a text block draws stays inside the box
`Shape::bounds` reports for it, so selection, hit testing and redraw
regions cover all of its ink, and `scripts/check.sh --fuzz` stops failing
at random on this invariant.

## Known facts (2026-11-08)

- **Symptom:** the three `*undo_all_then_redo_all_is_symmetric`
  properties fail with `ink Glyph { ch: 'a', pos: (250.8, 1.0) }` outside
  the block's box, `max.y` 22.5 (`tests/fuzz.rs`, the T30 ink invariant in
  the text-shape check). Found by the first `--fuzz` run of
  [[T76 Remembered text size past the ladder]].
- **Not caused by T76:** it fails the same with the clamp removed. It
  needs a long formula session, and only shows when the random inputs hit
  it, so a green `--fuzz` run does not mean it is gone.
- **Seed:** `cc 1bf62a4f742068dd71bc2aa699366205a90af4d8b775a11fcd964bfa8086e409`,
  in full in the stash `batch-T76-2026-11-08 fuzz seed: text ink outside
  its box` (one line of `tests/fuzz.proptest-regressions`). Find it by its
  message; stash numbers move. It was left uncommitted so the gates stay
  green until this task.
- **Where to look:** `text_layout::bounds` (`src/core/text_layout.rs`),
  `typeset`, and `Layout::bounds` in `src/core/tex.rs`. The ink's `y` of
  1.0 line against a box ending at 22.5 suggests the box is short by a
  line or a formula's height. **The cause is not known yet.** The fuzzer's
  formula tokens grew a lot in T56–T71, so one of those commands may be
  what exposes it.

## Cause (found 2026-11-08)

The seed's block holds **no formula**: it is 419 characters of formula
*source* with no `$`, so it is plain text.

- Such a block is drawn from `text_layout::glyphs` (`col × ADVANCE`) and
  its box is `source_bounds` (`cols × ADVANCE`). The drawn ink is inside
  the box; nothing visible is wrong.
- `text_layout::typeset`, which the fuzz invariant reads, runs the same
  text through `tex::layout`, whose `Line::cell` *adds* `ADVANCE` once per
  character. In `f32` that drifts: at column 418 the cell is at
  250.80103 em instead of 250.8, past the box by more than the invariant's
  `1e-3` em slack. The `y` of 1.0 in the report is the correct baseline.
- With a `$` the box comes from the same sums as the ink, so they agree.

The fix makes `typeset` of text without markup lay out on the cell grid
the renderer uses. The box's definition does not change and no drawn glyph
or rule moves (text with `$` still goes through `tex::layout` untouched),
so no ADR is needed.

## Spec

- **AC-1** — The state from the seed has no typeset ink outside its box.
  *Test:* `core::editor::tests::regression_t78_formula_ink_outside_box`
  (lower-case id: `non_snake_case` fails under `-D warnings`), from
  `tests/snapshots/regressions/T78-formula-ink-outside-box.snap`.
- **AC-2** — The smallest case, a long line without markup, typesets on
  the cells it is drawn at and inside its box.
  *Test:* `core::text_layout::tests::typeset_of_long_plain_text_stays_on_the_cell_grid`,
  `core::text_layout::tests::typeset_of_plain_text_has_the_cell_box`.
- **AC-3** — Text with markup is laid out exactly as before: no glyph or
  rule moves. *Test:*
  `core::text_layout::tests::typeset_of_markup_is_the_tex_layout`, and
  `scripts/e2e.sh` with no golden changed.
- **AC-4** — The seed is committed in `tests/fuzz.proptest-regressions`
  and `scripts/check.sh --fuzz` is green with it. *Test:* the `tests/fuzz.rs`
  properties.

## Out of scope

- Widening the fuzz invariant's slack to make the failure go away.
- Changing how formulas look, beyond what the fix needs.

## Files owned

- `src/core/text_layout.rs`, `src/core/tex.rs`
- `src/core/editor.rs` (test only)
- `tests/fuzz.proptest-regressions`
- `tests/snapshots/regressions/T78-formula-ink-outside-box.snap`
- `docs/decisions/` (a new ADR only if the box's definition changes)

## Subtasks (one commit each)

- [ ] spec — `docs(T78): specify the formula ink box fix`
- [ ] regression — `test(T78): add regression test for formula ink outside its box`
- [ ] behaviour — `fix(T78): keep formula ink inside the text box`
- [ ] quality — `chore(T78): pass clippy and rustfmt`
- [ ] docs — `docs(T78): record the fix and close the task`

## Learning path

None planned.

## Log

- 2026-11-08 — created from the `--fuzz` failure logged in T76. Drop the
  seed's stash only once its line is committed.
