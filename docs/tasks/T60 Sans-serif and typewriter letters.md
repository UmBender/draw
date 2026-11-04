---
id: T60
title: Sans-serif and typewriter letters
status: review
wave: 39
branch: task/T60-sans-and-mono
depends_on: [T54, T55]
adrs: ["[[ADR-T60-1 Bold sans-serif inside a bold alphabet]]"]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T60 Sans-serif and typewriter letters

## Goal

`\mathsf{G}` (sans-serif) and `\mathtt{x}` (typewriter) in formulas,
for categories, matrices or code names, with sans-serif and monospace
digits.

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
- Sans `U+1D5A0–1D5D3`, sans digits `U+1D7E2–1D7EB`; monospace
  `U+1D670–1D6A3`, monospace digits `U+1D7F6–1D7FF`. Bold and italic
  sans also exist (`U+1D5D4–1D66F`).
- The owner's list said `matht`. Read as `\mathtt`; the spec confirms
  this.

## Proposal (settle in the spec step)

1. `\mathsf` and `\mathtt` alphabet rows.
2. The spec decides whether `\mathsf` under `\boldmath`/`\bm` maps to
   bold sans (Unicode has it) or stays regular sans.

## Spec

Owner decisions (batch plan, 2026-11-04): `\matht` is `\mathtt`; bold
sans under a bold alphabet, recorded in
[[ADR-T60-1 Bold sans-serif inside a bold alphabet]]. Every glyph below
has a `core::math_metrics` row (T55 subset).

| Command | Latin A–Z | Latin a–z | Digits | Greek | Holes | Slant |
|---------|-----------|-----------|--------|-------|-------|-------|
| `\mathsf` | U+1D5A0… | U+1D5BA… | U+1D7E2… | inherit | none | no |
| `\mathtt` | U+1D670… | U+1D68A… | U+1D7F6… | inherit | none | no |
| bold sans (internal, not a command) | U+1D5D4… | U+1D5EE… | U+1D7EC… | inherit | none | no |

- **AC-1** `\mathsf` maps Latin and digits to sans-serif, Greek as
  outside (`\mathsf{G}` is `𝖦`, `\mathsf{x1}` is `𝗑 𝟣`) —
  `tex_mathsf_maps_to_sans_serif`.
- **AC-2** `\mathtt` maps Latin and digits to monospace, Greek as
  outside (`\mathtt{x1}` is `𝚡 𝟷`) — `tex_mathtt_maps_to_monospace`.
- **AC-3** Both parse with one argument (braced or one token), are a
  literal in the warning colour without one, print back unchanged, and
  draw only math-face glyphs with `math_metrics` rows —
  `tex_parse_sans_and_mono_take_one_argument`,
  `tex_sans_and_mono_draw_glyphs_of_the_math_face`.
- **AC-4** An accent over a sans, monospace or bold-sans letter or digit
  is centred (upright rows, ADR-T56-1) —
  `tex_layout_accent_centred_over_sans_and_mono_letters`.
- **AC-5** `\mathsf` in a bold alphabet is bold sans, upright:
  `\mathbf{\mathsf{x}}`, `\bm{\mathsf{x}}`, `\boldsymbol{\mathsf{x}}`,
  `\mathsf{\mathbf{x}}` and `\mathsf{\bm{x}}` are `𝘅`, digits bold sans
  (`𝟭`), Greek as the bold alphabet draws it; `\mathrm` in between
  breaks the chain (ADR-T60-1) —
  `tex_mathsf_in_a_bold_alphabet_is_bold_sans`.
- **AC-6** `\mathtt` has no bold: `\mathbf{\mathtt{x}}` is `𝚡`,
  `\mathtt{\mathbf{x}}` is `𝐱`; other nestings innermost-wins; every
  T54/T56 bold nesting result unchanged; `\mathsf{A}_n` keeps an italic
  n; operators and `\text` unchanged —
  `tex_sans_and_mono_nest_and_leave_operators_alone`.
- **AC-7** The fuzzer types `\mathsf` and `\mathtt` (`tests/fuzz.rs`).

## Out of scope

- Sans-serif Greek.
- Bold italic sans (`U+1D63C…`) and italic sans: no command maps to
  them (ADR-T60-1).
- Bold monospace: Unicode has none.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T60-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T60): …`
- [x] tests — `test(T60): …`
- [x] models — `feat(T60): …`
- [x] behaviour — `feat(T60): …`
- [x] quality — `chore(T60): …`
- [x] docs — `docs(T60): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — tests, rows and docs done. `AlphabetDef` has a `bold`
  flag; `\mathsf`, `\mathtt` are rows in `ALPHABETS`, bold sans is the
  internal `MATHSF_BOLD` row picked in `Style::with` (ADR-T60-1). No
  deviation from the spec or *Files owned*. The quality step only ran
  rustfmt on the tests. `scripts/check.sh` and `--fuzz` green. No
  tutorial. Manual check for the owner:
  `$\mathsf{G}, \mathtt{x1}, \mathbf{\mathsf{x}}, \hat{\mathsf{A}}$`
  (sans G, monospace x1, bold sans x, centred hat).
