---
id: T66
title: Big set and logic operators
status: in-progress
wave: 45
branch: task/T66-big-operators
depends_on: [T55, T65]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T66 Big set and logic operators

## Goal

Big operators beyond `\sum` and `\prod`, with limits like them:
`\bigcup_{i=1}^{n} A_i`, `\bigcap`, `\bigwedge`, `\bigvee`, `\bigoplus`,
`\bigotimes`, `\bigodot`, `\biguplus`, `\bigsqcup`, `\coprod`.

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
- `Class::BigOp` enlarges a glyph by `BIG_OP_SCALE` 1.3, centred on the
  axis, with limits above and below (`big_operator`, `limits`).
- `⋃ ⋂ ⋀ ⋁ ∐` are in `U+2200–22FF`. `⨀ ⨁ ⨂ ⨄ ⨆` (`U+2A00–2A06`) come from
  T55.

## Proposal (settle in the spec step)

1. Ten `Class::BigOp` rows.
2. Tests: limits sit centred above/below like `\sum`, and the box
   contains the enlarged glyph.

## Spec

Owner decisions (batch plan, 2026-11-04): ten `Class::BigOp` rows, laid
out like `\sum` (enlarged by `BIG_OP_SCALE`, centred on the axis, limits
centred above and below). No ADR. Every character below has a
`core::math_metrics` row (T55 subset, checked: `U+2210`, `U+22C0–22C3`,
`U+2A00–2A06`).

| Command | Text | Class |
|---------|------|-------|
| `\bigcup` | `⋃` U+22C3 | `BigOp` |
| `\bigcap` | `⋂` U+22C2 | `BigOp` |
| `\bigwedge` | `⋀` U+22C0 | `BigOp` |
| `\bigvee` | `⋁` U+22C1 | `BigOp` |
| `\bigoplus` | `⨁` U+2A01 | `BigOp` |
| `\bigotimes` | `⨂` U+2A02 | `BigOp` |
| `\bigodot` | `⨀` U+2A00 | `BigOp` |
| `\biguplus` | `⨄` U+2A04 | `BigOp` |
| `\bigsqcup` | `⨆` U+2A06 | `BigOp` |
| `\coprod` | `∐` U+2210 | `BigOp` |

- **AC-1** Each command of the table parses to one symbol node with
  that text and class, and prints back unchanged —
  `tex_parse_big_set_operators_have_their_text_and_class`.
- **AC-2** Each draws exactly one glyph of the math face with a
  `math_metrics` row (no text-face fallback, no warning colour), at
  `BIG_OP_SCALE`, its middle on the axis, and the box is as wide as the
  enlarged glyph's advance and contains its ink —
  `tex_big_set_operators_draw_enlarged_glyphs_of_the_math_face`.
- **AC-3** `\bigcup_{i=1}^{n} A_i` (and each other command with the
  same limits) puts `n` centred above and `i=1` centred below the
  operator, at least `LIMIT_GAP` from its ink, as `\sum` does —
  `tex_layout_big_set_operator_limits_are_centred_above_and_below`.
- **AC-4** The fuzzer types every new command (`tests/fuzz.rs`).

## Out of scope

- Display-size operators, `\limits`/`\nolimits`.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T66-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T66): …`
- [ ] tests — `test(T66): …`
- [ ] models — `feat(T66): …`
- [ ] behaviour — `feat(T66): …`
- [ ] quality — `chore(T66): …`
- [ ] docs — `docs(T66): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
