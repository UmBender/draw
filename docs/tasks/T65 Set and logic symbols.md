---
id: T65
title: Set and logic symbols
status: todo
wave: 44
branch: task/T65-set-logic-symbols
depends_on: [T55]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T65 Set and logic symbols

## Goal

The symbols of sets and logic that are still missing, so that unions,
intersections, and/or/xor and inner products read as in LaTeX:
`\setminus \complement \varnothing \supset \subseteq \supseteq
\subsetneq \supsetneq \nsubseteq \ni \sqcup \sqcap \uplus`, `\lnot \veebar
\barwedge \top \bot \nexists \implies \impliedby \vdash \models`,
and the bracket pairs of inner products and norms (`\lvert \rvert \lVert
\rVert`, with `\langle u, v \rangle` already there).

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
- Present already: `\cup \cap \land \lor \wedge \vee \neg \oplus \in
  \notin \subset \forall \exists \emptyset \langle \rangle \mid \vert \Vert`.
- `⟹ U+27F9` and `⟸ U+27F8` need the T55 long arrows. Most of the rest
  are in Mathematical Operators `U+2200–22FF` (already subset).
- Owner wording: "Union/intersection/and/or/xor/inner product". In LaTeX
  xor is `\oplus` or amssymb's `\veebar` (`⊻`).

## Proposal (settle in the spec step)

1. `SYMBOLS` rows with the right class: relations (`Rel`), binary
   operators (`Bin`), ordinary (`Ord`), `Open`/`Close` for the bars.
2. `\implies`/`\impliedby` get LaTeX's wider spacing (thick space
   around) or plain `Rel` spacing: the spec decides.
3. Test: a table-driven test of name → character → class, like
   `tex_parse_physics_symbols_have_their_text_and_class`.

## Spec

To be written in the spec step.

## Out of scope

- Big operators (`\bigcup`, …): T66.
- `\not` as a prefix that slashes any relation.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T65-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [ ] spec — `docs(T65): …`
- [ ] tests — `test(T65): …`
- [ ] models — `feat(T65): …`
- [ ] behaviour — `feat(T65): …`
- [ ] quality — `chore(T65): …`
- [ ] docs — `docs(T65): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
