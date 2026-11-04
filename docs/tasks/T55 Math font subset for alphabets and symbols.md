---
id: T55
title: Math font subset for alphabets and symbols
status: todo
wave: 34
branch: task/T55-math-font-subset
depends_on: [T54]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T55 Math font subset for alphabets and symbols

## Goal

The embedded Latin Modern Math subset grows once to hold every glyph
the TeX series T56–T71 needs: the Mathematical Alphanumeric Symbols
(bold, bold italic, script lowercase, fraktur, double-struck, sans-serif,
monospace, bold Greek, the digit styles), long and extra arrows,
supplemental operators (`⨁ ⨂ ⨀ ⨄ ⨆`, `⨿`), geometric shapes (`□ △ ◁ ▷`)
and the missing operators (`∖ ∁ ⊻ ⊲ ⊴ ⋊ ⋉ ≀ ⊣ ⊢ ⊨ …`). The metrics table
is regenerated. No formula behaviour changes in this task: later tasks
only add commands.

## Known facts (2026-11-03)

- The subset command and ranges are in `assets/fonts/README.md`
  (today: ASCII, Latin-1, Latin Extended-A, Greek, punctuation,
  letterlike, arrows, math operators, `U+2308–230B`, `U+27E8–27E9`,
  italic Latin `U+1D434–1D467`, script capitals `U+1D49C–1D4B5`, italic
  Greek `U+1D6E2–1D71B`). The subset is about 139 KB; ADR-T33-1 said
  "under 200 KB".
- `scripts/math_metrics.py ORIGINAL SUBSET src/core/math_metrics.rs`
  regenerates the table (888 rows today). It needs fontTools and the
  original `latinmodern-math.otf` 1.959 from CTAN `lm-math`. Neither is
  installed on the dev machine yet.
- `math_metrics_match_the_embedded_font` (shell) checks the table against
  the font.
- Latin Modern Math is expected to cover the whole `U+1D400–1D7FF`
  block. Holes that live in Letterlike Symbols (`ℂ ℍ ℕ ℙ ℚ ℝ ℤ`, `ℭ ℌ ℑ ℜ
  ℨ`, `ℯ ℊ ℴ`, …) are already in the subset. The spec step lists what the
  font really has.
- Owner (2026-11-03): ships in 1.5.0 ([[T53 Release 1.5.0]]).

## Proposal (settle in the spec step)

1. Spec: list the exact ranges T56–T71 need (go through each task's
   symbol list) and check them against the font's cmap. Add `U+1D400–1D7FF`,
   `U+27F0–27FF` (long arrows), `U+2900–297F` (supplemental arrows used by
   T68), `U+2A00–2A0C`, `U+2A3F`, `U+25A0–25FF` (geometric shapes).
2. ADR `ADR-T55-1` (amends ADR-T33-1): new ranges, new size budget
   (measure first; expected 250–350 KB), metrics table row count. Check
   that the atlas and start-up time stay inside
   [[ADR-T14-1 Performance budgets]].
3. Tests: one table row per new range spot-checked (`get('𝐀')`,
   `get('𝔄')`, `get('𝟘')`, `get('⟹')`, `get('⨁')`), and the font check
   test green.

## Spec

To be written in the spec step.

## Out of scope

- New commands (T56–T71).
- Glyphs Latin Modern Math does not have (e.g. rsfs-style `\mathscr`, a
  second script style).

## Files owned

- `assets/fonts/LatinModernMath-draw-subset.otf`, `assets/fonts/README.md`
- `src/core/math_metrics.rs` (regenerated)
- `scripts/math_metrics.py` (only if the generator needs a fix)
- `src/shell/glyphs.rs` (font check test only)
- new ADR `ADR-T55-1`

## Subtasks (one commit each)

- [ ] spec — `docs(T55): …`
- [ ] tests — `test(T55): …`
- [ ] models — `feat(T55): …`
- [ ] behaviour — `feat(T55): …`
- [ ] quality — `chore(T55): …`
- [ ] docs — `docs(T55): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
