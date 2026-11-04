---
id: T70
title: Quantum notation
status: in-progress
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

Owner decisions (batch plan, 2026-11-04): `\bra \ket \braket \ketbra
\expval \mel \comm \acomm` are rows of T69's expansion table
([[ADR-T69-1 Physics shortcuts expand to their long forms]]); `\dag` is
`†` (`Ord`); `\Tr` is a new operator name (`Op`); `\tr` already exists
(T67) and is reused. No new ADR: the expansion mechanism is T69's, and
these are table rows.

Every glyph drawn already has a `core::math_metrics` row (`⟨ ⟩ | † [ ]
{ }`, the letters of `Tr`); no font change.

| Source | Draws (long form) |
|--------|-------------------|
| `\bra{a}` | `\langle a\rvert` |
| `\ket{a}` | `\lvert a\rangle` |
| `\braket{a}`, `\braket{a}{b}` | `\langle a\vert a\rangle`, `\langle a\vert b\rangle` |
| `\ketbra{a}`, `\ketbra{a}{b}` | `\lvert a\rangle\langle a\rvert`, `\lvert a\rangle\langle b\rvert` |
| `\expval{A}`, `\expval{A}{\psi}` | `\langle A\rangle`, `\langle \psi\vert A\vert \psi\rangle` |
| `\mel{n}{H}{m}` | `\langle n\vert H\vert m\rangle` |
| `\comm{A}{B}` | `[A,B]` |
| `\acomm{A}{B}` | `\{A,B\}` |
| `\dag` | `†`, `Ord` |
| `\Tr` | `Tr`, `Op` (as `\tr`) |

The second argument of `\braket`, `\ketbra` and `\expval` is optional
(braced, as T69's optional arguments), as in `physics`. The inner bars
are `\vert` (ordinary, no space), as `physics` draws them with
`\middle\vert`; the outer bars are `\lvert`/`\rvert`.

- **AC-1** `\dag` and `\Tr` are symbol rows with that text and class,
  drawn unwarned from the math face; `\tr` is unchanged and `\dagger`
  stays `Bin` —
  `tex_parse_quantum_symbols_have_their_text_and_class`,
  `tex_quantum_symbols_draw_glyphs_of_the_math_face`.
- **AC-2** The bra–ket and commutator commands are rows of `EXPANSIONS`
  with the argument counts of the table, take their arguments as T69
  shortcuts do, and print back unchanged —
  `tex_parse_quantum_shortcuts_take_their_arguments`,
  `tex_expansions_are_well_formed` (its name list gains the eight).
- **AC-3** Without a required argument (`\ket`, `\mel{n}{H}`,
  `\comm{A}`) the command is a literal in the warning colour —
  `tex_parse_quantum_shortcut_without_required_argument_is_literal`.
- **AC-4** Each command draws the same box and ink as its hand-written
  form, alone, between neighbours (`\ket{0}\otimes\ket{1}`,
  `\comm{A}{B} = AB - BA`), nested (`\bra{\vb{r}}`,
  `\expval{\comm{A}{B}}`) and with scripts (`A^\dag`, `\ket{\psi}^\dag`,
  `\Tr\rho^2`) — `tex_layout_quantum_shortcuts_match_their_long_forms`.
- **AC-5** The fuzzer types every new command (`tests/fuzz.rs`); the
  print round trip and totality properties cover them.

Manual checks left for the owner: `$\braket{\phi}{\psi}$`,
`$\mel{n}{H}{m}$`, `$\comm{A}{B}$`.

## Out of scope

- Growing brackets around tall arguments (`\bra{\frac{1}{2}}` keeps
  normal-size brackets).
- Starred forms (`\braket*`), `\dyad`, `\ev`, `\matrixel`, `\pb`
  (Poisson bracket) and the other `physics` aliases.

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
