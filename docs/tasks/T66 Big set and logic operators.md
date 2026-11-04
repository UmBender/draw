---
id: T66
title: Big set and logic operators
status: todo
wave: 45
branch: task/T66-big-operators
depends_on: [T55, T65]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T66 Big set and logic operators

## Goal

Big operators beyond `\sum` and `\prod`, with limits like them:
`\bigcup_{i=1}^{n} A_i`, `\bigcap`, `\bigwedge`, `\bigvee`, `\bigoplus`,
`\bigotimes`, `\bigodot`, `\biguplus`, `\bigsqcup`, `\coprod`.

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
- `Class::BigOp` enlarges a glyph by `BIG_OP_SCALE` 1.3, centred on the
  axis, with limits above and below (`big_operator`, `limits`).
- `⋃ ⋂ ⋀ ⋁ ∐` are in `U+2200–22FF`. `⨀ ⨁ ⨂ ⨄ ⨆` (`U+2A00–2A06`) come from
  T55.

## Proposal (settle in the spec step)

1. Ten `Class::BigOp` rows.
2. Tests: limits sit centred above/below like `\sum`, and the box
   contains the enlarged glyph.

## Spec

To be written in the spec step.

## Out of scope

- Display-size operators, `\limits`/`\nolimits`.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T66-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T66): …`
- [ ] tests — `test(T66): …`
- [ ] models — `feat(T66): …`
- [ ] behaviour — `feat(T66): …`
- [ ] quality — `chore(T66): …`
- [ ] docs — `docs(T66): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
