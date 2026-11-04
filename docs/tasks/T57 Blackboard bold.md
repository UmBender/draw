---
id: T57
title: Blackboard bold
status: todo
wave: 36
branch: task/T57-blackboard-bold
depends_on: [T54, T55]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T57 Blackboard bold

## Goal

Number sets as in LaTeX: `\mathbb{R}`, `\mathbb{N}`, `\mathbb{Z}_p`,
`\mathbb{F}_q`, and `\mathds{1}` (the dsfont indicator function) drawn
with the font's double-struck letters and digits.

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
- Double-struck `U+1D538–1D56B` with holes in Letterlike Symbols
  (`ℂ U+2102, ℍ U+210D, ℕ U+2115, ℙ U+2119, ℚ U+211A, ℝ U+211D, ℤ U+2124`),
  double-struck digits `U+1D7D8–1D7E1`.
- amssymb's `\mathbb` has capitals only; dsfont's `\mathds` has capitals
  plus `1`, `h` and `k`. Unicode (and the font) also has lowercase.

## Proposal (settle in the spec step)

1. `\mathbb` and `\mathds` are one alphabet (capitals, lowercase and
   digits double-struck); the ADR notes that draw accepts more than
   amssymb.
2. The tutorial idea from step 33 ("Try it" 2) becomes real: update that
   tutorial's exercise to point at another alphabet.

## Spec

To be written in the spec step.

## Out of scope

- Shortcuts like `\R`, `\N` (not standard LaTeX).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T57-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T57): …`
- [ ] tests — `test(T57): …`
- [ ] models — `feat(T57): …`
- [ ] behaviour — `feat(T57): …`
- [ ] quality — `chore(T57): …`
- [ ] docs — `docs(T57): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
