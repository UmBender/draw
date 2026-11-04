---
id: T58
title: Fraktur letters
status: todo
wave: 37
branch: task/T58-fraktur
depends_on: [T54, T55]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T58 Fraktur letters

## Goal

`\mathfrak{g}`, `\mathfrak{p}`, `\mathfrak{S}_n` (Lie algebras, prime
ideals, symmetric groups) drawn in the font's fraktur letters.

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
- Fraktur `U+1D504–1D537` with holes `ℭ U+212D, ℌ U+210C, ℑ U+2111,
  ℜ U+211C, ℨ U+2128`. `\Re`/`\Im` already draw `ℜ`/`ℑ`.

## Proposal (settle in the spec step)

1. `\mathfrak` alphabet row (Latin capitals and lowercase).
2. Test that every hole maps to its Letterlike character.

## Spec

To be written in the spec step.

## Out of scope

- Bold fraktur (`\mathbf{\mathfrak{…}}` keeps the innermost rule of T54).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T58-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T58): …`
- [ ] tests — `test(T58): …`
- [ ] models — `feat(T58): …`
- [ ] behaviour — `feat(T58): …`
- [ ] quality — `chore(T58): …`
- [ ] docs — `docs(T58): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
