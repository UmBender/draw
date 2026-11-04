---
id: T62
title: Complete Greek letters
status: in-progress
wave: 41
branch: task/T62-complete-greek
depends_on: [T54]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T62 Complete Greek letters

## Goal

Every Greek letter LaTeX (with amsmath/amssymb) can write is in draw:
the missing variants `\varkappa` and `\digamma`, the italic capitals
`\varGamma` … `\varOmega`, and the capitals that look like Latin letters
(`\Alpha`, `\Beta`, … as unicode-math defines them).

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
- `SYMBOLS` has the 24 lowercase letters (with `\omicron`) and the
  variants `\varepsilon \vartheta \varpi \varrho \varsigma \varphi`, and
  the 11 capitals with their own shape.
- `italic` already maps `ϰ` (`\varkappa`), but no command reaches it.
- `ϝ U+03DD` (`\digamma`) is in the Greek range of the subset.
- Italic capitals are `U+1D6E2–1D6FA` (in the subset). `\mathnormal`
  reaches them since T54.

## Proposal (settle in the spec step)

1. Add `\varkappa`, `\digamma`, `\Digamma`, `\varGamma` … `\varOmega`
   (amsmath names, italic capitals), `\Alpha \Beta \Epsilon \Zeta \Eta
   \Iota \Kappa \Mu \Nu \Omicron \Rho \Tau \Chi` (upright Greek
   capitals, unicode-math names).
2. Test: every Greek letter of `U+0391–03C9` plus the variants has a
   command, and every command draws a glyph from the math face.

## Spec

Settled by the owner (batch plan 2026-11-04): `SYMBOLS` rows only, no
new parsing or layout code, no ADR. `\digamma` / `\Digamma` are dropped
(see *Out of scope*).

| Commands | Text | Class |
|----------|------|-------|
| `\varkappa` | `ϰ` U+03F0 (the italic mapping draws `𝜘` U+1D718) | `Ord` |
| `\varGamma \varDelta \varTheta \varLambda \varXi \varPi \varSigma \varUpsilon \varPhi \varPsi \varOmega` | the italic capitals `𝛤 𝛥 𝛩 𝛬 𝛯 𝛱 𝛴 𝛶 𝛷 𝛹 𝛺` (U+1D6E4 …) | `Ord` |
| `\Alpha \Beta \Epsilon \Zeta \Eta \Iota \Kappa \Mu \Nu \Omicron \Rho \Tau \Chi` | `Α Β Ε Ζ Η Ι Κ Μ Ν Ο Ρ Τ Χ` (U+0391 …) | `Ord` |

The `\var…` capitals are stored already italic, so no alphabet changes
them (in LaTeX they are `\mathord` symbols of the letters font, which
`\mathrm` does not switch). The upright capitals are Greek capitals like
`\Gamma`: upright by default, italic in `\mathnormal` / `\mathit`.

- **AC-1** — every Greek letter of U+0391–03C9 (without the unassigned
  U+03A2) and the variants `ϵ ϑ ϰ ϕ ϱ ϖ` is the text of some `SYMBOLS`
  row, and every name of the table above parses to one `Node::Symbol`.
  Test: `tex_greek_every_letter_has_a_command`.
- **AC-2** — every Greek command (a row whose text is a Greek or italic
  Greek letter) draws exactly one glyph from the math face, with a
  `math_metrics` row and no warning colour; `\varkappa` draws `𝜘`.
  Test: `tex_greek_commands_draw_glyphs_of_the_math_face`.
- **AC-3** — `\varGamma` … `\varOmega` draw the same glyph as
  `\mathnormal{\Gamma}` … `\mathnormal{\Omega}`, differ from `\Gamma` …,
  and stay italic inside `\mathrm`.
  Test: `tex_var_greek_capitals_are_italic`.
- **AC-4** — `\Alpha` … `\Chi` draw U+0391 … upright, and
  `\mathnormal{\Alpha}` draws the italic U+1D6E2.
  Test: `tex_latin_like_greek_capitals_are_upright`.
- **AC-5** — `\digamma` and `\Digamma` are not commands: drawn as typed
  in the warning colour. Test: `tex_digamma_is_not_a_command`.
- **AC-6** — the fuzzer types `\varkappa \varGamma \Alpha \mathnormal\Alpha`
  among its formula tokens; the fuzz properties stay green.
  Test: `arb_formula_text` tokens in `tests/fuzz.rs`, run by
  `scripts/check.sh` and `scripts/check.sh --fuzz`.

Steps: the *models* step produces nothing (no new types or signatures;
the rows are behaviour) and is skipped.

## Out of scope

- `\digamma`, `\Digamma`: Latin Modern Math has no digamma (U+03DC /
  U+03DD are absent from the font subset and `math_metrics`), and the
  subset is not changed in this batch.

- Upright lowercase Greek (`\upalpha`, `\mathrm{\alpha}`): the spec says
  whether `\mathrm` maps Greek. Default: no, as in LaTeX.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T62-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T62): …`
- [ ] tests — `test(T62): …`
- [ ] models — `feat(T62): …`
- [ ] behaviour — `feat(T62): …`
- [ ] quality — `chore(T62): …`
- [ ] docs — `docs(T62): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — correction to *Known facts*: `ϝ U+03DD` is **not** in the
  font subset (neither is `Ϝ U+03DC`; no `math_metrics` rows), so
  `\digamma` / `\Digamma` are dropped (owner, batch plan).
