---
id: T67
title: Algebra and group theory symbols
status: review
wave: 46
branch: task/T67-algebra-symbols
depends_on: [T55, T64]
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T67 Algebra and group theory symbols

## Goal

The most used notation of abstract algebra and group theory:
normal subgroups `N \trianglelefteq G` (`\lhd \unlhd \rhd \unrhd
\triangleleft \triangleright \ntriangleleft`), semidirect products
`N \rtimes H` (`\ltimes`), wreath `\wr`, `\odot \ominus \oslash \boxplus
\boxtimes \amalg \diamond`, `G/N`, `\langle g \rangle`, `[G:H]`, and the
upright names `\Hom \End \Aut \Inn \Gal \Ker \im \coker \id \sgn \tr
\rank \char`.

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
- `⊲ ⊴ ⊳ ⊵ ⋊ ⋉ ≀ ⊙ ⊖ ⊘ ⊞ ⊠ ⋄` are in `U+2200–22FF`; `⨿ U+2A3F` comes
  from T55.
- `\Hom`, `\End`, … are not LaTeX built-ins: they are usual
  `\DeclareMathOperator`s. draw defines them, built on `\operatorname`
  (T64). `\char` clashes with a TeX primitive; the spec picks a name.
- Quotients, cosets, generated subgroups and indices need nothing
  new; the feature note shows them.

## Proposal (settle in the spec step)

1. `SYMBOLS` rows (`Rel` for the triangle relations, `Bin` for the
   products) and `Op` rows for the names.
2. The spec makes the final list. Rule: a symbol goes in if it shows up
   in a standard first-year algebra text (Dummit–Foote, Lang).

## Spec

Owner decisions (batch plan, 2026-11-04): the triangle relations are
`Rel`, the products `Bin`, `⨿` comes from the T55 subset; the names are
`Op` rows, drawn exactly like `\operatorname{…}` (T64). **No `\char`**
(it would redefine a TeX primitive): the feature note shows
`\operatorname{char}`. List rule: the symbol shows up in a standard
first-year algebra text. `\tr` is the one T70 reuses. No ADR needed:
these are table rows only. Every character below has a
`core::math_metrics` row (T55 subset).

| Command | Text | Class |
|---------|------|-------|
| `\lhd` | `⊲` U+22B2 | `Rel` |
| `\rhd` | `⊳` U+22B3 | `Rel` |
| `\unlhd` | `⊴` U+22B4 | `Rel` |
| `\unrhd` | `⊵` U+22B5 | `Rel` |
| `\triangleleft` | `⊲` U+22B2 | `Rel` |
| `\triangleright` | `⊳` U+22B3 | `Rel` |
| `\trianglelefteq` | `⊴` U+22B4 | `Rel` |
| `\trianglerighteq` | `⊵` U+22B5 | `Rel` |
| `\ntriangleleft` | `⋪` U+22EA | `Rel` |
| `\ntriangleright` | `⋫` U+22EB | `Rel` |
| `\rtimes` | `⋊` U+22CA | `Bin` |
| `\ltimes` | `⋉` U+22C9 | `Bin` |
| `\wr` | `≀` U+2240 | `Bin` |
| `\odot` | `⊙` U+2299 | `Bin` |
| `\ominus` | `⊖` U+2296 | `Bin` |
| `\oslash` | `⊘` U+2298 | `Bin` |
| `\boxplus` | `⊞` U+229E | `Bin` |
| `\boxtimes` | `⊠` U+22A0 | `Bin` |
| `\amalg` | `⨿` U+2A3F | `Bin` |
| `\diamond` | `⋄` U+22C4 | `Bin` |

Operator names, `Class::Op`, text = the name: `\Hom \End \Aut \Inn \Gal
\Ker \im \coker \id \sgn \tr \rank`.

- **AC-1** Each symbol of the table parses to one symbol node with that
  text and class, and prints back unchanged —
  `tex_parse_algebra_symbols_have_their_text_and_class`.
- **AC-2** Each symbol draws exactly one glyph of the math face with a
  `math_metrics` row (no text-face fallback, no warning colour) —
  `tex_algebra_symbols_draw_glyphs_of_the_math_face`.
- **AC-3** Each operator name parses to one `Op` symbol node whose text
  is its name, prints back unchanged, and draws its letters upright from
  the math face, as wide as the word —
  `tex_parse_algebra_operator_names_are_operator_words`,
  `tex_algebra_operator_names_are_upright_math_words`.
- **AC-4** Each operator name lays out exactly like
  `\operatorname{<name>}`, alone, before `(A, B)`, before a letter and
  with a script (`\Hom(A, B)`, `\tr x`, `\Aut^2`) —
  `tex_layout_algebra_operator_names_match_operatorname`.
- **AC-5** Spacing by class: `N \trianglelefteq G`, `N \lhd G`,
  `N \ntriangleleft G` get relation space; `N \rtimes H`, `a \wr b`,
  `a \odot b`, `a \amalg b` binary space —
  `tex_layout_algebra_symbols_space_by_class`.
- **AC-6** The fuzzer types every new command (`tests/fuzz.rs`).

UNATTENDED CHOICE: `\triangleleft` / `\triangleright` draw `⊲` / `⊳` as
`Rel`, like `\lhd` / `\rhd` (LaTeX's are small binary triangles, but the
owner asked for `Rel` triangles and the T55 subset has no U+25C3 /
U+25B9). `\ntriangleright` is added next to `\ntriangleleft` (same
subset, the pair is used together).

## Out of scope

- Commutative diagrams, extensible arrows (T68 and [[T72 Feature proposal]]).
- `\char` as a command (TeX primitive); use `\operatorname{char}`.
- `\ntrianglelefteq`, `\ntrianglerighteq`, `\vartriangleleft`, `\boxminus`,
  `\boxdot` and other variants beyond the list above.
- Quotients `G/N`, cosets, `\langle g \rangle`, `[G:H]`: nothing new,
  shown in the feature note.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T67-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T67): …`
- [x] tests — `test(T67): …`
- [x] models — `feat(T67): …` (no new types: one commit with behaviour)
- [x] behaviour — `feat(T67): …`
- [x] quality — `chore(T67): …` (nothing to change, no commit)
- [x] docs — `docs(T67): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — tests, rows and docs done. 20 symbol rows and 12 `Op`
  name rows in `SYMBOLS`; no new types, so models and behaviour are one
  `feat` commit (as in T66). UNATTENDED CHOICE: `\triangleleft` /
  `\triangleright` are `Rel` and draw `⊲ ⊳` (see *Spec*);
  `\ntriangleright` added next to `\ntriangleleft`. No ADR. The quality
  step changed nothing (no `chore` commit). `scripts/check.sh` green;
  the first `--fuzz` run failed in `snapshot_round_trip_continues_equal`
  (and its pen-tap / shape-text variants), outside T67 (no formula
  typed): "own text rejected: line 26: text size out of range" after
  Text tool, click, `Ctrl+[`, `Scroll { delta: 1771.5367 }` at the
  origin, click, Space, click and drag at `x = 1983.8613`. proptest
  wrote seed `cc 31c20d58…` to `tests/fuzz.proptest-regressions`; it was
  removed again (a seed keeps the gates red and the fix is not this
  task's) and the second `--fuzz` run was green. For the owner: a
  `fix/…` task for the snapshot text-size range. No deviation from
  *Files owned*; no tutorial. Manual checks for the owner:
  `$N \trianglelefteq G$`, `$N \rtimes H$`, `$\Hom(A, B)$`.
