---
id: T68
title: Category theory symbols
status: todo
wave: 47
branch: task/T68-category-symbols
depends_on: [T55, T67]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T68 Category theory symbols

## Goal

The arrows and symbols of category theory: `f\colon A \to B`,
monos and epis `\hookrightarrow \twoheadrightarrow \rightarrowtail`,
long arrows `\longrightarrow \longleftarrow \longmapsto
\Longrightarrow \Longleftrightarrow`, parallel arrows
`\rightrightarrows \leftleftarrows \leftrightarrows`, diagonal arrows
`\nearrow \searrow \swarrow \nwarrow`, `\Uparrow \Downarrow`, adjunctions
`F \dashv G`, `\cong`, and the names `\Ob \Mor \op` (`\mathcal{C}^{\op}`),
`\Set \Grp \Ab \Top` as the usual upright `\mathbf` names.

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
- `\to \mapsto \leftarrow \Rightarrow \uparrow \downarrow` exist. Long
  arrows `U+27F5–27FC` and `⇇ ⇉ ↣ ↠ ↪` (`U+21xx`, already subset) need
  T55 only for the long ones.
- `\colon` is punctuation in LaTeX (`f\colon A\to B`, no thick space
  before the colon); draw types `:` as a relation today.
- `\longrightarrow` and `\implies` are listed as missing in the feature
  note's limits.

## Proposal (settle in the spec step)

1. `SYMBOLS` rows (`Rel` for arrows), `\colon` as `Class::Punct`.
2. `\op`, `\Ob`, `\Mor` as `Op` names. `\Set`, `\Grp`, … (bold
   upright, needs T56) are a spec decision: proposed no; write
   `\mathbf{Set}`.
3. Remove `\longrightarrow` from *Limits* in the feature note.

## Spec

To be written in the spec step.

## Out of scope

- Commutative diagrams (`tikz-cd`), `\xrightarrow{f}` with a label:
  in [[T72 Feature proposal]].

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T68-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T68): …`
- [ ] tests — `test(T68): …`
- [ ] models — `feat(T68): …`
- [ ] behaviour — `feat(T68): …`
- [ ] quality — `chore(T68): …`
- [ ] docs — `docs(T68): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
