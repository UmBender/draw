---
id: T39
title: Accents in formulas
status: todo
wave: 25
branch: task/T39-accents
depends_on: [T36]
adrs: []
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

To be written in the spec step: acceptance criteria and the tests that
prove them. Expected:

- **AC-1 — Parse.** Every accent command parses to a one-argument node
  and prints back losslessly; `\vecx` and a missing argument stay
  literals.
- **AC-2 — Placement.** The accent is above the argument's ink and
  horizontally centred on it (within a tolerance; skewed for one italic
  letter); `\overline` is as wide as its argument.
- **AC-3 — Box.** The accented box contains all ink (existing
  proptests), its width equals the argument's, and scripts attach after
  it.
- **AC-4 — Glyphs.** Every glyph an accent draws is in the math font and
  in `core::math_metrics` (existing shell tests extended).
- **AC-5 — Budgets and fuzz.** T30/T33 perf thresholds kept; the fuzz
  text sessions also type accents.
- **AC-6 — Docs.** README *Formulas in text blocks* and the feature note
  list the accent commands.

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
