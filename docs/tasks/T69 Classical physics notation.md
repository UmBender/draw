---
id: T69
title: Classical physics notation
status: todo
wave: 48
branch: task/T69-classical-physics
depends_on: [T55, T56]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T69 Classical physics notation

## Goal

The notation of classical physics is complete: the triple dot accent
`\dddot{x}`, d'Alembertian `\Box`, `\square`, `\triangle` and `\angle`,
units `\ohm \micro` (`\degree` and `\AA` exist), bold vectors through
`\vb{F}` and unit vectors `\vu{n}`, and the most used `physics`-package
shortcuts: differentials `\dd{x}`, derivatives `\dv{f}{x}`, `\pdv{f}{x}`,
`\abs{x}`, `\norm{v}`, `\order{h^2}`, `\grad \curl`.

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
- Already there: `\partial \nabla \hbar \vec \dot \ddot \hat \propto \sim
  \parallel \perp \otimes \oint \degree \AA`.
- `□ U+25A1`, `△ U+25B3` need T55; `∠ ∡ Ω µ` are already in the subset.
- `physics` redefines `\div` as divergence; in draw `\div` stays `÷` (the
  spec confirms). Brackets do not grow in draw, so `\abs`/`\norm` give
  bars at normal size.
- `\vb`/`\vu` need bold (T56).

## Proposal (settle in the spec step)

1. `\dddot`: an `AccentDef` (three dots; check `U+20DB` or build it from
   dots as `\ddot` does).
2. `\dv{f}{x}` and `\pdv{f}{x}`: two-argument commands drawn as
   `\frac{d f}{d x}` / `\frac{\partial f}{\partial x}`; one-argument forms
   `\dv{x}` → `\frac{d}{dx}`. `\dd{x}` → thin space, upright d, x.
   `\abs`, `\norm`, `\order` wrap their argument. ADR `ADR-T69-1`: the
   set of `physics` commands draw supports, and why the rest are left out.
3. Tests: each expands to the same ink as its long form.

## Spec

To be written in the spec step.

## Out of scope

- `\left`/`\right` growing brackets, `\eval`, matrices.
- SI units (`siunitx`).

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T69-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T69): …`
- [ ] tests — `test(T69): …`
- [ ] models — `feat(T69): …`
- [ ] behaviour — `feat(T69): …`
- [ ] quality — `chore(T69): …`
- [ ] docs — `docs(T69): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
