---
id: T89
title: Growing brackets
status: done
wave: 51
branch: task/T89-growing-brackets
depends_on: []
adrs: [ADR-T89-1]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T89 Growing brackets

## Goal

`\left` and `\right` draw brackets that grow to fit what they enclose. (idea F1 of [[Feature proposal]])

## Known facts (2026-11-09)

- `delimited` in `src/core/tex.rs` already scales a bracket glyph for `\binom`, up to `BINOM_MAX_SCALE`.
- The font subset has no size variants, so glyphs are scaled as `\binom` does.

## Spec

Decisions: [[ADR-T89-1 Left and right brackets centred on the axis]]. All tests are in `src/core/tex.rs`.

- **AC-1** — `\left( … \right)` scales both brackets to the enclosed height, for `( ) [ ] \{ \} | \| \langle \rangle \lfloor \rfloor \lceil \rceil`.
  Tests: `tex_parse_left_right_is_one_node_keeping_its_source`, `tex_layout_left_right_scales_brackets_to_the_content`, `tex_layout_left_right_grows_every_delimiter`, `tex_layout_left_right_centres_brackets_on_the_axis`, `tex_layout_left_right_stops_at_the_scale_limit`, and the proptests over `arb_source`.
- **AC-2** — `.` draws nothing on that side.
  Test: `tex_layout_left_right_null_delimiter_draws_nothing`.
- **AC-3** — An unmatched `\left` or `\right` is drawn in red, as other invalid input is, and never panics.
  Tests: `tex_parse_unmatched_left_or_right_is_literal`, `tex_parse_left_or_right_without_a_delimiter_is_literal`, `tex_parse_left_right_across_a_group_is_literal`, `tex_parse_left_after_text_is_not_a_body`, `tex_parse_deep_left_right_is_literal_past_the_limit`, `tex_layout_unmatched_left_right_is_drawn_in_the_warning_colour`, and `tests/fuzz.rs`.
- **AC-4** — Pairs nest, and scripts on the closing bracket attach to it.
  Tests: `tex_parse_left_right_pairs_nest`, `tex_layout_left_right_outer_pair_is_at_least_as_tall_as_inner`, `tex_layout_left_right_script_attaches_to_the_closing_bracket`.
- **AC-5** — Short content leaves the brackets at normal size.
  Test: `tex_layout_left_right_short_content_keeps_normal_size`.

## Out of scope

- `\middle`.
- Adding size variants to the font.

## Files owned

- `src/core/tex.rs`
- `docs/features/TeX formulas.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T89-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T89): specify growing brackets`
- [x] tests — `test(T89): add failing tests`
- [x] models — `feat(T89): add types`
- [x] behaviour — `feat(T89): implement`
- [x] quality — `chore(T89): pass clippy and rustfmt`
- [x] docs — `docs(T89): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — UNATTENDED CHOICE: raised the bracket scale cap for `\left`/`\right` to 12 (`LEFT_RIGHT_MAX_SCALE`), as the plan allows, because the 3 × of `\binom` is too short for tall content; `\binom` keeps 3 × ([[ADR-T89-1 Left and right brackets centred on the axis]]).
- 2026-11-09 — UNATTENDED CHOICE: a pair is one ordinary atom rather than TeX's Inner class, so there is no thin space beside the brackets. Delimiters are exactly the spec's list plus `.`; `\vert`, `\lvert` and the like show in red after `\left`.
- 2026-11-09 — the behaviour commit also corrects one test expectation from the test step: whitespace after a delimiter belongs to the body.
- 2026-11-09 — `scripts/check.sh --fuzz` green. Nothing was viewed in the running app. Manual check for the owner: `$\left( \frac{a}{b} \right)^2$`.
- 2026-11-09 — integrated. Left for the owner: check `$\left( \frac{a}{b} \right)^2$`.
