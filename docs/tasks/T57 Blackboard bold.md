---
id: T57
title: Blackboard bold
status: review
wave: 36
branch: task/T57-blackboard-bold
depends_on: [T54, T55]
adrs: ["[[ADR-T57-1 One double-struck alphabet for mathbb and mathds]]"]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task, tex]
---

# T57 Blackboard bold

## Goal

Number sets as in LaTeX: `\mathbb{R}`, `\mathbb{N}`, `\mathbb{Z}_p`,
`\mathbb{F}_q`, and `\mathds{1}` (the dsfont indicator function) drawn
with the font's double-struck letters and digits.

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
- Double-struck `U+1D538–1D56B` with holes in Letterlike Symbols
  (`ℂ U+2102, ℍ U+210D, ℕ U+2115, ℙ U+2119, ℚ U+211A, ℝ U+211D, ℤ U+2124`),
  double-struck digits `U+1D7D8–1D7E1`.
- amssymb's `\mathbb` has capitals only; dsfont's `\mathds` has capitals
  plus `1`, `h` and `k`. Unicode (and the font) also has lowercase.

## Proposal (settle in the spec step)

1. `\mathbb` and `\mathds` are one alphabet (capitals, lowercase and
   digits double-struck); the ADR notes that draw accepts more than
   amssymb.
2. The tutorial idea from step 33 ("Try it" 2) becomes real: update that
   tutorial's exercise to point at another alphabet.

## Spec

Settled (owner, batch plan 2026-11-04; a row of
[[ADR-T54-1 Math alphabets as a table of per-kind mappings]], decision in
[[ADR-T57-1 One double-struck alphabet for mathbb and mathds]]).

| Command | Latin A–Z | Latin a–z | Digits | Greek | Holes | Slant |
|---------|-----------|-----------|--------|-------|-------|-------|
| `\mathbb`, `\mathds` | U+1D538… | U+1D552… | U+1D7D8… | inherit | `C→ℂ H→ℍ N→ℕ P→ℙ Q→ℚ R→ℝ Z→ℤ` | no |

- **AC-1** `\mathbb` maps Latin capitals, lowercase and digits to
  double-struck, Greek stays as outside; every hole draws its Letterlike
  character — `tex_mathbb_maps_to_double_struck`,
  `tex_mathbb_holes_map_to_letterlike_symbols`.
- **AC-2** `\mathds` is the same row (`\mathds{1}` → 𝟙) —
  `tex_mathds_is_the_mathbb_row`.
- **AC-3** Both parse with one argument (braced or one token), are
  literals in the warning colour without one, print back unchanged, and
  draw only math-face glyphs with `math_metrics` rows —
  `tex_parse_blackboard_commands_take_one_argument`,
  `tex_blackboard_rows_draw_glyphs_of_the_math_face`.
- **AC-4** An accent over a double-struck letter is centred —
  `tex_layout_accent_centred_over_blackboard_letters`.
- **AC-5** Nesting and reach: `\mathbb{\mathrm{x}}` is upright `x`,
  `\mathbf{\mathbb{R}}` is `ℝ`; subscripts map (`\mathbb{Z}_p` keeps an
  italic p outside); operators and `\text` unchanged —
  `tex_blackboard_nests_and_leaves_operators_alone`.
- **AC-6** `\R` and `\N` stay unknown commands —
  `tex_blackboard_shortcuts_are_unknown`.
- **AC-7** The fuzzer types `\mathbb`, `\mathds` (`tests/fuzz.rs`).
- **AC-8** Tutorial step 33's "Try it" 2 points at an alphabet no task
  adds (`\mathbffrak`, bold fraktur) instead of `\mathbb`.

## Out of scope

- Shortcuts like `\R`, `\N` (not standard LaTeX; owner).
- Double-struck Greek (`ℾ ℽ ℿ ℼ ⅀`) and double-struck italic
  (`ⅅ ⅆ ⅇ ⅈ ⅉ`): no LaTeX alphabet command for them.

## Files owned

- `src/core/tex.rs`
- `tests/fuzz.rs` (formula tokens the fuzzer types)
- `README.md` (*Formulas in text blocks* section only)
- `docs/features/TeX formulas.md`
- new ADR `ADR-T57-*` if the spec needs one, a tutorial if the task has one

## Subtasks (one commit each)

- [x] spec — `docs(T57): …`
- [x] tests — `test(T57): …`
- [x] models — `feat(T57): …`
- [x] behaviour — `feat(T57): …`
- [x] quality — `chore(T57): …` (nothing to change, no commit)
- [x] docs — `docs(T57): …`

## Learning path

None planned; the feature note is enough.

## Log

- 2026-11-03 — note written at the owner's request, part of the TeX symbol
  series T54–T71; ships in 1.5.0.
- 2026-11-04 — spec written (batch run) as the owner decided; ADR-T57-1.
  The tutorial step 33 edit is outside *Files owned* but asked for by
  the owner in the batch plan. UNATTENDED CHOICE: the new exercise
  points at `\mathbffrak` (bold fraktur, U+1D56C, no holes), an alphabet
  no planned task adds.
- 2026-11-04 — tests, row and docs done. Deviation: the T54 guard
  `alphabet("mathbb").is_none()` in
  `tex_alphabet_rows_draw_glyphs_of_the_math_face` now checks an unused
  name (`mathbbb`); one T57 test expectation got the token's leading
  space. `scripts/check.sh` and `--fuzz` green; quality step changed
  nothing, so no `chore` commit. Manual check for the owner:
  `$\mathbb{Z}_p, \mathds{1}_A$`.
- 2026-11-04 — found while gating the docs step: the proptest
  `shell::app::tests::blit_rect_maps_corners_through_cameras` can fail
  at random (f32 cancellation: `rect.width()` is a difference of two
  ~1e5 px screen coordinates, the tolerance is ~1e-3). Minimal input:
  `ox = -7733.8784, oy = 0, z0 = 12.7648535, px = 8913.369, py = 0,
  z1 = 6.8030276, w = 1, h = 1, m = 0`. proptest wrote the seed to
  `proptest-regressions/shell/app.txt` and the docs commit picked it up
  by mistake. The batch run may not delete files (`git rm` / `rm`
  refused), so the next commit empties it to proptest's comment header
  (a seed would keep `scripts/check.sh` red; the file is outside *Files
  owned*). For the owner: `git rm proptest-regressions/shell/app.txt`,
  or keep it for a `fix/…` task that scales the width tolerance with the
  corner magnitude and checks the seed in with the fix.
