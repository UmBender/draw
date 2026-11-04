---
id: T63
title: Trigonometric and hyperbolic functions
status: todo
wave: 42
branch: task/T63-trig-functions
depends_on: []
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T63 Trigonometric and hyperbolic functions

## Goal

Trigonometric and hyperbolic functions are upright with operator
spacing, as in LaTeX: `\sin \cos \tan \cot \sec \csc`, `\arcsin \arccos
\arctan`, `\sinh \cosh \tanh \coth`, so `$\sin^2\theta + \cos^2\theta = 1$`
and `$\tan x$` typeset properly.

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
- `\log \ln \gcd \lcm` are `Class::Op` rows (upright word, thin space
  before them after an ordinary atom, ADR-T33-1 `OP_SPACE`).
- Scripts on an `Op` (`\sin^2`) sit beside it like on a letter.

## Proposal (settle in the spec step)

1. Thirteen `Class::Op` rows. Also `\arccot`, `\arcsec`, `\arccsc`,
   `\sech`, `\csch` (not in LaTeX's base set, but common): the spec
   decides. Proposed: yes, they are cheap.
2. Test `$\sin x$` spacing equals `$\log x$` spacing.

## Spec

To be written in the spec step.

## Out of scope

- Function application rules beyond TeX's (no automatic parentheses).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T63-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T63): …`
- [ ] tests — `test(T63): …`
- [ ] models — `feat(T63): …`
- [ ] behaviour — `feat(T63): …`
- [ ] quality — `chore(T63): …`
- [ ] docs — `docs(T63): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
