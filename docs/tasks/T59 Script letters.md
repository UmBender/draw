---
id: T59
title: Script letters
status: done
wave: 38
branch: task/T59-script-letters
depends_on: [T54, T55]
adrs: ["[[ADR-T59-1 One script alphabet for mathscr and mathcal]]"]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T59 Script letters

## Goal

`\mathscr{L}`, `\mathscr{F}` and lowercase script (`\mathscr{l}`) in
formulas. `\mathcal` stays capitals only, as in TeX.

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
- Script capitals `U+1D49C–1D4B5` (used by `\mathcal` since T33),
  script lowercase `U+1D4B6–1D4CF` with holes `ℯ U+212F, ℊ U+210A,
  ℴ U+2134`.
- In LaTeX `\mathscr` (rsfs) is a rounder, more formal script than
  `\mathcal`. Latin Modern Math has one script style. Unicode tells the
  two apart only with variation selectors (`U+FE00` chancery,
  `U+FE01` roundhand), which this font may not support. The spec checks.

## Proposal (settle in the spec step)

1. `\mathscr` alphabet row: capitals as `\mathcal`, plus lowercase.
2. If the font has a second script style (variation sequences or a
   stylistic set), the ADR says whether `\mathscr` uses it. Otherwise
   it says that `\mathscr` and `\mathcal` capitals look the same.

## Spec

Settled in [[ADR-T59-1 One script alphabet for mathscr and mathcal]]
(owner, batch plan 2026-11-04, plus the font facts below).

Font facts (T55 subset, which covers all of `U+1D400–1D7FF` and
`U+2100–214F`): script capitals and their eight Letterlike holes are
there; **script lowercase is not** — no glyph in `U+1D4B6–1D4CF`, and
`ℯ U+212F`, `ℊ U+210A`, `ℴ U+2134` are absent. The font has one script
style (no variation sequences, GSUB only `aalt dtls ssty`). By the
plan's shared rule, lowercase script is dropped (*Out of scope*), not
drawn as fallback cells.

| Command | Latin A–Z | Latin a–z | Digits | Greek | Holes | Slant |
|---------|-----------|-----------|--------|-------|-------|-------|
| `\mathscr` | U+1D49C… (as `\mathcal`) | inherit | inherit | inherit | `B E F H I L M R` (as `\mathcal`) | yes |

- **AC-1** `\mathscr` draws every kind exactly as `\mathcal` does:
  script capitals (`\mathscr{LF}` is `ℒ ℱ`), everything else as
  outside — `tex_mathscr_maps_like_mathcal`.
- **AC-2** Lowercase is the surrounding lowercase (math italic `𝑙` for
  `\mathscr{l}`), from the math face, not warned; the font has no
  script lowercase (no `math_metrics` row for `U+1D4B6–1D4CF` or
  `ℯ ℊ ℴ`) — `tex_mathscr_lowercase_falls_back_to_the_outside_style`.
- **AC-3** It parses with one argument (braced or one token), is a
  literal in the warning colour without one, prints back unchanged, and
  draws only math-face glyphs with `math_metrics` rows —
  `tex_parse_mathscr_takes_one_argument`,
  `tex_mathscr_draws_glyphs_of_the_math_face`.
- **AC-4** An accent over a script capital is skewed exactly as under
  `\mathcal` (slanted row, ADR-T56-1) —
  `tex_layout_accent_over_mathscr_as_over_mathcal`.
- **AC-5** Nesting and reach: innermost wins (`\mathscr{\mathrm{L}}` is
  `L`, `\mathbf{\mathscr{L}}` is `ℒ`, `\mathscr{\mathbf{L}}` is `𝐋`);
  `\mathscr{L}_n` keeps an italic n; operators and `\text` unchanged —
  `tex_mathscr_nests_and_leaves_operators_alone`.
- **AC-6** The fuzzer types `\mathscr` (`tests/fuzz.rs`).

## Out of scope

- Adding a second font for rsfs-style script.
- Script lowercase (`\mathscr{l}` as `𝓁`): Latin Modern Math 1.959 has
  no glyph for it (ADR-T59-1); only T55 changes the subset.
- Bold script (`\mathbf{\mathscr{L}}` keeps the innermost rule of T54),
  although the font has bold script capitals.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T59-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T59): …`
- [x] tests — `test(T59): …`
- [x] models — `feat(T59): …`
- [x] behaviour — `feat(T59): …`
- [x] quality — `chore(T59): …`
- [x] docs — `docs(T59): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — spec written (batch run). UNATTENDED CHOICE: the owner
  decision asked for lowercase script (`ℯ ℊ ℴ` holes), but Latin Modern
  Math 1.959 has no script lowercase at all (no `math_metrics` row in
  `U+1D4B6–1D4CF`, none for `ℯ ℊ ℴ`, though the T55 subset covers those
  ranges). Following the plan's shared rule (a glyph the subset lacks is
  dropped, not faked), `\mathscr` ships capitals only, the same row as
  `\mathcal`; `\mathscr{l}` draws an italic l. Recorded in ADR-T59-1.
  The owner's manual check `$\mathscr{l}$` will show `𝑙`, not `𝓁`.
- 2026-11-04 — tests, row and docs done; `\mathscr` is `..MATHCAL` with
  its own name. No deviation from the spec or *Files owned*; the only
  departure from the plan is the dropped lowercase above. The quality
  step only ran rustfmt on the tests. `scripts/check.sh` and `--fuzz`
  green. No tutorial. Manual check for the owner:
  `$\mathscr{L}, \mathscr{l}$` (script L, italic l).
- 2026-11-04 — integrated on `main` (merge `0f00316`), no conflicts; the owner's manual checks and review of the unattended choices are still open.
