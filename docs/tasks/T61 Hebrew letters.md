---
id: T61
title: Hebrew letters
status: review
wave: 40
branch: task/T61-hebrew-letters
depends_on: []
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T61 Hebrew letters

## Goal

The Hebrew letters of set theory, as in LaTeX: `\aleph_0`,
`2^{\aleph_0}`, `\beth_1`, `\gimel`, `\daleth`.

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
- `ℵ U+2135, ℶ U+2136, ℷ U+2137, ℸ U+2138` are in the Letterlike
  Symbols range, which is already in both subsets: no font change.
- Owner (2026-11-03): math letters only. Hebrew text (`א–ת` in
  `\text{…}` or plain text) is not wanted. It would need another font
  and right-to-left layout.

## Proposal (settle in the spec step)

1. Four `SYMBOLS` rows, `Class::Ord`, upright (not mapped by alphabets).
2. Scripts attach normally (`\aleph_0`).

## Spec

Settled as the proposal (owner, batch plan 2026-11-04): four `SYMBOLS`
rows, no new parsing or layout code, no ADR.

| Command | Text | Class |
|---------|------|-------|
| `\aleph` | `ℵ` U+2135 | `Ord` |
| `\beth` | `ℶ` U+2136 | `Ord` |
| `\gimel` | `ℷ` U+2137 | `Ord` |
| `\daleth` | `ℸ` U+2138 | `Ord` |

- **AC-1** — each of the four commands parses to one `Node::Symbol` and
  `symbol(name)` gives the text and class of the table.
  Test: `tex_parse_hebrew_letters_have_their_text_and_class`.
- **AC-2** — the letters are drawn upright from the math face: neither
  the italic mapping nor `\mathcal` changes them, and each has a
  `math_metrics` row (so no text-face fallback, no warning colour).
  Test: `tex_hebrew_letters_are_upright_math`.
- **AC-3** — scripts attach as on any ordinary symbol: in `\aleph_0` the
  `0` sits at the letter's advance, lowered and scaled to
  `SCRIPT_SCALE`; in `2^{\aleph_0}` the letter is a scaled superscript
  with its own subscript.
  Test: `tex_layout_hebrew_letter_takes_scripts`.
- **AC-4** — the fuzzer types the four commands (and `\aleph_0`) among
  its formula tokens; the existing fuzz properties stay green.
  Test: `arb_formula_text` tokens in `tests/fuzz.rs`, run by
  `scripts/check.sh` and `scripts/check.sh --fuzz`.

Steps: the *models* step produces nothing (no new types or signatures;
the rows are behaviour) and is skipped.

## Out of scope

- The Hebrew block `U+0590–05FF`, right-to-left text.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T61-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T61): …`
- [x] tests — `test(T61): …`
- [x] models — skipped: no new types or signatures (rows only)
- [x] behaviour — `feat(T61): …`
- [x] quality — skipped: `scripts/check.sh --fuzz` green with no changes
- [x] docs — `docs(T61): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — implemented in the unattended batch run: four `Ord` rows
  after the Greek capitals; `italic` / `calligraphic` already leave
  U+2135–2138 as they are and `math_metrics` already had their rows, so
  no font or mapping change. Models step skipped (no types); quality step
  skipped (`scripts/check.sh --fuzz` green, fmt and clippy clean on the
  first run). No ADR, no tutorial (plan defaults). Fuzzer tokens:
  `\aleph \beth \gimel \daleth \aleph_0`. README and feature note list
  the four commands.
- 2026-11-04 — manual check left for the owner: look at
  `$\aleph_0 < 2^{\aleph_0}$` in the app.
