---
id: T65
title: Set and logic symbols
status: review
wave: 44
branch: task/T65-set-logic-symbols
depends_on: [T55]
adrs: ["[[ADR-T65-1 Wide relations padded with a thick space]]"]
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

Owner decisions (batch plan, 2026-11-04): rows with the right classes;
`\implies` / `\impliedby` get amsmath's wider spacing, a thick space
each side on top of `Rel` spacing
([[ADR-T65-1 Wide relations padded with a thick space]]); `\lvert \rvert
\lVert \rVert` are `Open` / `Close`. Every character below has a
`core::math_metrics` row (T55 subset).

| Command | Text | Class |
|---------|------|-------|
| `\setminus` | `∖` U+2216 | `Bin` |
| `\complement` | `∁` U+2201 | `Ord` |
| `\varnothing` | `∅` U+2205 | `Ord` |
| `\supset` | `⊃` U+2283 | `Rel` |
| `\subseteq` | `⊆` U+2286 | `Rel` |
| `\supseteq` | `⊇` U+2287 | `Rel` |
| `\subsetneq` | `⊊` U+228A | `Rel` |
| `\supsetneq` | `⊋` U+228B | `Rel` |
| `\nsubseteq` | `⊈` U+2288 | `Rel` |
| `\ni` | `∋` U+220B | `Rel` |
| `\sqcup` | `⊔` U+2294 | `Bin` |
| `\sqcap` | `⊓` U+2293 | `Bin` |
| `\uplus` | `⊎` U+228E | `Bin` |
| `\lnot` | `¬` U+00AC | `Ord` |
| `\veebar` | `⊻` U+22BB | `Bin` |
| `\barwedge` | `⊼` U+22BC | `Bin` |
| `\top` | `⊤` U+22A4 | `Ord` |
| `\bot` | `⊥` U+22A5 | `Ord` |
| `\nexists` | `∄` U+2204 | `Ord` |
| `\implies` | `⟹` U+27F9 | `Rel`, wide |
| `\impliedby` | `⟸` U+27F8 | `Rel`, wide |
| `\vdash` | `⊢` U+22A2 | `Rel` |
| `\models` | `⊨` U+22A8 | `Rel` |
| `\lvert` | `\|` U+007C | `Open` |
| `\rvert` | `\|` U+007C | `Close` |
| `\lVert` | `‖` U+2016 | `Open` |
| `\rVert` | `‖` U+2016 | `Close` |

- **AC-1** Each command of the table parses to one symbol node with
  that text and class, and prints back unchanged —
  `tex_parse_set_and_logic_symbols_have_their_text_and_class`.
- **AC-2** Each draws exactly one glyph of the math face with a
  `math_metrics` row (no text-face fallback, no warning colour) —
  `tex_set_and_logic_symbols_draw_glyphs_of_the_math_face`.
- **AC-3** `p \implies q` is wider than a plain relation by a thick
  space (`WIDE_REL_SPACE` = 5/18 em) on each side: width =
  p + ⟹ + q + 2·`REL_SPACE` + 2·`WIDE_REL_SPACE`, and ⟹ starts at
  p + `REL_SPACE` + `WIDE_REL_SPACE`; the same for `\impliedby`. In a
  script, where relations get no space, the padding stays (amsmath's
  `\;` is in mu) — `tex_layout_implies_pads_a_thick_space_beyond_rel`.
- **AC-4** The others space by class: `A \setminus B` and `p \veebar q`
  get binary space, `A \subseteq B` and `\Gamma \vdash p` relation space,
  `\lnot p` and `\top` none, `\lVert v \rVert` and `\lvert x \rvert` no
  space inside the bars, and a `-` after `\lvert` is unary —
  `tex_layout_set_and_logic_symbols_space_by_class`.
- **AC-5** The fuzzer types every new command (`tests/fuzz.rs`).

UNATTENDED CHOICE: `\varnothing` draws `∅` U+2205 (unicode-math's
mapping, same glyph as `\emptyset`), because the T55 subset has no
stylistic variant and no U+2300; `\barwedge` draws `⊼` U+22BC (same
shape), because the subset lacks unicode-math's U+2305.

## Out of scope

- Big operators (`\bigcup`, …): T66.
- `\not` as a prefix that slashes any relation.
- A distinct `\varnothing` glyph (U+2300 or a stylistic alternate of
  U+2205): not in the T55 subset.
- Other negated relations (`\nsupseteq`, …) beyond the list above.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T65-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T65): …`
- [x] tests — `test(T65): …`
- [x] models — `feat(T65): …`
- [x] behaviour — `feat(T65): …`
- [x] quality — `chore(T65): …` (nothing to change, no commit)
- [x] docs — `docs(T65): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — tests, rows and docs done. 27 `SYMBOLS` rows;
  `\implies` / `\impliedby` are padded in `symbol_box` by
  `WIDE_REL_SPACE` from the `WIDE_RELATIONS` list
  ([[ADR-T65-1 Wide relations padded with a thick space]]), at every
  size. UNATTENDED CHOICE: `\varnothing` → U+2205 (same as
  `\emptyset`) and `\barwedge` → U+22BC, since the T55 subset lacks
  U+2300 / U+2305 (see *Spec*). The quality step changed nothing (no
  `chore` commit). `scripts/check.sh` and `--fuzz` green. No deviation
  from *Files owned*; no tutorial. Manual checks for the owner:
  `$p \implies q$` (visibly wider than `$p \Rightarrow q$`),
  `$A \setminus B$`, `$\lVert v \rVert$`.
