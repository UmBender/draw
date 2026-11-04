---
id: T54
title: Math alphabet styles
status: todo
wave: 33
branch: task/T54-math-alphabets
depends_on: []
adrs: []
feature: "[[TeX formulas]]"
tutorial: "[[54 Math alphabets as a mapping table]]"
tags: [task, tex]
---

# T54 Math alphabet styles

## Goal

Formulas know the idea of a **math alphabet**: a style that maps the
letters (and, where it applies, digits and Greek) of its argument to
other characters. `\mathcal` becomes one alphabet of a table instead of
a special case, and three alphabets that need no new glyphs arrive with
it: `\mathrm{d}` (upright roman), `\mathit{diff}` (italic, set as one
word) and `\mathnormal{\Gamma}` (the default style, with italic Greek
capitals as in LaTeX). Later tasks (T56–T60) then only add rows.

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
- Upright Latin, the italic Latin of `U+1D434–1D467` and the italic Greek
  capitals of `U+1D6E2–1D6FA` are already in the subset, so this task
  needs no font change.
- In LaTeX the innermost alphabet wins (`\mathrm{\mathcal{A}}` is a
  calligraphic A), and an alphabet does not change operators, digits in
  `\mathit`, or `\text{…}`.

## Proposal (settle in the spec step)

1. Replace `Style` with an alphabet table: one row per command name,
   with a mapping function per character kind (Latin capitals, Latin
   lowercase, digits, lowercase Greek, capital Greek). A missing mapping
   falls back to the enclosing style, then to italic.
2. `\mathrm`, `\mathit`, `\mathnormal` take one argument like
   `\mathcal`. Without an argument they are literals (warning colour).
3. `\mathrm` letters are upright `Ord` atoms: `\mathrm{d}x` gets no
   italic correction. `\mathit{diff}` spaces as one word (no gaps between
   letters, as in LaTeX).
4. ADR `ADR-T54-1` (amends ADR-T33-1): the alphabet table, nesting
   rule, fallback.

## Spec

To be written in the spec step.

## Out of scope

- Alphabets that need new glyphs (bold, blackboard, fraktur, script,
  sans, mono): T56–T60.
- `\operatorname` (T64).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T54-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T54): …`
- [ ] tests — `test(T54): …`
- [ ] models — `feat(T54): …`
- [ ] behaviour — `feat(T54): …`
- [ ] quality — `chore(T54): …`
- [ ] docs — `docs(T54): …`

## Learning path

Possibly a short tutorial on table-driven character mapping (Unicode math alphanumerics); decided in the spec step.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
