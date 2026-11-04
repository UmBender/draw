---
id: T64
title: Named operators with limits
status: todo
wave: 43
branch: task/T64-named-operators
depends_on: [T63]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T64 Named operators with limits

## Goal

The remaining named operators of LaTeX are upright, and those that take
limits put them below: `\inf_{x\in S}`, `\sup`, `\liminf`, `\limsup`,
`\det`, `\Pr`, `\max`/`\min` (already there), plus `\exp \lg \deg \dim
\ker \hom \arg`. `\operatorname{rank}` and `\operatorname*{argmin}_x`
write any other operator name.

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
- `Class::Limits` (`\min \max \lim`) puts scripts below/above;
  `Class::Op` puts them beside.
- In LaTeX `\det \gcd \inf \sup \liminf \limsup \Pr \max \min \lim` take
  limits; `\gcd` is `Class::Op` in draw today (the spec decides whether
  to change it).
- `\text{…}` already parses a raw body, so `\operatorname` can reuse
  it.

## Proposal (settle in the spec step)

1. `Class::Limits` rows: `\inf \sup \liminf \limsup \det \Pr`
   (`\liminf`/`\limsup` drawn as "lim inf"/"lim sup" with a thin space).
2. `Class::Op` rows: `\exp \lg \deg \dim \ker \hom \arg`.
3. `\operatorname{…}` (`Op`) and `\operatorname*{…}` (`Limits`): one
   argument, upright word. The ADR covers the starred form in the
   parser.
4. Algebra names (`\Hom \End \Aut \im \coker \id`) are T66/T67, built
   on `\operatorname`.

## Spec

To be written in the spec step.

## Out of scope

- `\varliminf`, `\varlimsup`, `\injlim`, `\projlim`.
- `\DeclareMathOperator` or any user macros.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T64-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T64): …`
- [ ] tests — `test(T64): …`
- [ ] models — `feat(T64): …`
- [ ] behaviour — `feat(T64): …`
- [ ] quality — `chore(T64): …`
- [ ] docs — `docs(T64): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
