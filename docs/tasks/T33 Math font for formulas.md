---
id: T33
title: Math font for formulas
status: done
wave: 18
branch: task/T33-math-font
depends_on: [T30]
adrs: ["[[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]"]
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

[[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]
records the font, the faces and character mapping, the generated metrics
table, the atlas key and the retuned constants (amending the metrics of
[[ADR-T30-1 TeX subset, box layout and metrics]]).

Acceptance criteria and the tests that prove them:

- **AC-1 — Embedded font.** `shell::app::MATH_FONT` is the Latin Modern
  Math subset (OpenType CFF, under 200 KB); every character the layout
  can put in the math face — every row of the metrics table, the italic
  and script letters, every symbol of the table — has a glyph in it.
  Tests: `math_font_is_embedded_and_small` (`src/shell/app.rs`);
  `tex_symbols_all_have_glyphs_in_the_math_font` (`src/shell/render.rs`).
- **AC-2 — Metrics table.** `core::math_metrics` has one row per
  character, sorted and finite; `get` finds each row and nothing else;
  the rows match the embedded font within 0.005 em.
  Tests: `math_metrics_table_is_sorted_and_finite`,
  `math_metrics_get_finds_every_row` (`src/core/math_metrics.rs`);
  `math_metrics_match_the_embedded_font` (`src/shell/glyphs.rs`).
- **AC-3 — Faces and mapping.** Formula letters are math italic,
  lowercase Greek italic, capital Greek, digits, operators and function
  names upright in the math face, `\mathcal{…}` script letters, literals
  and plain text in the text face, a character missing from the math font
  falls back to a text-face cell.
  Tests (`src/core/tex.rs`): `tex_parse_mathcal_takes_one_argument`,
  `tex_math_letters_are_italic`,
  `tex_lowercase_greek_is_italic_and_capitals_upright`,
  `tex_digits_operators_and_functions_are_upright_math`,
  `tex_mathcal_maps_to_script_letters`,
  `tex_literals_and_plain_text_use_the_text_face`,
  `tex_char_without_math_glyph_falls_back_to_a_text_cell`.
- **AC-4 — Proportional layout.** Glyph boxes come from the table; the
  T30 layout tests are restated with table metrics and the T33
  constants (`tex_layout_char_uses_its_glyph_metrics` replaces
  `tex_layout_char_is_one_cell`; scripts, fractions, roots, binomials,
  limits, spacing, line growth and mixed lines keep their names);
  a superscript clears the base's italic correction; a line's box holds
  italic overhangs at both ends.
  New tests (`src/core/tex.rs`):
  `tex_layout_superscript_clears_the_italic_correction`,
  `tex_layout_subscript_top_stays_low`,
  `tex_layout_line_holds_italic_overhangs`; proptests
  `tex_layout_bounds_contain_every_ink` and `tex_layout_is_finite`
  unchanged.
- **AC-5 — Rendering.** The atlas keys glyphs by face; formula glyphs
  are drawn from the math font, plain text and literals from the text
  font.
  Tests: `atlas_keeps_faces_apart`,
  `atlas_without_math_font_draws_math_with_the_text_font`
  (`src/shell/glyphs.rs`); `push_text_block_formula_uses_the_math_face`
  (`src/shell/render.rs`), and the T30 render tests.
- **AC-6 — Budgets and fuzz.** The T30 perf tests keep their thresholds
  with both fonts loaded (`perf_tex_layout_200_blocks_is_under_2ms`,
  `perf_tessellate_200_formula_blocks_is_under_1ms` in `tests/perf.rs`);
  the fuzz text sessions also type `\mathcal` and italic-overhang
  letters, and `check_invariants` / `fuzz_tex_parse_and_layout_are_total`
  hold (`tests/fuzz.rs`).
- **AC-7 — Docs.** README *Formulas in text blocks* mentions the font
  and `\mathcal`; `assets/fonts/README.md` records the subset command,
  the generator and the license.

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
- `assets/fonts/*`, `scripts/math_metrics.py` (new, metrics generator)
- `tests/fuzz.rs`, `tests/perf.rs`, `README.md`
- new ADRs `ADR-T33-*`, feature note `docs/features/TeX formulas.md`
  (update), tutorial
  `docs/tutorials/33 Typesetting with a proportional math font.md`

## Subtasks (one commit each)

- [x] spec — `docs(T33): specify math font acceptance criteria`
- [x] tests — `test(T33): add failing tests for math faces, metrics and rendering`
- [x] models — `feat(T33): add glyph faces and the math metrics table`
- [x] behaviour — `feat(T33): typeset formulas in Latin Modern Math`
- [x] quality — `chore(T33): pass clippy, rustfmt and rustdoc`
- [x] docs — `docs(T33): update feature note and add tutorial`

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
- 2026-10-15 — Spec step: the subset has no script lowercase (neither
  has TeX's `\mathcal`), so `\mathcal` maps capitals only; ADR-T33-1
  was corrected with the test commit, before integration.
- 2026-10-15 — Behaviour step: three spec tests were fixed with the
  code — `tex_literals_and_plain_text_use_the_text_face` forgot that the
  plain-text space is a glyph too; `math_metrics_match_the_embedded_font`
  allows 0.005 em instead of 0.002 em, since fontdue measures flattened
  curves (worst row 0.0032 em off the exact ink box; a control-point box
  matched worse); the spacing test now also asserts TeX's thin space
  before an operator after an atom (`n \log n`) and after punctuation,
  both missing until a rendered preview showed them.
- 2026-10-15 — Checked visually by rasterizing sample formulas
  (scripts, limits, fractions, roots, binomials, `\mathcal`, Greek,
  literals) with both fonts outside the app: italic variables, upright
  operators and functions, LaTeX-like spacing. The in-app look on the
  target is **pending the owner**.
- 2026-10-15 — Budgets (AC-6), release build on the development
  machine, single-threaded: `perf_tex_layout_200_blocks_is_under_2ms`
  median 1.02 ms (0.89 ms before: glyph boxes and ink spans now look up
  the metrics table); `perf_tessellate_200_formula_blocks_is_under_1ms`
  0.21 ms; `perf_tessellate_10k_glyphs_is_under_1ms` 0.48 ms;
  `perf_text_layout_and_cull_500_blocks_is_under_1ms` 0.10 ms. Run in
  parallel, unrelated perf tests miss their budgets from contention.
- 2026-10-15 — `docs/architecture/Architecture.md` updated: new module
  `core::math_metrics`, atlas keyed by face. Generator kept outside
  `scripts/check.sh` (needs fontTools); the table check runs in the
  tests. `scripts/check.sh` and `scripts/check.sh --fuzz` green. Status
  → review.
- 2026-10-15 — Integrated into `main`. Status → done.
