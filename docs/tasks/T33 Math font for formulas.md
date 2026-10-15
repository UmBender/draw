---
id: T33
title: Math font for formulas
status: ready
wave: 18
branch: task/T33-math-font
depends_on: [T30]
adrs: []
feature: "[[TeX formulas]]"
tutorial: "[[33 Typesetting with a proportional math font]]"
tags: [task]
---

# T33 Math font for formulas

## Goal

Typeset `$…$` formulas in **Latin Modern Math** — the look of LaTeX —
instead of the monospaced text font: italic serif variables, upright
digits, operators and function names, proportional spacing, and
calligraphic capitals with `\mathcal{…}`. Plain text and the source shown
while a block is edited stay in the monospaced font. Formulas become
easier to read (`x` vs `×`, `l` vs `1`, `\min` vs `min`) and look like
the notes they are copied from.

## Proposal (settle in the spec step)

- **Font.** Embed a subset of Latin Modern Math 1.959 (GUST Font
  License): ASCII, Greek, the operators and arrows of the T30 table, math
  italic Latin and Greek (`U+1D434–1D467`, `U+1D6A8–1D71B`, `ℎ` U+210E)
  and script capitals (`U+1D49C–1D4B5` and their letterlike holes
  `ℬ ℰ ℱ ℋ ℐ ℒ ℳ ℛ`). About 110 KB. The subset gets its own file name
  (the GFL asks derived files to be renamed); `assets/fonts/README.md`
  records the command, the license file is added.
- **Faces.** A positioned glyph carries a face: `Text` (monospaced, as
  now) or `Math`. In a formula, Latin and lowercase Greek letters map to
  math italic codepoints, `\mathcal{A}` to script capitals, everything
  else (digits, operators, brackets, function names, `\text{…}`) is
  upright from the math font. Literal (warning) nodes stay `Text`.
- **Metrics in `core`.** `core` stays font-free: a generated table of
  per-glyph advance, ascent and descent (em) for every math glyph the
  layout can emit, with a shell test proving it matches the embedded
  font. The box layout uses these instead of the 0.6 em cell; the T30
  constants (script shifts, gaps, axis, rule) are retuned for the new
  font, with italic correction where a superscript would collide.
- **Rendering.** The glyph atlas is keyed by face as well as char and
  size; both fonts share the atlas pages and the one textured batch of
  ADR-T32-2.
- **Budgets.** The T30 perf tests keep their thresholds; the fuzz
  invariants (bounds contain ink, total parse and layout) hold.

## Spec

To be written in the spec step: ADR `ADR-T33-1` (font, faces, metrics
table, atlas key) and acceptance criteria naming their tests.

## Out of scope

- Display mode, accents, matrices and the rest of T30's out-of-scope
  list.
- `\mathbb`, `\mathfrak`, bold math — the subset can grow later.
- Using the font's OpenType `MATH` table at runtime (the constants are
  read once and written into `core`; the table is dropped from the
  subset).
- Changing the plain text font.

## Files owned

- `src/core/tex.rs`, `src/core/text_layout.rs`, a new
  `src/core/math_metrics.rs` (generated table), `src/core/mod.rs`
- `src/shell/glyphs.rs`, `src/shell/render.rs`, `src/shell/app.rs`
- `assets/fonts/*`, `scripts/` (metrics generator, if any)
- `tests/fuzz.rs`, `tests/perf.rs`, `README.md`
- new ADRs `ADR-T33-*`, feature note `docs/features/TeX formulas.md`
  (update), tutorial
  `docs/tutorials/33 Typesetting with a proportional math font.md`

## Subtasks (one commit each)

- [ ] spec — `docs(T33): specify math font acceptance criteria`
- [ ] tests — `test(T33): add failing tests for math faces, metrics and rendering`
- [ ] models — `feat(T33): add glyph faces and the math metrics table`
- [ ] behaviour — `feat(T33): typeset formulas in Latin Modern Math`
- [ ] quality — `chore(T33): pass clippy, rustfmt and rustdoc`
- [ ] docs — `docs(T33): update feature note and add tutorial`

## Learning path

Step 33 — requires step 30 (TeX formulas) and step 32 (glyph atlas).

## Log

- 2026-10-15 — Created from the owner's request for a calligraphic,
  LaTeX-like formula font; the owner chose Latin Modern Math over STIX
  Two and New Computer Modern. Checked before writing the note: fontdue
  loads and rasterizes the CFF subset (math italic, script capitals,
  `∑`, `√`). Runs in wave 18 before T31, which also owns
  `src/shell/render.rs` and `src/core/text_layout.rs`; later waves shift
  by one.
