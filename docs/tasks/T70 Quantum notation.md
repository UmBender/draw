---
id: T70
title: Quantum notation
status: todo
wave: 49
branch: task/T70-quantum-notation
depends_on: [T69]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T70 Quantum notation

## Goal

Dirac notation and the operators of quantum mechanics have short forms
from the `physics` package: `\bra{\psi}`, `\ket{0}`, `\braket{\phi}{\psi}`,
`\ketbra{0}{1}`, `\expval{H}`, `\mel{n}{H}{m}`, commutators `\comm{A}{B}`
and `\acomm{A}{B}`, `\Tr`/`\tr`, the adjoint `A^\dag`, and `\otimes`
products of kets.

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
- Today a bra-ket is written by hand: `\langle a \mid b \rangle`.
  `\dagger`, `\otimes`, `\hat`, `\hbar` exist.
- `\dag` is a LaTeX text symbol; in math `^\dagger` is the usual form.

## Proposal (settle in the spec step)

1. Commands expand to brackets, bars and their arguments
   (`\braket{a}{b}` → `⟨a|b⟩` with `\mid`-like bars). One shared expansion
   helper, introduced in T69.
2. `\dag` (`†`, `Ord`), `\Tr`/`\tr` (`Op`).
3. Tests: each command's ink equals its hand-written form.

## Spec

To be written in the spec step.

## Out of scope

- Growing brackets around tall arguments (`\bra{\frac{1}{2}}` keeps
  normal-size brackets).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T70-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T70): …`
- [ ] tests — `test(T70): …`
- [ ] models — `feat(T70): …`
- [ ] behaviour — `feat(T70): …`
- [ ] quality — `chore(T70): …`
- [ ] docs — `docs(T70): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
