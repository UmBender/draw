---
id: T56
title: Bold math
status: todo
wave: 35
branch: task/T56-bold-math
depends_on: [T54, T55]
adrs: []
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

To be written in the spec step.

## Out of scope

- Bold operators, bold fraction bars or roots.
- `\pmb` (poor man's bold).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T56-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T56): …`
- [ ] tests — `test(T56): …`
- [ ] models — `feat(T56): …`
- [ ] behaviour — `feat(T56): …`
- [ ] quality — `chore(T56): …`
- [ ] docs — `docs(T56): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
