---
id: T61
title: Hebrew letters
status: todo
wave: 40
branch: task/T61-hebrew-letters
depends_on: []
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T61 Hebrew letters

## Goal

The Hebrew letters of set theory, as in LaTeX: `\aleph_0`,
`2^{\aleph_0}`, `\beth_1`, `\gimel`, `\daleth`.

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
- `ℵ U+2135, ℶ U+2136, ℷ U+2137, ℸ U+2138` are in the Letterlike
  Symbols range, which is already in both subsets: no font change.
- Owner (2026-11-03): math letters only. Hebrew text (`א–ת` in
  `\text{…}` or plain text) is not wanted. It would need another font
  and right-to-left layout.

## Proposal (settle in the spec step)

1. Four `SYMBOLS` rows, `Class::Ord`, upright (not mapped by alphabets).
2. Scripts attach normally (`\aleph_0`).

## Spec

To be written in the spec step.

## Out of scope

- The Hebrew block `U+0590–05FF`, right-to-left text.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T61-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T61): …`
- [ ] tests — `test(T61): …`
- [ ] models — `feat(T61): …`
- [ ] behaviour — `feat(T61): …`
- [ ] quality — `chore(T61): …`
- [ ] docs — `docs(T61): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
