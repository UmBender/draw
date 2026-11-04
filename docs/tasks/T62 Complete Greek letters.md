---
id: T62
title: Complete Greek letters
status: todo
wave: 41
branch: task/T62-complete-greek
depends_on: [T54]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T62 Complete Greek letters

## Goal

Every Greek letter LaTeX (with amsmath/amssymb) can write is in draw:
the missing variants `\varkappa` and `\digamma`, the italic capitals
`\varGamma` … `\varOmega`, and the capitals that look like Latin letters
(`\Alpha`, `\Beta`, … as unicode-math defines them).

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
- `SYMBOLS` has the 24 lowercase letters (with `\omicron`) and the
  variants `\varepsilon \vartheta \varpi \varrho \varsigma \varphi`, and
  the 11 capitals with their own shape.
- `italic` already maps `ϰ` (`\varkappa`), but no command reaches it.
- `ϝ U+03DD` (`\digamma`) is in the Greek range of the subset.
- Italic capitals are `U+1D6E2–1D6FA` (in the subset). `\mathnormal`
  reaches them since T54.

## Proposal (settle in the spec step)

1. Add `\varkappa`, `\digamma`, `\Digamma`, `\varGamma` … `\varOmega`
   (amsmath names, italic capitals), `\Alpha \Beta \Epsilon \Zeta \Eta
   \Iota \Kappa \Mu \Nu \Omicron \Rho \Tau \Chi` (upright Greek
   capitals, unicode-math names).
2. Test: every Greek letter of `U+0391–03C9` plus the variants has a
   command, and every command draws a glyph from the math face.

## Spec

To be written in the spec step.

## Out of scope

- Upright lowercase Greek (`\upalpha`, `\mathrm{\alpha}`): the spec says
  whether `\mathrm` maps Greek. Default: no, as in LaTeX.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T62-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T62): …`
- [ ] tests — `test(T62): …`
- [ ] models — `feat(T62): …`
- [ ] behaviour — `feat(T62): …`
- [ ] quality — `chore(T62): …`
- [ ] docs — `docs(T62): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
