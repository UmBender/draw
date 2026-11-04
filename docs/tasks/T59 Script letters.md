---
id: T59
title: Script letters
status: todo
wave: 38
branch: task/T59-script-letters
depends_on: [T54, T55]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T59 Script letters

## Goal

`\mathscr{L}`, `\mathscr{F}` and lowercase script (`\mathscr{l}`) in
formulas. `\mathcal` stays capitals only, as in TeX.

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
- Script capitals `U+1D49C–1D4B5` (used by `\mathcal` since T33),
  script lowercase `U+1D4B6–1D4CF` with holes `ℯ U+212F, ℊ U+210A,
  ℴ U+2134`.
- In LaTeX `\mathscr` (rsfs) is a rounder, more formal script than
  `\mathcal`. Latin Modern Math has one script style. Unicode tells the
  two apart only with variation selectors (`U+FE00` chancery,
  `U+FE01` roundhand), which this font may not support. The spec checks.

## Proposal (settle in the spec step)

1. `\mathscr` alphabet row: capitals as `\mathcal`, plus lowercase.
2. If the font has a second script style (variation sequences or a
   stylistic set), the ADR says whether `\mathscr` uses it. Otherwise
   it says that `\mathscr` and `\mathcal` capitals look the same.

## Spec

To be written in the spec step.

## Out of scope

- Adding a second font for rsfs-style script.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T59-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T59): …`
- [ ] tests — `test(T59): …`
- [ ] models — `feat(T59): …`
- [ ] behaviour — `feat(T59): …`
- [ ] quality — `chore(T59): …`
- [ ] docs — `docs(T59): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
