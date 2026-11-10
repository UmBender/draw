---
id: T105
title: Display mode
status: in-progress
wave: 59
branch: task/T105-display-mode
depends_on: []
adrs: ["[[ADR-T105-1 Display style as a style field and display segments]]"]
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

Owner decisions are marked (owner); the rest is settled in [[ADR-T105-1 Display style as a style field and display segments]]. Tests are in `src/core/tex.rs` (`mod tests`) unless named otherwise.

- **AC-1** (owner) — `$$…$$` is a display formula (`Segment::Display`). `\$` stays a literal dollar; an unclosed `$$` stays text; `$…$` is unchanged.
  Tests: `tex_parse_double_dollar_is_a_display_formula`, `tex_parse_unclosed_double_dollar_reads_as_before`, `tex_parse_display_formula_does_not_cross_a_line_break`, `tex_parse_escaped_dollar_inside_display_stays_literal`, `tex_parse_unclosed_or_empty_dollar_is_text` (unchanged).
- **AC-2** (owner) — In display style, big operators and integrals are drawn at `DISPLAY_OP_SCALE` (1.8, ADR-T105-1), with `\sum`-like limits above and below and integral limits beside, as now.
  Tests: `tex_layout_display_big_operators_use_the_display_scale`, `tex_layout_display_sum_limits_are_centred_above_and_below`, `tex_layout_display_integral_scripts_sit_beside_the_sign`, `tex_layout_display_scripts_are_text_style`.
- **AC-3** (owner) — In display style, the numerator and denominator of `\frac` (and the parts of `\binom`, `\choose`, …) are laid out in text style, at full size; a fraction inside them shrinks as an inline one does now.
  Tests: `tex_layout_display_fraction_parts_are_full_size`, `tex_layout_display_nested_fraction_shrinks`, `tex_layout_display_binomial_and_infix_parts_are_full_size`.
- **AC-4** (owner) — `\dfrac` / `\dbinom` force display parts and `\tfrac` / `\tbinom` force inline parts in any formula; `\displaystyle` and `\textstyle` switch the style for the rest of their group.
  Tests: `tex_parse_dfrac_and_tfrac_take_two_arguments`, `tex_layout_dfrac_and_dbinom_have_full_size_parts_inline`, `tex_layout_tfrac_and_tbinom_have_script_parts_in_display`, `tex_parse_style_switches_are_nodes`, `tex_layout_displaystyle_switches_the_rest_of_its_group`, `tex_layout_textstyle_switches_back_in_display`, `tex_layout_style_switch_is_not_an_atom`.
- **AC-5** — A display formula sits on its own line of the block, centred on the block's width; the text before and after it moves to the lines above and below.
  Tests: `tex_layout_display_formula_sits_on_its_own_line`, `tex_layout_display_formula_is_centred_on_the_block_width`, `tex_layout_line_break_around_display_adds_no_empty_line`, `tex_layout_wide_display_formula_sets_the_block_width`.
- **AC-6** — Inline formulas are unchanged (every existing layout test passes as is, except that inline `\dbinom` now forces display parts by AC-4: `tex_layout_choose_and_tbinom_dbinom_match_binom` becomes `tex_layout_choose_and_tbinom_match_binom`); sources round-trip; display formulas are fuzzed.
  Tests: `tex_parse_never_panics_and_prints_back_the_source`, `tex_layout_bounds_contain_every_ink`, `tex_layout_is_finite` (generator gains `$$` and the new commands), `tex_print_display_formula_round_trips`, and `fuzz_tex_parse_and_layout_are_total` in `tests/fuzz.rs` (generator gains a `$$…$$` arm and the new commands).

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
- 2026-11-10 — spec: ADR-T105-1. UNATTENDED CHOICE: display operator scale 1.8; `Segment::Display` and `Node::MathStyle(bool)` with a `display` field on the layout style; a display formula does not cross a line break and falls back to the old reading when unclosed; text after it keeps its spaces; it is centred on the widest line. UNATTENDED CHOICE: the existing test `tex_layout_choose_and_tbinom_dbinom_match_binom` loses its `\dbinom` check (AC-4 makes inline `\dbinom` display-style) and is renamed `tex_layout_choose_and_tbinom_match_binom`.
