---
id: T105
title: Display mode
status: todo
wave: 59
branch: task/T105-display-mode
depends_on: []
adrs: []
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T105 Display mode

## Goal

`$$…$$` typesets a formula in display style, as LaTeX does: big operators and integrals are drawn larger, and fractions and binomials keep their parts at full size. `\dfrac`, `\tfrac`, `\dbinom`, `\tbinom`, `\displaystyle` and `\textstyle` switch the style inside any formula, so formulas pasted from a problem statement look as they do there.

## Known facts (2026-11-10)

- `tex::parse` splits a block on single `$`; `$$` alone is text and `$$x$$` reads as a stray `$`, a formula and another `$` (test `tex_parse_unclosed_or_empty_dollar_is_text`).
- Big operators and integrals are drawn at `BIG_OP_SCALE` (1.3) in every formula; `\sum` already takes its limits above and below, integrals beside ([[TeX formulas]]).
- Fraction and binomial parts are laid out at `SCRIPT_SCALE` (0.7). `\dbinom` and `\tbinom` draw like `\binom`; `\dfrac` and `\tfrac` do not exist.
- [[TeX formulas#Limits]] lists "no display mode" and "big operators are always the inline size".
- The font subset has no display-size variants of the big operators, so a display operator is the same glyph scaled ([[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]); adding the variants would cost binary size.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `$$…$$` is a display formula. `\$` stays a literal dollar; an unclosed `$$` stays text; `$…$` is unchanged.
- **AC-2** (owner) — In display style, big operators and integrals are drawn at a larger scale than `BIG_OP_SCALE` (proposed about 1.8, chosen in the ADR), with `\sum`-like limits above and below and integral limits beside, as now.
- **AC-3** (owner) — In display style, the numerator and denominator of `\frac` (and the parts of `\binom`, `\choose`, …) are laid out in text style, at full size; a fraction inside them shrinks as an inline one does now.
- **AC-4** (owner) — `\dfrac` / `\dbinom` force display parts and `\tfrac` / `\tbinom` force inline parts in any formula; `\displaystyle` and `\textstyle` switch the style for the rest of their group.
- **AC-5** — A display formula sits on its own line of the block, centred on the block's width; the text before and after it moves to the lines above and below.
- **AC-6** — Inline formulas are unchanged (every existing layout test passes as is); sources round-trip; display formulas are fuzzed (`$$` with random contents in the TeX proptests and `tests/fuzz.rs`).

## Out of scope

- `\limits` / `\nolimits`, `\[ … \]`, `equation`, `align` and numbered equations.
- Display-size glyph variants from the full font.
- `\scriptstyle` and `\scriptscriptstyle`.

## Files owned

- `src/core/tex.rs`, `src/core/text_layout.rs`
- `docs/features/TeX formulas.md`, `README.md` (formula table), `docs/features/Contest cheat sheet.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T105-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T105): specify display mode`
- [ ] tests — `test(T105): add failing tests for display mode`
- [ ] models — `feat(T105): add display style to formula nodes`
- [ ] behaviour — `feat(T105): lay out display formulas`
- [ ] quality — `chore(T105): pass clippy and rustfmt`
- [ ] docs — `docs(T105): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from F3 of the [[Feature proposal]].
