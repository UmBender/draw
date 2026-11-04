---
id: T60
title: Sans-serif and typewriter letters
status: todo
wave: 39
branch: task/T60-sans-and-mono
depends_on: [T54, T55]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T60 Sans-serif and typewriter letters

## Goal

`\mathsf{G}` (sans-serif) and `\mathtt{x}` (typewriter) in formulas,
for categories, matrices or code names, with sans-serif and monospace
digits.

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
- Sans `U+1D5A0–1D5D3`, sans digits `U+1D7E2–1D7EB`; monospace
  `U+1D670–1D6A3`, monospace digits `U+1D7F6–1D7FF`. Bold and italic
  sans also exist (`U+1D5D4–1D66F`).
- The owner's list said `matht`. Read as `\mathtt`; the spec confirms
  this.

## Proposal (settle in the spec step)

1. `\mathsf` and `\mathtt` alphabet rows.
2. The spec decides whether `\mathsf` under `\boldmath`/`\bm` maps to
   bold sans (Unicode has it) or stays regular sans.

## Spec

To be written in the spec step.

## Out of scope

- Sans-serif Greek.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T60-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T60): …`
- [ ] tests — `test(T60): …`
- [ ] models — `feat(T60): …`
- [ ] behaviour — `feat(T60): …`
- [ ] quality — `chore(T60): …`
- [ ] docs — `docs(T60): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
