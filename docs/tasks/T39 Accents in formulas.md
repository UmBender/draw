---
id: T39
title: Accents in formulas
status: in-progress
wave: 25
branch: task/T39-accents
depends_on: [T36]
adrs: ["[[ADR-T39-1 Accents from spacing glyphs with a slant skew]]"]
feature: "[[TeX formulas]]"
tutorial: "[[39 Accents that sit on their letters]]"
tags: [task]
---

# T39 Accents in formulas

## Goal

`$\vec{F} = m\ddot{x}$`, `$\hat{n}$`, `$\bar{v}$` and `$\overline{AB}$`
typeset an accent centred over its argument, as in inline LaTeX. These
are the accents physics notes use most: vectors, time derivatives, unit
vectors and averages. Today `\vec` and the others are unknown commands,
drawn as literal text.

## Known facts (2026-10-21)

- `core::tex` already parses one-argument commands (`\sqrt`, `\pmod`,
  `\mathcal`: `self.args(1, end)`, `src/core/tex.rs:881`) and lays out
  boxes with an ascent, a descent and `Ink::Rule` for bars
  (`fn root`, `src/core/tex.rs:1344`).
- The math font subset has no combining accents (`U+0300–036F`,
  `U+20D0–20FF`) and no spacing modifiers (`U+02B0–02FF`), but it has
  spacing forms that can serve as accents, all with rows in
  `core::math_metrics`: `^` (U+005E), `~` (U+007E), `¨` (U+00A8),
  `¯` (U+00AF), `·` (U+00B7), `→` (U+2192).
- The `MATH` table, with its top-accent attachment points, is dropped
  from the subset (ADR-T33-1), so the horizontal placement over an
  italic letter has to be approximated.
- T36 owns `src/core/tex.rs` in wave 24; this task runs after it.

## Proposal (settle in the spec step)

- **Commands.** `\vec` (arrow), `\hat` (`^`), `\tilde` (`~`), `\bar`
  (short rule or `¯`), `\dot` (one dot), `\ddot` (two dots, `¨`),
  `\overline` (a rule as wide as the argument). Each takes one argument;
  a missing argument keeps today's literal fallback.
- **Glyph or rule.** `\bar` and `\overline` draw an `Ink::Rule` (bar
  thickness `RULE`), so their width follows the argument; the others use
  the glyphs above, scaled down where needed (`\vec`'s arrow), with no
  font change. `\widehat` / `\widetilde` stay out.
- **Placement.** The accent's box sits a fixed gap above the argument's
  ascent, centred on the argument's ink. For a single italic letter the
  centre is shifted right by a skew taken from the slant (an
  approximation of the dropped top-accent attachment; recorded in an
  ADR). The result's ascent grows to contain the accent; width and
  descent are the argument's.
- **Scripts.** `\vec{F}_x`, `\dot{x}^2` attach scripts to the accented
  box like to any nucleus.

## Spec

Decided in [[ADR-T39-1 Accents from spacing glyphs with a slant skew]].
An accent is a `Node::Command` with one argument, like `\sqrt` — no new
node kind, so printing stays lossless for free. A new table
`tex::ACCENTS` (name → `Mark`) lists the commands; the parser and the
layout look accents up there, and the shell test reads the glyphs from
it.

| Command | `Mark` | Drawn as |
|---------|--------|----------|
| `\vec` | `Glyph('→', VEC_SCALE)` | the arrow at half size |
| `\hat` | `Glyph('^', 1.0)` | U+005E |
| `\tilde` | `Glyph('~', 1.0)` | U+007E |
| `\dot` | `Glyph('·', 1.0)` | U+00B7 |
| `\ddot` | `Glyph('¨', 1.0)` | U+00A8 |
| `\bar` | `Bar` | a rule `ACCENT_BAR` long (the macron's ink width), centred |
| `\overline` | `Overline` | a rule from the argument's left to its advance |

Placement (all lengths times the argument's scale):

- The mark's ink bottom sits `ACCENT_GAP` above the argument's ascent; a
  rule's lower edge likewise (rule width `RULE`).
- Glyphs and `\bar` are centred on the argument's ink span; when the
  argument is one italic letter (a letter, or a lowercase Greek symbol,
  that the style maps to another glyph) the centre moves right by
  `ACCENT_SKEW × ascent` — the approximation of the dropped top-accent
  attachment.
- The accented box has the argument's width, descent and italic
  correction; its ascent reaches the mark's top. It is `Class::Ord`, so
  scripts attach to it like to any nucleus.

| AC | Criterion | Test(s) |
|----|-----------|---------|
| AC-1 | Every `ACCENTS` command parses to `Node::Command` with one argument, braced or one token (with its lead whitespace); no two rows share a name, nor a `SYMBOLS` name | `core::tex::tests::tex_parse_accents_take_one_argument` |
| AC-1b | `\vecx`, `\vec` at the end and `\hat^2` stay literals | `tex_parse_accent_without_argument_is_literal` |
| AC-1c | Lossless printing and totality with accents in the source | proptest `tex_parse_never_panics_and_prints_back_the_source` (`arb_source` tokens gain `\vec \hat \bar \dot \ddot \tilde \overline`) |
| AC-2 | Every mark's ink lies `ACCENT_GAP` above the argument's ascent | `tex_layout_accent_sits_above_its_argument` |
| AC-2b | Over an upright or multi-letter argument the mark is centred on the argument's ink | `tex_layout_accent_is_centred_on_an_upright_argument` |
| AC-2c | Over one italic letter the centre is skewed right by `ACCENT_SKEW × ascent` | `tex_layout_accent_over_an_italic_letter_is_skewed` |
| AC-2d | `\bar` is one rule `ACCENT_BAR` long; `\overline` one rule from 0 to the argument's width | `tex_layout_bar_is_a_short_rule`, `tex_layout_overline_spans_its_argument` |
| AC-3 | The accented box keeps the argument's width and descent; its ascent contains the mark | `tex_layout_accent_keeps_the_argument_width_and_descent`, proptests `tex_layout_bounds_contain_every_ink`, `tex_layout_is_finite` |
| AC-3b | `\vec{F}_x`, `\dot{x}^2` put their scripts after the accented box; an accent in a script is drawn at script scale | `tex_layout_accent_takes_scripts` |
| AC-4 | Every glyph an accent draws is in the math font and in `core::math_metrics` | `shell::render::tests::tex_symbols_all_have_glyphs_in_the_math_font` (extended with `ACCENTS`) |
| AC-5 | T30/T33 perf thresholds kept with an accent formula in the benchmark; the fuzz text sessions type accents | `tests/perf.rs` `formula_texts` gains `$\vec{F} = m\ddot{x}$ and $\overline{AB}$`; `tests/fuzz.rs` `arb_formula_text` tokens |
| AC-6 | README *Formulas in text blocks* and the feature note list the accent commands | review |

## Out of scope

- Wide accents (`\widehat`, `\widetilde`, `\overrightarrow`) and
  under-accents (`\underline`, `\underbrace`).
- Growing the math font subset; true top-accent attachment from the
  `MATH` table.

## Files owned

- `src/core/tex.rs`
- `src/shell/render.rs`, `src/shell/glyphs.rs` (tests only, if needed)
- `tests/fuzz.rs`, `tests/perf.rs`
- `README.md` — section *Formulas in text blocks* only
- feature note `docs/features/TeX formulas.md`, tutorial
  `docs/tutorials/39 Accents that sit on their letters.md`, new
  `ADR-T39-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T39): specify accent acceptance criteria`
- [ ] tests — `test(T39): add failing tests for accents`
- [ ] models — `feat(T39): add accent commands and node`
- [ ] behaviour — `feat(T39): place accents over their argument`
- [ ] quality — `chore(T39): pass clippy, rustfmt and rustdoc`
- [ ] docs — `docs(T39): update feature note and add tutorial`

## Learning path

Step 39 — requires step 33 (math font) and step 36 (operators whose
limits sit beside them).

## Log

- 2026-10-21 — Created from the owner's request to give accents their own
  task, after they were left out of T36's physics symbols. Checked before
  writing: the subset has no combining accents, but the spacing glyphs
  listed above are in the font and the metrics table. Runs in wave 25,
  after T36, because both own `src/core/tex.rs`.
