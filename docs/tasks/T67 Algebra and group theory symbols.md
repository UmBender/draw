---
id: T67
title: Algebra and group theory symbols
status: todo
wave: 46
branch: task/T67-algebra-symbols
depends_on: [T55, T64]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T67 Algebra and group theory symbols

## Goal

The most used notation of abstract algebra and group theory:
normal subgroups `N \trianglelefteq G` (`\lhd \unlhd \rhd \unrhd
\triangleleft \triangleright \ntriangleleft`), semidirect products
`N \rtimes H` (`\ltimes`), wreath `\wr`, `\odot \ominus \oslash \boxplus
\boxtimes \amalg \diamond`, `G/N`, `\langle g \rangle`, `[G:H]`, and the
upright names `\Hom \End \Aut \Inn \Gal \Ker \im \coker \id \sgn \tr
\rank \char`.

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
- `⊲ ⊴ ⊳ ⊵ ⋊ ⋉ ≀ ⊙ ⊖ ⊘ ⊞ ⊠ ⋄` are in `U+2200–22FF`; `⨿ U+2A3F` comes
  from T55.
- `\Hom`, `\End`, … are not LaTeX built-ins: they are usual
  `\DeclareMathOperator`s. draw defines them, built on `\operatorname`
  (T64). `\char` clashes with a TeX primitive; the spec picks a name.
- Quotients, cosets, generated subgroups and indices need nothing
  new; the feature note shows them.

## Proposal (settle in the spec step)

1. `SYMBOLS` rows (`Rel` for the triangle relations, `Bin` for the
   products) and `Op` rows for the names.
2. The spec makes the final list. Rule: a symbol goes in if it shows up
   in a standard first-year algebra text (Dummit–Foote, Lang).

## Spec

To be written in the spec step.

## Out of scope

- Commutative diagrams, extensible arrows (T68 and [[T72 Feature proposal]]).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T67-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T67): …`
- [ ] tests — `test(T67): …`
- [ ] models — `feat(T67): …`
- [ ] behaviour — `feat(T67): …`
- [ ] quality — `chore(T67): …`
- [ ] docs — `docs(T67): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
