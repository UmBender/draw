---
id: T69
title: Classical physics notation
status: in-progress
wave: 48
branch: task/T69-classical-physics
depends_on: [T55, T56]
adrs: ["[[ADR-T69-1 Physics shortcuts expand to their long forms]]"]
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

Owner decisions (batch plan, 2026-11-04): `\dddot`; `\Box \square
\triangle \angle`; `\ohm` (Ω), `\micro` (µ); `\vb{F}` (= `\mathbf`),
`\vu{n}` (= `\hat{\mathbf{n}}`); `\dd{x}`, `\dv`, `\pdv` (one- and
two-argument forms), `\abs`, `\norm`, `\order`; `\grad` (∇) and `\curl`
(∇×) only, no `\laplacian`; `\div` stays `÷`; brackets don't grow. The
shortcuts go through one expansion table that T70 reuses
([[ADR-T69-1 Physics shortcuts expand to their long forms]]).

Every glyph drawn has a `core::math_metrics` row (checked: `U+20DB`,
`U+25A1`, `U+25B3`, `U+2220`, `U+2126`, `U+00B5`, `U+2207`, `U+00D7`);
no font change.

| Source | Draws |
|--------|-------|
| `\dddot{x}` | three dots over `x` (`U+20DB`, a combining mark of the math face, as `\ddot` uses `¨`) |
| `\Box`, `\square` | `□` U+25A1, `Ord` |
| `\triangle` | `△` U+25B3, `Ord` |
| `\angle` | `∠` U+2220, `Ord` |
| `\ohm`, `\micro` | `Ω` U+2126 (ohm sign), `µ` U+00B5 (micro sign), `Ord`, upright in every style |
| `\grad` | `∇`, `Ord` (as `\nabla`) |
| `\curl` | the ink of `\nabla\times` |
| `\dd`, `\dd{x}` | `\,\mathrm{d}`, `\,\mathrm{d}x` |
| `\dv{x}`, `\dv{f}{x}` | `\frac{\mathrm{d}}{\mathrm{d}x}`, `\frac{\mathrm{d}f}{\mathrm{d}x}` |
| `\pdv{x}`, `\pdv{f}{x}` | `\frac{\partial}{\partial x}`, `\frac{\partial f}{\partial x}` |
| `\abs{x}`, `\norm{v}` | `\lvert x\rvert`, `\lVert v\rVert` |
| `\order{h^2}` | `\mathcal{O}(h^2)` |
| `\vb{F}`, `\vu{n}` | `\mathbf{F}`, `\hat{\mathbf{n}}` |

- **AC-1** `\Box \square \triangle \angle \ohm \micro \grad` are symbol
  rows with that text and class, each drawn as one unwarned math-face
  glyph with metrics; `\ohm` and `\micro` stay upright inside
  `\mathbf`/`\mathit`; `\div` is still `÷` (`Bin`) —
  `tex_parse_classical_physics_symbols_have_their_text_and_class`,
  `tex_classical_physics_symbols_draw_glyphs_of_the_math_face`,
  `tex_layout_units_stay_upright_in_every_style`,
  `tex_parse_div_stays_division`.
- **AC-2** `\dddot` is an accent (row of `ACCENTS`, one argument, the
  accent parse tests cover it) that draws `U+20DB` from the math face
  above its argument, centred on its ink like `\ddot`, and keeps the
  argument's width — `tex_layout_dddot_draws_three_dots_over_its_argument`,
  `tex_parse_accents_take_one_argument` (its name list gains `dddot`).
- **AC-3** The shortcuts parse as `Node::Command` with their required
  arguments (any argument form) and then each optional argument that is
  a `{group}`: `\dv{x}` has one argument, `\dv{f}{x}` two, `\dd` none,
  `\dd{x}` one, `\dd x` none (then a space and `x`), `\curl` none; all
  print back unchanged —
  `tex_parse_physics_shortcuts_take_their_arguments`.
- **AC-4** A shortcut without a required argument (`\dv`, `\abs`,
  `\vb^2`) is a literal in the warning colour —
  `tex_parse_physics_shortcut_without_required_argument_is_literal`.
- **AC-5** Each shortcut draws the same box (width, ascent, descent,
  italic correction and ink) as its long form, alone and between
  neighbours (`a\abs{x}b`, `\curl\vb{A}`, `\dd{x}\,\dd{y}`), nested
  (`\abs{\vb{F}}`, `\dv{\vb{r}}{t}`) and in scripts (`e^{\abs{x}}`) —
  `tex_layout_physics_shortcuts_match_their_long_forms`.
- **AC-6** A shortcut that is the base of a script is one ordinary atom:
  `\abs{x}^2` draws as `{\lvert x\rvert}^2` —
  `tex_layout_physics_shortcut_as_script_base_is_one_atom`.
- **AC-7** The expansion helper: `expansion(name)` finds the row;
  `expand` returns `None` for any other node; every row's name is
  unique and none is also a symbol, accent, alphabet or other command;
  each template parses with no literal and uses only the slots `#1` …
  `#k` its argument count allows —
  `tex_expansions_are_well_formed`, `tex_expand_replaces_slots_with_arguments`.
- **AC-8** The fuzzer types every new command (`tests/fuzz.rs`); the
  print round trip and totality properties cover them.

Manual checks left for the owner: `$\dv{f}{x}$`, `$\pdv{f}{x}$`,
`$\vb{F} = m\dddot{x}$`.

## Out of scope

- `\left`/`\right` growing brackets, `\eval`, matrices.
- SI units (`siunitx`).
- `\laplacian`, the divergence meaning of `\div`, the `[n]` order of
  `\dv`/`\pdv`/`\dd`, starred forms, `\qty` and the other bracket-sizing
  commands, `\vdot`, `\cross`, `\fdv`, `\var` (ADR-T69-1).

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
