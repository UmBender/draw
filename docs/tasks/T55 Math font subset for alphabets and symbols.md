---
id: T55
title: Math font subset for alphabets and symbols
status: in-progress
wave: 34
branch: task/T55-math-font-subset
depends_on: [T54]
adrs: ["[[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]"]
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

Decision: [[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]
(amends [[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]).

### Ranges

The subset keeps every range of ADR-T33-1 and adds these (the three
alphanumeric ranges of ADR-T33-1 now fall inside `U+1D400–1D7FF`):

| Range | What | In the font (cmap of Latin Modern Math 1.959) |
|-------|------|-----------------------------------------------|
| `U+1D400–1D7FF` | Mathematical Alphanumeric Symbols: bold, italic, bold italic, script, fraktur, double-struck, sans-serif (all weights), monospace, Greek styles, digit styles | 945 of 1024. The 79 missing: the 24 reserved holes encoded in Letterlike Symbols (`ℎ ℬ ℰ ℱ ℋ ℐ ℒ ℳ ℛ ℯ ℊ ℴ ℭ ℌ ℑ ℜ ℨ ℂ ℍ ℕ ℙ ℚ ℝ ℤ`), 4 unassigned (`U+1D6A6–1D6A7`, `U+1D7CC–1D7CD`), and 51 the font lacks: **script lowercase** `U+1D4B6–1D4CF` (23), **bold script lowercase** `U+1D4EA–1D503` (26), bold digamma `U+1D7CA–1D7CB` (2). Bold script capitals `U+1D4D0–1D4E9` are there. |
| `U+27F0–27FF` | long arrows (`⟵ ⟶ ⟷ ⟸ ⟹ ⟺ ⟻ ⟼ ⟽ ⟾ ⟿`, `⟴`) | 12 of 16 (`U+27F4–27FF`) |
| `U+2900–297F` | supplemental arrows | 2 of 128 (`⤆ ⤇`, `U+2906–2907`) |
| `U+2A00–2A0C` | n-ary operators (`⨀ ⨁ ⨂ ⨃ ⨄ ⨅ ⨆ ⨉ ⨌`) | 9 of 13 (`U+2A00–2A06`, `U+2A09`, `U+2A0C`) |
| `U+2A3F` | amalgamation `⨿` | 1 of 1 |
| `U+25A0–25FF` | geometric shapes (`■ □ ▪ ▫ ▬ ▭ ▲ △ ▶ ▷ ▼ ▽ ◀ ◁ ◊ ○ ● ◦ ◯`) | 19 of 96 |
| `U+20D0–20DF` | combining marks for symbols (`U+20DB` for T69's `\dddot`) | 14 of 16 (no `U+20D9`, `U+20DA`) |

Facts for later tasks: the font has **no script lowercase** (plain or
bold), so T59 can only map capitals; `U+2AEB` (`⫫`) and `U+2AA6` are **not** in the
font; the font has no variation sequences (no cmap format 14) and its
GSUB has only the features `aalt dtls ssty` (so no `ss01`-style second
script for T59). The missing operators of the goal (`∖ ∁ ⊻ ⊲ ⊴ ⋊ ⋉ ≀ ⊣
⊢ ⊨`) were already in the subset (`U+2200–22FF`).

Measured: the subset is **270 232 bytes** (was 139 336), the metrics
table has **1762 rows** (was 888). The rows that were there keep their
values; only new rows appear. The rebuild command with the old ranges
reproduces the committed subset byte for byte, so the new subset is
reproducible with the command in `assets/fonts/README.md`.

### Acceptance criteria

- **AC-1** — one new row per added range is in the table and the font:
  `math_metrics_new_ranges_have_spot_check_rows` (`𝐀 U+1D400`,
  `𝔄 U+1D504`, `𝟘 U+1D7D8`, `⟹ U+27F9`, `⨁ U+2A01`, `⤇ U+2907`, `⨿ U+2A3F`,
  `□ U+25A1`, `U+20DB`).
- **AC-2** — the table holds exactly what the font has in each new range:
  `math_metrics_new_ranges_match_the_font_cmap` (counts 945, 12, 2, 9,
  1, 19, 14 from the table above).
- **AC-3** — characters the font lacks stay out (the formula code draws
  them with the text face): `math_metrics_glyphs_missing_from_the_font_have_no_row`
  (`U+2AEB`, `U+2AA6`, the hole `U+1D455`, script `𝒶 U+1D4B6`,
  `U+1D7CA`, `U+20D9`).
- **AC-4** — the table matches the embedded font:
  `math_metrics_match_the_embedded_font` (shell, existing) stays green
  with 1762 rows.
- **AC-5** — the font stays inside the new budget:
  `math_font_is_embedded_and_small` (shell, budget raised from 200 000 to
  300 000 bytes by ADR-T55-1).
- **AC-6** — no formula behaviour changes: every existing test (TeX
  layout, `tex_symbols_all_have_glyphs_in_the_math_font`, fuzz) stays
  green unchanged; `tests/perf.rs` budgets ([[ADR-T14-1 Performance budgets]])
  stay green in release.

## Out of scope

- New commands (T56–T71).
- Glyphs Latin Modern Math does not have (e.g. rsfs-style `\mathscr`, a
  second script style, script lowercase, `⫫ U+2AEB`, `U+2AA6`).

## Files owned

- `assets/fonts/LatinModernMath-draw-subset.otf`, `assets/fonts/README.md`
- `src/core/math_metrics.rs` (regenerated)
- `scripts/math_metrics.py` (only if the generator needs a fix)
- `src/shell/glyphs.rs` (font check test only)
- `src/shell/app.rs` (the size budget in `math_font_is_embedded_and_small`
  only; added in the spec step, see *Log*)
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
- 2026-11-04 — spec written (batch run). UNATTENDED CHOICE: added
  `src/shell/app.rs` to *Files owned*, limited to the `200_000` →
  `300_000` byte budget in `math_font_is_embedded_and_small`. That test
  encodes ADR-T33-1's "under 200 KB", which the 270 232-byte subset
  breaks; the owner's 300 KB budget (ADR-T55-1) needs it. No other line
  of `app.rs` changes. Owner: confirm or move the guard.
