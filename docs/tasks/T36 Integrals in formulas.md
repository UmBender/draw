---
id: T36
title: Integrals in formulas
status: todo
wave: 24
branch: task/T36-integrals
depends_on: [T33]
adrs: []
feature: "[[TeX formulas]]"
tutorial: "[[36 Operators whose limits sit beside them]]"
tags: [task]
---

# T36 Integrals in formulas

## Goal

`$\int_0^1 f(x)\,dx$` typesets an integral sign, drawn large like `\sum`,
with its limits to the right of the sign (sub- and superscript), as in
inline LaTeX. Today `\int` is an unknown command and is drawn as the
literal text `\int`. `\iint` and `\oint` come with it.

## Known facts (2026-10-21)

- `core::tex` has `\sum` and `\prod` as `Class::BigOp` (enlarged glyph,
  limits above and below) and `\min`, `\max`, `\lim` as `Class::Limits`
  (`src/core/tex.rs:316`). An unknown command is a literal
  (`tex_parse_unknown_command_is_literal`).
- The math font subset already covers `U+2200–22FF`, so `∫` (U+222B),
  `∬` (U+222C) and `∮` (U+222E) are in `LatinModernMath-draw-subset.otf`;
  `core::math_metrics` has no rows for them yet, so
  `scripts/math_metrics.py` (needs fontTools) must be rerun.
- The integral sign is slanted: a superscript placed at the advance
  would collide with the top of the sign, so the italic correction
  matters (ADR-T33-1).

## Proposal (settle in the spec step)

- **Class.** A new `Class::Integral` (or a `limits: bool` flag on
  `BigOp`): the glyph is enlarged like `BigOp`, but `_` / `^` attach as
  scripts beside it — subscript tucked under the slant, superscript after
  the italic correction — instead of above and below.
- **Symbols.** `\int` → `∫`, `\iint` → `∬`, `\oint` → `∮`, all
  `Class::Integral`. Spacing: an operator atom, like `\sum` (thin space
  after it before an ordinary atom).
- **Metrics.** Rows for the three glyphs in `core::math_metrics`,
  regenerated from the embedded font; the shell test that compares the
  table with the font keeps holding.
- **Out of the subset.** `\limits` / `\nolimits` and display-size
  variants from the `MATH` table.

## Spec

To be written in the spec step: acceptance criteria and the tests that
prove them. Expected:

- **AC-1 — Symbols.** `\int`, `\iint`, `\oint` parse to their symbols;
  `\intx` stays a literal.
- **AC-2 — Size.** The sign is enlarged by the same factor as `\sum`.
- **AC-3 — Limits beside.** `\int_a^b` puts `a` and `b` to the right of
  the sign, the superscript to the right of the italic correction, and
  the box contains all ink (existing proptests).
- **AC-4 — Glyphs and metrics.** The three glyphs are in the math font
  and in `core::math_metrics` (existing shell tests extended).
- **AC-5 — Budgets and fuzz.** T30/T33 perf thresholds kept; the fuzz
  text sessions also type `\int_…^…`.
- **AC-6 — Docs.** README *Formulas in text blocks* and the feature note
  list the new commands.

## Out of scope

- `\limits` / `\nolimits`, display mode, `\int\limits`.
- Other operators (`\bigcup`, `\bigoplus`, …) — the class makes them a
  one-line addition later.

## Files owned

- `src/core/tex.rs`, `src/core/math_metrics.rs`, `scripts/math_metrics.py`
- `src/shell/render.rs`, `src/shell/glyphs.rs` (tests only, if needed)
- `tests/fuzz.rs`, `tests/perf.rs`
- `README.md` — section *Formulas in text blocks* only
- feature note `docs/features/TeX formulas.md`, tutorial
  `docs/tutorials/36 Operators whose limits sit beside them.md`, new
  `ADR-T36-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T36): specify integral acceptance criteria`
- [ ] tests — `test(T36): add failing tests for integrals`
- [ ] models — `feat(T36): add integral symbols, class and metrics`
- [ ] behaviour — `feat(T36): place integral limits beside the sign`
- [ ] quality — `chore(T36): pass clippy, rustfmt and rustdoc`
- [ ] docs — `docs(T36): update feature note and add tutorial`

## Learning path

Step 36 — requires step 30 (TeX formulas) and step 33 (math font).

## Log

- 2026-10-21 — Created from the owner's question whether `\int` is
  supported (it is not: drawn as literal text). Checked before writing:
  the glyphs are already in the font subset; the metrics table needs
  regenerating. Runs in wave 24 in parallel with T37 and T38 (disjoint
  files; README sections differ).
