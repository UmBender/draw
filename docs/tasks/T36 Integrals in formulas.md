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
literal text `\int`. `\iint`, `\iiint`, `\oint` and `\oiint` come with
it.

The task also adds the symbol commands most used in physics (see
*Physics symbols* below), so that common formulas — Maxwell's equations,
Schrödinger's equation, bra–ket notation, thermodynamics — typeset
without literals.

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
- **Symbols.** `\int` → `∫`, `\iint` → `∬`, `\iiint` → `∭`, `\oint` →
  `∮`, `\oiint` → `∯`, all `Class::Integral`. Spacing: an operator atom, like `\sum` (thin space
  after it before an ordinary atom).
- **Physics symbols.** One `SYMBOLS` row each, LaTeX's atom class.
  Every character is inside the current math-font subset ranges
  (`U+00A0–017F`, `U+2000–206F`, `U+2100–214F`, `U+2190–21FF`,
  `U+2200–22FF`, `U+27E8–27E9`), so the subset command does not change:

  | Group | Commands |
  |-------|----------|
  | Calculus, vectors (`Ord`) | `\partial` ∂, `\nabla` ∇ |
  | Letters, constants (`Ord`) | `\hbar` ℏ, `\ell` ℓ, `\Re` ℜ, `\Im` ℑ, `\AA` Å, `\degree` ° |
  | Logic, sets (`Ord`) | `\forall` ∀, `\exists` ∃, `\emptyset` ∅, `\prime` ′ |
  | Dots, bars (`Ord`) | `\vdots` ⋮, `\ddots` ⋱, `\vert` \|, `\Vert` ‖ |
  | Relations (`Rel`) | `\propto` ∝, `\sim` ∼, `\simeq` ≃, `\cong` ≅, `\ll` ≪, `\gg` ≫, `\lesssim` ≲, `\gtrsim` ≳, `\doteq` ≐, `\coloneqq` ≔, `\parallel` ∥, `\perp` ⊥, `\mid` ∣ |
  | Arrows (`Rel`) | `\rightarrow` →, `\leftrightarrow` ↔, `\Leftarrow` ⇐, `\Leftrightarrow` ⇔, `\uparrow` ↑, `\downarrow` ↓, `\mapsto` ↦, `\rightleftharpoons` ⇌ |
  | Binary (`Bin`) | `\mp` ∓, `\otimes` ⊗, `\wedge` ∧, `\vee` ∨, `\circ` ∘, `\ast` ∗, `\star` ⋆, `\bullet` ∙, `\dagger` † |
  | Brackets | `\langle` ⟨ (`Open`), `\rangle` ⟩ (`Close`) |

  Already present and unchanged: the Greek alphabet, `\infty`, `\pm`,
  `\times`, `\cdot`, `\approx`, `\equiv`, `\le`/`\ge`/`\ne`, `\to`,
  `\Rightarrow`, `\iff`, `\sum`, `\prod`, `\lim`, `\ln`, `\log`.
  Not added, because the glyph is outside the subset: `\longrightarrow`
  (U+27F6), `\implies` (U+27F9), `\triangle` (U+25B3); a later task can
  widen the subset.
- **Metrics.** Rows for the new glyphs in `core::math_metrics`,
  regenerated from the embedded font; the shell test that compares the
  table with the font keeps holding.
- **Out of the subset.** `\limits` / `\nolimits` and display-size
  variants from the `MATH` table.

## Spec

To be written in the spec step: acceptance criteria and the tests that
prove them. Expected:

- **AC-1 — Symbols.** `\int`, `\iint`, `\iiint`, `\oint`, `\oiint`
  parse to their symbols; `\intx` stays a literal.
- **AC-1b — Physics symbols.** Every command of the *Physics symbols*
  table parses to its character with its class (one table-driven test);
  no two `SYMBOLS` rows share a name.
- **AC-2 — Size.** The sign is enlarged by the same factor as `\sum`.
- **AC-3 — Limits beside.** `\int_a^b` puts `a` and `b` to the right of
  the sign, the superscript to the right of the italic correction, and
  the box contains all ink (existing proptests).
- **AC-4 — Glyphs and metrics.** Every new glyph is in the math font
  and in `core::math_metrics` (existing shell tests extended).
- **AC-5 — Budgets and fuzz.** T30/T33 perf thresholds kept; the fuzz
  text sessions also type `\int_…^…`.
- **AC-6 — Docs.** README *Formulas in text blocks* and the feature note
  list the new commands (grouped as in the table above).

## Out of scope

- `\limits` / `\nolimits`, display mode, `\int\limits`.
- Other operators (`\bigcup`, `\bigoplus`, …) — the class makes them a
  one-line addition later.
- Accents (`\vec`, `\hat`, `\dot`, `\ddot`, `\bar`) — common in physics
  but a layout construct, not a symbol; candidate for its own task.
- Widening the math-font subset (see the *Not added* list).

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
- [ ] models — `feat(T36): add integral and physics symbols, class and metrics`
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
- 2026-10-21 — Scope widened at the owner's request: "I want all of the
  most common used symbols from physics to be added." Added the
  *Physics symbols* table (all inside the current font subset), the five
  integral signs and AC-1b; accents and subset growth left out of scope.
