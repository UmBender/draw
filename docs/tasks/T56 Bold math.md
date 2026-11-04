---
id: T56
title: Bold math
status: done
wave: 35
branch: task/T56-bold-math
depends_on: [T54, T55]
adrs: ["[[ADR-T56-1 Accent skew follows a slant flag per math alphabet]]"]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T56 Bold math

## Goal

Bold letters in formulas, as in LaTeX: `\mathbf{v}` (upright bold Latin
and digits, upright bold Greek capitals), `\boldsymbol{\alpha}` and
`\bm{x}` (bold italic, including lowercase Greek), and `\boldmath`,
which makes the rest of its formula bold.

## Known facts (2026-11-03)

- Formulas are parsed and laid out in `core::tex` (`src/core/tex.rs`):
  symbol commands are rows of `SYMBOLS` (`name`, `text`, `Class`),
  one-argument commands go through `Parser::command`, and letters are
  mapped by `Style` (`Italic`, `Calligraphic`) through `italic` /
  `calligraphic` ([[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]).
- A character missing from the embedded math font subset falls back to
  a monospaced text-face cell, and an unknown command is drawn in the
  warning colour. `core::math_metrics` must hold a row for every
  character the math face draws (`math_metrics_match_the_embedded_font`).
- Owner (2026-11-03): ships in 1.5.0 ([[T53 Release 1.5.0]]), the last
  feature release.
- Bold `U+1D400–1D433`, bold italic `U+1D468–1D49B`, bold Greek
  `U+1D6A8–1D6E1`, bold italic Greek `U+1D71C–1D755`, bold digits
  `U+1D7CE–1D7D7` (from T55).
- In LaTeX `\boldmath` is a declaration used *outside* math mode; inside
  `$…$` it has no effect. Latin Modern Math has no bold operators, so
  only letters, digits and Greek can be bold.

## Proposal (settle in the spec step)

1. `\mathbf`: alphabet row (upright bold; Greek lowercase stays italic,
   as in LaTeX).
2. `\boldsymbol`/`\bm`: alphabet row (bold italic Latin and Greek).
3. `\boldmath`: the spec decides between (a) a declaration inside a
   formula that turns on `\boldsymbol` style until the end of the
   formula or the enclosing group (`{\boldmath x} y`), and (b) the
   LaTeX meaning, which would make it useless in draw. Proposed: (a),
   recorded in `ADR-T56-1` as a departure from LaTeX.

## Spec

Settled (owner, batch plan 2026-11-04; rows of
[[ADR-T54-1 Math alphabets as a table of per-kind mappings]], skew rule
in [[ADR-T56-1 Accent skew follows a slant flag per math alphabet]]).

| Command | Latin A–Z | Latin a–z | Digits | Greek capitals | Greek lowercase |
|---------|-----------|-----------|--------|----------------|-----------------|
| `\mathbf` | bold U+1D400… | bold U+1D41A… | bold U+1D7CE… | bold U+1D6A8… | inherit |
| `\boldsymbol`, `\bm` | bold italic U+1D468… | bold italic U+1D482… | bold U+1D7CE… | bold italic U+1D71C… | bold italic U+1D736… |

Bold italic has no hole (`h` → U+1D489, unlike italic `ℎ`). Both
commands take one argument (braced or one token) and nest like every
alphabet (innermost wins); operators, functions, `\text`, `\partial`
and Hebrew letters are never bold (the font has no bold operators).

`AlphabetDef` gains `slant: bool`: whether the glyphs the row maps lean
right. An accent over one letter is skewed only when the row that draws
it maps the letter to another character **and** is slanted, so
`\hat{\mathbf{v}}` is centred while `\hat{\boldsymbol{v}}` and
`\mathbf{\hat{\alpha}}` (italic α) are skewed. Existing rows keep their
behaviour: `MATH_ITALIC`, `\mathit`, `\mathnormal`, `\mathcal` slanted,
`\mathrm` upright.

- **AC-1** `\mathbf` maps Latin, digits and Greek capitals to upright
  bold; lowercase Greek stays math italic —
  `tex_mathbf_maps_to_upright_bold`.
- **AC-2** `\boldsymbol` and `\bm` are the same row and map Latin, digits
  and all Greek (variants included) to bold italic / bold —
  `tex_boldsymbol_and_bm_map_to_bold_italic`.
- **AC-3** Both commands parse with one argument, are literals (warning
  colour) without one, and print back unchanged —
  `tex_parse_bold_commands_take_one_argument`; every glyph they draw is
  in the math face and `math_metrics` —
  `tex_bold_rows_draw_glyphs_of_the_math_face` (and the existing
  `tex_alphabet_rows_draw_glyphs_of_the_math_face` over `ALPHABETS`).
- **AC-4** Accent skew: centred over upright bold, skewed over bold italic
  and over italic Greek inside `\mathbf`; existing skews unchanged —
  `tex_layout_accent_skew_over_bold_letters` (plus
  `tex_layout_accent_skew_follows_the_alphabet`).
- **AC-5** Nesting and reach: `\mathbf{\boldsymbol{x}}` is bold italic,
  `\boldsymbol{\mathrm{x}}` upright; operators, functions, `\text` and
  `\partial` unchanged — `tex_bold_alphabets_nest_and_leave_operators_alone`.
- **AC-6** `\boldmath` stays an unknown command drawn in the warning
  colour — `tex_boldmath_is_an_unknown_command`.
- **AC-7** The fuzzer types `\mathbf`, `\boldsymbol`, `\bm` and
  `\boldmath` (`tests/fuzz.rs`).

## Out of scope

- `\boldmath`: in LaTeX it is a declaration valid only *outside* math
  mode, so inside `$…$` it does nothing; draw leaves it an unknown
  command (owner, batch plan 2026-11-04).
- Bold operators, bold fraction bars or roots (Latin Modern Math has no
  bold operators).
- `\pmb` (poor man's bold).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T56-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T56): …`
- [x] tests — `test(T56): …`
- [x] models — `feat(T56): …`
- [x] behaviour — `feat(T56): …`
- [x] quality — `chore(T56): …` (nothing to change, no commit)
- [x] docs — `docs(T56): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — spec written (batch run); `\boldmath` left out as the
  owner decided. UNATTENDED CHOICE: new `AlphabetDef::slant` flag so an
  accent is centred over upright bold (as in LaTeX), recorded in
  ADR-T56-1.
- 2026-11-04 — tests, `slant` field, `\mathbf`/`\boldsymbol`/`\bm` rows.
  In the behaviour step two test assumptions turned out wrong and were
  settled: the font gives bold `f` an italic correction (the assertion
  that upright bold has none was dropped), and `\boldsymbol` digits are
  upright, so digits are never skewed (added to ADR-T56-1).
- 2026-11-04 — `scripts/check.sh` and `scripts/check.sh --fuzz` green
  with no changes, so no quality commit. README row and feature note
  section added; status `review`. Manual checks for the owner:
  `$\mathbf{v} \cdot \bm{\alpha}$`, `$\mathbf{\Gamma}$`,
  `$\hat{\mathbf{v}}$` (centred).
- 2026-11-04 — integrated on `main` (merge `e2d7e8c`), no conflicts; the owner's manual checks and review of the unattended choices are still open.
