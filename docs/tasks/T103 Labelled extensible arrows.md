---
id: T103
title: Labelled extensible arrows
status: review
wave: 58
branch: task/T103-labelled-arrows
depends_on: [T99]
adrs: ["[[ADR-T103-1 Extensible arrows from the arrow glyph and a ruled shaft]]"]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T103 Labelled extensible arrows

## Goal

`\xrightarrow{f}` and its relatives draw an arrow that stretches to fit a label above it, and an optional `[below]` label under it.

## Known facts (2026-11-10)

- Arrows are fixed-size glyphs; `\xrightarrow` prints literally ([[TeX formulas#Limits]]).
- Fractions already draw rules; scripts already set text in script size.
- Branches from T99 only because both edit `src/core/tex.rs`.

## Spec

Design: [[ADR-T103-1 Extensible arrows from the arrow glyph and a ruled shaft]]. Tests are in `src/core/tex.rs` (`mod tests`) unless named otherwise.

- **AC-1** (owner) — `\xrightarrow`, `\xleftarrow`, `\xleftrightarrow`, `\xRightarrow`, `\xLeftarrow`, `\xLeftrightarrow`, `\xmapsto` and `\xhookrightarrow` take a required `{above}` and an optional `[below]` argument: `\xrightarrow[n\to\infty]{f}`.
  Tests: `tex_parse_xarrows_take_a_label_and_an_optional_below_label`, `tex_parse_xarrow_below_label_ends_at_the_first_bracket_outside_braces`, `tex_parse_xarrows_print_back`.
- **AC-2** — Labels are set in script style, centred above / below the arrow; the arrow is at least as long as the fixed arrow and grows to the wider label plus padding (0.5 em) on both sides.
  Tests: `tex_layout_xarrow_labels_are_script_size_and_centred`, `tex_layout_xarrow_grows_to_the_wider_label_plus_padding`, `tex_layout_xarrow_is_never_shorter_than_its_arrow`, `tex_layout_stretched_xarrow_draws_its_heads_and_a_shaft`, `tex_layout_stretched_double_xarrow_has_two_shaft_rules`, `tex_layout_stretched_xmapsto_and_xhookrightarrow_draw_their_tails`.
- **AC-3** — The arrow is a relation (`Rel` spacing) sitting on the math axis like `\rightarrow`.
  Tests: `tex_layout_xarrow_is_spaced_as_a_relation`, `tex_layout_xarrow_shaft_sits_on_the_axis`.
- **AC-4** — Empty labels `\xrightarrow{}` give a plain arrow of minimum length; a missing `{…}` is invalid input shown in red, as other malformed commands are.
  Tests: `tex_layout_xarrow_with_empty_labels_is_the_plain_arrow`, `tex_parse_xarrow_without_label_is_literal`, `tex_layout_xarrow_without_label_is_drawn_in_the_warning_colour`.
- **AC-5** — Fuzzed; boxes stay finite, nested labels (`\xrightarrow{\xrightarrow{a}}`) work.
  Tests: `tex_layout_nested_xarrows_stay_finite_and_grow`, the proptests of `tex.rs` (`arb_source` gains the arrows and `[`, `]`), and `fuzz_tex_parse_and_layout_are_total` in `tests/fuzz.rs` (`arb_formula_text` gains the arrows).

## Out of scope

- `\overset`, `\underset`, `\stackrel`, `\overrightarrow`.
- Commutative diagrams.

## Files owned

- `src/core/tex.rs`, `src/core/text_layout.rs`, `src/shell/render.rs` (only if a rule needs drawing support)
- `docs/features/TeX formulas.md`, `README.md` (formula table), `docs/features/Contest cheat sheet.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T103-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T103): specify labelled extensible arrows`
- [x] tests — `test(T103): add failing tests for labelled extensible arrows`
- [x] models — `feat(T103): add the extensible arrow node and table`
- [x] behaviour — `feat(T103): lay out labelled extensible arrows`
- [x] quality — `chore(T103): pass clippy and rustfmt`
- [x] docs — `docs(T103): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
- 2026-11-10 — spec: ADR-T103-1. Branched from `task/T99-spacing-commands` (in review), as the batch plan says.
- 2026-11-10 — UNATTENDED CHOICE: the required label may be a single token (`\xrightarrow f`), as for every other command and in amsmath; only a missing label is invalid.
- 2026-11-10 — UNATTENDED CHOICE: labels sit `XARROW_GAP` = 0.111 em (TeX's `\bigopspacing1`) from the arrow's ink.
- 2026-11-10 — UNATTENDED CHOICE: the stretched `\xhookrightarrow` hook is a half circle of rules, since a glyph tail would show its own head mid-shaft (ADR-T103-1).
- 2026-11-10 — tests failed to compile for the right reason (no `Node::XArrow`, `Bracket`, `XARROW_*`); all green after the behaviour step. `src/shell/render.rs` was not needed: rules already draw.
- 2026-11-10 — quality: clippy asked for no `Option<Option<_>>` in the bracket parser (now a look-ahead) and longer test binding names.
- 2026-11-10 — FINDING (not T103): the first `scripts/check.sh --fuzz` run failed `snapshot_mutated_text_is_total` with "numbered shape holds text" — a graph from `Paste("1-2\n1-2 2-3\n$a$ $b$\n1-2\n1-2")`, `Ctrl+L`, a snapshot mutated with token `0`, then `Ctrl+Shift+Z`. Snapshot and editor code, outside this task's files; random seed `cc b900c994181812be68ac3e9325f0843a37d0a578f56661327f33479782fd6906`. The seed line was left out of `tests/fuzz.proptest-regressions` (not owned); the rerun was green. Worth a `fix/` task.
- 2026-11-10 — `scripts/check.sh --fuzz` green; `scripts/e2e.sh` green, no golden changed. `Contest cheat sheet.md` has no formula table, so it is unchanged.
- 2026-11-10 — manual check left for the owner: type `$A \xrightarrow[n\to\infty]{f_n} B \xleftarrow{g} C$` and judge, and look at a stretched `\xhookrightarrow` and `\xRightarrow` (shaft rules meeting the head glyphs).
