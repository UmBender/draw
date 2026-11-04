---
id: T68
title: Category theory symbols
status: review
wave: 47
branch: task/T68-category-symbols
depends_on: [T55, T67]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T68 Category theory symbols

## Goal

The arrows and symbols of category theory: `f\colon A \to B`,
monos and epis `\hookrightarrow \twoheadrightarrow \rightarrowtail`,
long arrows `\longrightarrow \longleftarrow \longmapsto
\Longrightarrow \Longleftrightarrow`, parallel arrows
`\rightrightarrows \leftleftarrows \leftrightarrows`, diagonal arrows
`\nearrow \searrow \swarrow \nwarrow`, `\Uparrow \Downarrow`, adjunctions
`F \dashv G`, `\cong`, and the names `\Ob \Mor \op` (`\mathcal{C}^{\op}`),
`\Set \Grp \Ab \Top` as the usual upright `\mathbf` names.

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
- `\to \mapsto \leftarrow \Rightarrow \uparrow \downarrow` exist. Long
  arrows `U+27F5–27FC` and `⇇ ⇉ ↣ ↠ ↪` (`U+21xx`, already subset) need
  T55 only for the long ones.
- `\colon` is punctuation in LaTeX (`f\colon A\to B`, no thick space
  before the colon); draw types `:` as a relation today.
- `\longrightarrow` and `\implies` are listed as missing in the feature
  note's limits.

## Proposal (settle in the spec step)

1. `SYMBOLS` rows (`Rel` for arrows), `\colon` as `Class::Punct`.
2. `\op`, `\Ob`, `\Mor` as `Op` names. `\Set`, `\Grp`, … (bold
   upright, needs T56) are a spec decision: proposed no; write
   `\mathbf{Set}`.
3. Remove `\longrightarrow` from *Limits* in the feature note.

## Spec

Owner decisions (batch plan, 2026-11-04): the arrows and `\dashv` are
`Rel`; `\colon` is `Class::Punct`; `\op \Ob \Mor` are `Op` names, drawn
like `\operatorname{…}`. **No `\Set \Grp \Ab \Top` commands**: the
feature note shows `\mathbf{Set}` (T56). `\longrightarrow` leaves the
feature note's *Limits*. No ADR: `Punct` spacing already exists (a thin
space after, none before), so these are table rows only. Every
character below has a `core::math_metrics` row (T55 subset, checked:
no long arrow is missing).

| Command | Text | Class |
|---------|------|-------|
| `\colon` | `:` U+003A | `Punct` |
| `\hookrightarrow` | `↪` U+21AA | `Rel` |
| `\hookleftarrow` | `↩` U+21A9 | `Rel` |
| `\twoheadrightarrow` | `↠` U+21A0 | `Rel` |
| `\twoheadleftarrow` | `↞` U+219E | `Rel` |
| `\rightarrowtail` | `↣` U+21A3 | `Rel` |
| `\leftarrowtail` | `↢` U+21A2 | `Rel` |
| `\longrightarrow` | `⟶` U+27F6 | `Rel` |
| `\longleftarrow` | `⟵` U+27F5 | `Rel` |
| `\longleftrightarrow` | `⟷` U+27F7 | `Rel` |
| `\Longrightarrow` | `⟹` U+27F9 | `Rel` |
| `\Longleftarrow` | `⟸` U+27F8 | `Rel` |
| `\Longleftrightarrow` | `⟺` U+27FA | `Rel` |
| `\longmapsto` | `⟼` U+27FC | `Rel` |
| `\rightrightarrows` | `⇉` U+21C9 | `Rel` |
| `\leftleftarrows` | `⇇` U+21C7 | `Rel` |
| `\leftrightarrows` | `⇆` U+21C6 | `Rel` |
| `\rightleftarrows` | `⇄` U+21C4 | `Rel` |
| `\nearrow` | `↗` U+2197 | `Rel` |
| `\searrow` | `↘` U+2198 | `Rel` |
| `\swarrow` | `↙` U+2199 | `Rel` |
| `\nwarrow` | `↖` U+2196 | `Rel` |
| `\Uparrow` | `⇑` U+21D1 | `Rel` |
| `\Downarrow` | `⇓` U+21D3 | `Rel` |
| `\Updownarrow` | `⇕` U+21D5 | `Rel` |
| `\updownarrow` | `↕` U+2195 | `Rel` |
| `\dashv` | `⊣` U+22A3 | `Rel` |

Operator names, `Class::Op`, text = the name: `\op \Ob \Mor`.
`\cong` (`≅`, `Rel`) exists already.

- **AC-1** Each symbol of the table parses to one symbol node with that
  text and class, and prints back unchanged —
  `tex_parse_category_symbols_have_their_text_and_class`.
- **AC-2** Each symbol draws exactly one glyph of the math face with a
  `math_metrics` row (no text-face fallback, no warning colour) —
  `tex_category_symbols_draw_glyphs_of_the_math_face`.
- **AC-3** Each operator name parses to one `Op` symbol node whose text
  is its name, prints back unchanged, draws its letters upright from the
  math face, and lays out exactly like `\operatorname{<name>}` (alone,
  before `(\mathcal{C})`, before a letter, with a script) —
  `tex_parse_category_operator_names_are_operator_words`,
  `tex_layout_category_operator_names_match_operatorname`.
- **AC-4** `f\colon a\to b` is punctuation-spaced: no space before the
  colon, a thin space after it, relation space around `\to`; so it is
  narrower than `f:a\to b` (where `:` is a relation) —
  `tex_layout_colon_is_punctuation`.
- **AC-5** Spacing by class: `a \hookrightarrow b`, `a \longrightarrow b`,
  `F \dashv G`, `a \rightrightarrows b` get relation space, and
  `\Longrightarrow` gets **no** extra padding (unlike `\implies`) —
  `tex_layout_category_arrows_space_as_relations`.
- **AC-6** `\mathcal{C}^{\op}` draws the script `𝒞` with an upright `op`
  superscript of the math face —
  `tex_layout_op_superscript_is_upright`.
- **AC-7** The fuzzer types every new command (`tests/fuzz.rs`).

UNATTENDED CHOICE: siblings that use the same subset glyphs and are
standard in the same packages are added next to the listed ones:
`\hookleftarrow \twoheadleftarrow \leftarrowtail \longleftrightarrow
\Longleftarrow \rightleftarrows \Updownarrow \updownarrow`.

## Out of scope

- Commutative diagrams (`tikz-cd`), `\xrightarrow{f}` with a label:
  in [[T72 Feature proposal]].
- `\Set \Grp \Ab \Top` and other category names as commands; write
  `\mathbf{Set}`.
- Arrow variants beyond the table (`\upuparrows`,
  `\Lleftarrow`, `\leadsto`, …), `\xleftarrow`, `\Mapsto`.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T68-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T68): …`
- [x] tests — `test(T68): …`
- [x] models — `feat(T68): …` (no new types: one commit with behaviour)
- [x] behaviour — `feat(T68): …`
- [x] quality — `chore(T68): …` (nothing to change, no commit)
- [x] docs — `docs(T68): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — tests, rows and docs done. 27 symbol rows (26 `Rel`
  arrows and `\dashv`, `\colon` as `Punct`) and 3 `Op` name rows
  (`\op \Ob \Mor`) in `SYMBOLS`; every glyph has a `math_metrics` row
  (no long arrow missing after T55). No new types, so models and
  behaviour are one `feat` commit (as in T66, T67). No ADR: the
  existing `Punct` spacing gives `\colon` no space before and a thin
  space after. UNATTENDED CHOICE: siblings on the same subset glyphs
  added (`\hookleftarrow \twoheadleftarrow \leftarrowtail
  \longleftrightarrow \Longleftarrow \rightleftarrows \Updownarrow
  \updownarrow`, see *Spec*). No `\Set \Grp \Ab \Top` (owner): the
  feature note shows `\mathbf{Set}`. `\longrightarrow` removed from the
  feature note's *Limits*; a *Limits* line for diagrams and
  `\xrightarrow` added. The quality step changed nothing (no `chore`
  commit). `scripts/check.sh` and `scripts/check.sh --fuzz` green. No
  deviation from *Files owned*; no tutorial. Manual checks for the
  owner: `$f\colon A \to B$`, `$F \dashv G$`, `$A \hookrightarrow B$`.
