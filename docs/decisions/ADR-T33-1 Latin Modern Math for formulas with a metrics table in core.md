---
id: ADR-T33-1
title: Latin Modern Math for formulas with a metrics table in core
status: accepted
kind: decision
date: 2026-10-15
task: T33
builds_on: [ADR-0003, ADR-0014, ADR-T29-1, ADR-T32-1, ADR-T32-2]
amends: [ADR-T30-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T33-1 Latin Modern Math for formulas with a metrics table in core

## Context

[[ADR-T30-1 TeX subset, box layout and metrics]] typesets formulas in the
monospaced text font: every glyph a 0.6 em cell, letters upright, `x`
next to `×` and `l` next to `1`. The owner asked for formulas that look
like LaTeX — italic serif variables, calligraphic capitals — and chose
**Latin Modern Math** (the OpenType companion of Computer Modern) over
STIX Two Math and New Computer Modern Math.

A proportional font breaks the one assumption of the T30 layout: uniform
cell metrics. `core` must stay font-free and pure
([[ADR-0003 Headless core and thin shell]]), add no dependency
([[ADR-0014 Minimal dependencies and no unsafe]]), and keep `bounds`,
hit-testing and culling a pure function of the text.

## Decision

### Font

- Embed `assets/fonts/LatinModernMath-draw-subset.otf`, a subset of Latin
  Modern Math 1.959 (GUST Font License, `GUST-FONT-LICENSE.txt`): ASCII,
  Latin-1 and Latin Extended-A (for `\text{…}`), Greek, general
  punctuation, letterlike symbols, arrows, mathematical operators,
  floor/ceiling, and from the Mathematical Alphanumeric Symbols block the
  italic Latin (`U+1D434–1D467`), script Latin (`U+1D49C–1D4CF`) and
  italic Greek (`U+1D6E2–1D71B`) letters. The GFL asks derived files to
  be renamed, hence the `-draw-subset` name; the original name stays in
  the font's name table. The `MATH` table is dropped: its constants are
  copied into `core` once (below). Under 200 KB.
- The shell loads it next to the text font into the one glyph atlas
  ([[ADR-T32-1 Own glyph atlas rasterized with fontdue]]): atlas keys
  become `(face, char, raster px)`, both faces share the pages and the
  one textured batch ([[ADR-T32-2 Shapes and glyphs share one textured batch]]).
  An atlas without a math font draws the math face with the text font.

### Faces and character mapping

`core::tex::Face` is `Text` (the monospaced font) or `Math`, and every
`Ink::Glyph` carries one. Inside a formula:

| Source | Drawn as | Face |
|--------|----------|------|
| `a–z`, `A–Z` | math italic `𝑎–𝑧`, `𝐴–𝑍` (`h` → `ℎ` U+210E) | Math |
| lowercase Greek symbols (`\alpha`, `\varphi`, …) | math italic Greek `𝛼 …` | Math |
| capital Greek, digits, operators, brackets, relations, arrows | upright, as before | Math |
| `\log \min …`, `\bmod`, `\pmod`, `\text{…}` | upright roman | Math |
| `\mathcal{…}` letters | script `𝒜 ℬ 𝒞 …` (holes from Letterlike Symbols); other characters as outside it | Math |
| literal source (warning colour) | as typed | Text |
| a character with no glyph in the math font | as typed, one 0.6 em cell | Text |

Plain text outside `$…$` and the open block's source stay in the text
face and its cells ([[ADR-T29-1 Text shape with monospaced layout and an embedded font]]).

`\mathcal` joins the parse as a command with one argument (braced or
single token), like `\sqrt`; it is upright for anything that is not a
Latin letter.

### Metrics table in core

`src/core/math_metrics.rs` is **generated** by `scripts/math_metrics.py`
(fontTools) from the original font and the subset's character map: one
row per character of the subset, sorted by character, with advance,
ink box (`x_min x_max y_min y_max`) and the `MATH` table's italic
correction, in em, y up. `math_metrics::get(ch)` is a binary search.
A shell test checks the table against the embedded font with fontdue
(advance and ink box within 0.002 em), so the table cannot drift from
the font.

A math-face glyph box is its advance by its ink height (`y_max` up,
`−y_min` down) — real heights as in TeX, so `a` is shorter than `b`.
`Ink::bounds` of a glyph is the union of that box and its ink box, so
italic overhangs (`𝑓`, `𝑗`) are covered. A formula at the start of a
line moves right by its left overhang, and a line is as wide as its
rightmost ink, so a block's bounds contain all of its ink.

### Layout constants

Taken from the font's `MATH` table (em) and TeX's spacing:

| Constant | T30 | T33 | Source |
|----------|-----|-----|--------|
| `AXIS` | 0.34 | 0.25 | `AxisHeight` |
| `RULE` | 0.07 | 0.04 | `FractionRuleThickness` |
| `SUP_SHIFT` | 0.4 | 0.363 | `SuperscriptShiftUp` |
| `SUP_DROP` | 0.35 | 0.25 | `SuperscriptBaselineDropMax` |
| `SUP_BOTTOM_MIN` | — | 0.108 | `SuperscriptBottomMin` |
| `SUB_SHIFT` | 0.2 | 0.247 | `SubscriptShiftDown` |
| `SUB_DROP` | 0.05 | 0.05 | kept |
| `SUB_TOP_MAX` | — | 0.344 | `SubscriptTopMax` |
| `SCRIPT_GAP` | 0.1 | 0.16 | `SubSuperscriptGapMin` |
| `SCRIPT_SPACE` | — | 0.056 | `SpaceAfterScript` |
| `LIMIT_GAP` | 0.1 | 0.167 | `LowerLimitGapMin` |
| `FRAC_GAP` | 0.08 | 0.04 | `FractionNumeratorGapMin` |
| `FRAC_NUM_SHIFT` | — | 0.394 | `FractionNumeratorShiftUp` |
| `FRAC_DEN_SHIFT` | — | 0.345 | `FractionDenominatorShiftDown` |
| `SQRT_GAP` | 0.1 | 0.1 | kept (between text and display style) |
| `REL_SPACE` | 0.15 | 5/18 | TeX thick space |
| `BIN_SPACE` | 0.1 | 2/9 | TeX medium space |
| `OP_SPACE` | 0.1 | 1/6 | TeX thin space |

Rules per constant are those of ADR-T30-1, plus: a superscript starts
after the base's **italic correction** (the italic correction of the
base's last glyph); its baseline is at least `SUP_BOTTOM_MIN` above its
bottom; a subscript's top is at most `SUB_TOP_MAX` above the baseline
(it moves down otherwise); a fraction's numerator baseline is at least
`FRAC_NUM_SHIFT` up and its denominator's at least `FRAC_DEN_SHIFT`
down; scripts add `SCRIPT_SPACE` after themselves. Scales
(`SCRIPT_SCALE` 0.7, `MIN_SCALE` 0.5) match the font's
`ScriptPercentScaleDown` and are kept. `∑ ∏` keep `BIG_OP_SCALE` 1.3,
centred on the axis; binomial parentheses keep their scaling; the root
sign stays four rules.

Text-face glyphs inside formulas keep the T30 cell metrics
(`ADVANCE`, `GLYPH_ASCENT`, `GLYPH_DESCENT`, `TALL_*`).

## Alternatives considered

- **STIX Two Math** — already installed, Times-like; the owner preferred
  the LaTeX look.
- **Reading metrics with fontdue at run time and passing them into
  `core`** — `bounds` would depend on shell state, breaking the pure
  layout and the per-text cache; tests and fuzzing of `core` would need
  the font.
- **Keeping `MATH` in the subset and reading it at run time** — needs an
  OpenType parser in `core` or a new dependency; the constants never
  change for one embedded font.
- **A separate atlas per font** — two page sets and texture binds per
  frame; one atlas with a face in the key keeps the T32 single batch.
- **Italic text-font letters** — the text font has no italic.

## Consequences

- Formulas look like LaTeX; plain text still lines up in cells.
- The binary grows by the subset (about 120 KB).
- Every formula's width changes: blocks with formulas get a new
  (usually narrower) box. Saved documents keep their text, so they only
  look different.
- Updating the font means regenerating the table with the script; the
  check test fails until then.
- `core` holds a few hundred rows of font data — accepted for purity.

## Rollback plan

A rollback ADR plus `revert(T33)` commits restore the T30 cell metrics,
the single-face atlas and remove the font and the table.
