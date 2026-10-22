---
id: T36
title: Integrals in formulas
status: review
wave: 24
branch: task/T36-integrals
depends_on: [T33]
adrs: [ADR-T36-1]
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

Decided in [[ADR-T36-1 Integral class with scripts beside the sign]].
`\int` and its kin get a new `Class::Integral`: the glyph is enlarged and
centred on the axis like `\sum` (`BIG_OP_SCALE`), keeps its italic
correction, and `_` / `^` attach as scripts beside it — the superscript
after the italic correction, the subscript that far to the left of the
advance, tucked under the slant (the OpenType `MATH` rule for operators
without limits). For spacing and the unary-minus rule it is an operator,
like `\sum`.

The *Physics symbols* table is added as proposed. One more row:
`\,` → U+2006 (six-per-em space, 1/6 em in the font, already in the
subset), because the goal's own example `\int_0^1 f(x)\,dx` needs it; the
other spacing commands (`\;`, `\:`, `\!`, `\quad`) stay literals.

Correction of a *Known fact*: `core::math_metrics` already has one row per
character of the subset, `∫ ∬ ∭ ∮ ∯` and every physics glyph included
(italic correction of `∫`: 0.332 em). No regeneration is needed;
`scripts/math_metrics.py` and `src/core/math_metrics.rs` stay unchanged.

| AC | Criterion | Test(s) |
|----|-----------|---------|
| AC-1 | `\int \iint \iiint \oint \oiint` parse to `Node::Symbol` with `∫ ∬ ∭ ∮ ∯`, `Class::Integral`; `\intx` stays a literal | `core::tex::tests::tex_parse_integrals_are_symbols` |
| AC-1b | Every command of the *Physics symbols* table and `\,` parses to its symbol, text and class (table-driven); no two `SYMBOLS` rows share a name | `tex_parse_physics_symbols_have_their_text_and_class`, `tex_parse_symbols_come_from_the_table` (uniqueness, existing); `tex_parse_unknown_command_is_literal` now uses `\;` |
| AC-1c | `\,` is a space 1/6 em wide | `tex_layout_thin_space_is_a_sixth_of_an_em` |
| AC-2 | The sign is drawn at `BIG_OP_SCALE`, its middle on the axis, like `∑` | `tex_layout_integral_is_enlarged_like_sum` |
| AC-3 | `\int_a^b`: the superscript starts at advance + italic correction, the subscript at advance − italic correction, both right of the sign's left edge; superscript above, subscript below the baseline; no limits above/below; an operator thin space before an ordinary atom (`\int f`); the box contains all ink | `tex_layout_integral_scripts_sit_beside_the_sign`, `tex_layout_integral_gets_operator_space`, proptests `tex_layout_bounds_contain_every_ink`, `tex_layout_is_finite` (source tokens extended with `\int`, `\oint`, `\,`, `\partial`) |
| AC-4 | Every new glyph is in the math font and in `core::math_metrics` | `shell::render::tests::tex_symbols_all_have_glyphs_in_the_math_font` (existing, iterates `SYMBOLS`, so it covers the new rows) |
| AC-5 | T30/T33 perf thresholds kept with an integral formula in the benchmark; the fuzz text sessions type `\int_…^…` and physics symbols | `tests/perf.rs` `perf_tex_layout_200_blocks_is_under_2ms`, `perf_tessellate_200_formula_blocks_is_under_1ms` (formula list gains an integral); `tests/fuzz.rs` `arb_formula_text` tokens |
| AC-6 | README *Formulas in text blocks* and the feature note list the new commands, grouped as in the table | review |

## Out of scope

- `\limits` / `\nolimits`, display mode, `\int\limits`.
- Other operators (`\bigcup`, `\bigoplus`, …) — the class makes them a
  one-line addition later.
- Accents (`\vec`, `\hat`, `\dot`, `\ddot`, `\bar`) — common in physics
  but a layout construct, not a symbol: [[T39 Accents in formulas]].
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

- [x] spec — `docs(T36): specify integral acceptance criteria`
- [x] tests — `test(T36): add failing tests for integrals`
- [x] models — `feat(T36): add integral and physics symbols, class and metrics`
- [x] behaviour — `feat(T36): place integral limits beside the sign`
- [x] quality — `chore(T36): pass clippy, rustfmt and rustdoc`
- [x] docs — `docs(T36): update feature note and add tutorial`

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
- 2026-10-21 — Spec: `Class::Integral` with scripts beside the sign
  (ADR-T36-1). The *Known fact* about missing metrics rows was wrong:
  the table already has one row per subset character, so
  `math_metrics.rs` and `scripts/math_metrics.py` are unchanged (fontTools
  not needed). `\,` added as a row (U+2006) since the goal's example uses
  it; the old unknown-control-symbol test now uses `\;`.
- 2026-10-21 — Tests, models, behaviour, quality done. No shell change
  needed: `tex_symbols_all_have_glyphs_in_the_math_font` iterates
  `SYMBOLS` and passes with the new rows. Perf (release, one thread):
  tex parse + layout of 200 blocks 1.07 ms (budget 2 ms; 0.87 ms on
  `main`, whose benchmark lacks the new, longer integral formula),
  tessellation 0.18 ms (budget 1 ms). Run in parallel, four unrelated
  10k-shape benchmarks exceed 1 ms from contention; all pass
  single-threaded.
