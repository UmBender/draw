---
id: T71
title: Combinatorics notation
status: todo
wave: 50
branch: task/T71-combinatorics
depends_on: [T55]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T71 Combinatorics notation

## Goal

Combinatorics notation beyond `\binom`: `\tbinom`/`\dbinom`, the infix
`n \choose k`, Stirling numbers `n \brack k` and `n \brace k`,
multisets `\multiset{n}{k}` (double parentheses), falling and rising
factorials `x^{\underline{k}}` and `x^{\overline{k}}`, `\nmid`,
`\#` and `\lvert A \rvert` for cardinalities.

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
- `\binom` exists with scaled parentheses (`binomial`). `\overline`
  exists (ADR-T39-1). `\underline` does not: the feature note lists it
  under *Limits*.
- No display mode in draw, so `\dbinom` and `\tbinom` would draw like
  `\binom`.
- `\choose`, `\brack`, `\brace` are infix (they split their group in
  two), which the parser does not do yet.

## Proposal (settle in the spec step)

1. `\tbinom`, `\dbinom` as `\binom`. `\multiset` is not standard LaTeX
   (it comes from a common macro): the spec decides whether to add it.
2. Infix `\choose`, `\brack`, `\brace`: the parser splits the enclosing
   group (or the formula) at the command, the way `\over` works in TeX.
   ADR `ADR-T71-1`.
3. `\underline{…}`: a rule under the argument (the mirror of
   `\overline`).
4. `\nmid` (`∤`, `Rel`), `\#` (`#`, `Ord`).

## Spec

To be written in the spec step.

## Out of scope

- `\over`, `\atop` (the spec may add them, since the infix machinery
  is the same).
- Generating-function notation beyond what is already there
  (`[x^n] F(x)` needs nothing new).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T71-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T71): …`
- [ ] tests — `test(T71): …`
- [ ] models — `feat(T71): …`
- [ ] behaviour — `feat(T71): …`
- [ ] quality — `chore(T71): …`
- [ ] docs — `docs(T71): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
