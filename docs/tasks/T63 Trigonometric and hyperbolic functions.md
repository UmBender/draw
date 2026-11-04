---
id: T63
title: Trigonometric and hyperbolic functions
status: review
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

Settled (owner, batch plan 2026-11-04): eighteen `Class::Op` rows of
`SYMBOLS`, each drawing its own name as an upright word, like `\log`.
No new parsing or layout code, no ADR.

| Group | Commands |
|-------|----------|
| Trigonometric | `\sin \cos \tan \cot \sec \csc` |
| Inverse | `\arcsin \arccos \arctan` |
| Hyperbolic | `\sinh \cosh \tanh \coth` |
| Not in LaTeX's base set | `\arccot \arcsec \arccsc \sech \csch` |

- **AC-1** — each of the eighteen commands parses to one `Node::Symbol`
  and `symbol(name)` gives `(name, Class::Op)`.
  Test: `tex_parse_trig_functions_are_operator_words`.
- **AC-2** — each is drawn as its name, upright from the math face (every
  letter has a `math_metrics` row, nothing warns).
  Test: `tex_trig_functions_are_upright_math_words`.
- **AC-3** — operator spacing as `\log`: the space between the word and
  `x` in `$\sin x$` equals the one in `$\log x$` (`OP_SPACE`), for every
  command; `n\sin n` has a thin space on both sides, and `\sin(x)` has
  none before the bracket.
  Test: `tex_layout_trig_function_spacing_matches_log`.
- **AC-4** — in `\sin^2\theta` the `2` sits beside the word (at its
  advance, raised, scaled to `SCRIPT_SCALE`, not above it as a limit)
  and `θ` follows after `OP_SPACE`.
  Test: `tex_layout_trig_function_power_sits_beside_the_word`.
- **AC-5** — the fuzzer types the commands (and `\sin^2\theta`) among
  its formula tokens; the existing fuzz properties stay green.
  Test: `arb_formula_text` tokens in `tests/fuzz.rs`, run by
  `scripts/check.sh` and `scripts/check.sh --fuzz`.

Steps: the *models* step produces nothing (no new types or signatures;
the rows are behaviour) and is skipped.

## Out of scope

- Function application rules beyond TeX's (no automatic parentheses).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T63-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T63): …`
- [x] tests — `test(T63): …`
- [x] models — skipped: no new types or signatures (rows only)
- [x] behaviour — `feat(T63): …`
- [x] quality — skipped: `scripts/check.sh --fuzz` green with no changes
- [x] docs — `docs(T63): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — implemented in the unattended batch run, on top of
  `task/T61-hebrew-letters`: eighteen `Op` rows after `\lcm` (the
  thirteen LaTeX names plus `\arccot \arcsec \arccsc \sech \csch`, owner
  decision). Every letter they need already had a `math_metrics` row, so
  no font change; spacing and scripts come from the existing `Op` rules.
  Models step skipped (no types); quality step skipped
  (`scripts/check.sh --fuzz` green, fmt and clippy clean on the first
  run). No ADR, no tutorial (plan defaults). The feature note marks the
  five extras as not in LaTeX's base set.
- 2026-11-04 — manual check left for the owner: look at
  `$\sin^2\theta + \cos^2\theta = 1$` in the app.
