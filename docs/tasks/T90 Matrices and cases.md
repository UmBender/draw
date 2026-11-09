---
id: T90
title: Matrices and cases
status: in-progress
wave: 52
branch: task/T90-matrices
depends_on: [T89]
adrs: [ADR-T90-1]
feature: "[[TeX formulas]]"
tutorial: ""
tags: [task]
---

# T90 Matrices and cases

## Goal

Formulas can contain matrices and piecewise definitions. (idea F2 of [[Feature proposal]])

## Known facts (2026-11-09)

- A formula is one line of source; the box layout already stacks fractions.

## Spec

Decisions: [[ADR-T90-1 Environments as one table node]]. All tests are in `src/core/tex.rs`.

- **AC-1** — `matrix`, `pmatrix`, `bmatrix`, `vmatrix` and `cases` environments parse, with `&` between cells and `\\` between rows.
  Tests: `tex_parse_environment_is_one_node_keeping_its_source`, `tex_parse_every_environment_splits_cells_and_rows`, `tex_parse_cells_keep_their_whitespace`, `tex_parse_separators_in_a_nested_part_belong_to_it`, `tex_parse_nested_environments_keep_their_own_cells`, `tex_parse_script_before_a_separator_stays_literal`, `tex_parse_environment_at_the_cell_limit_is_a_table`, `tex_layout_every_environment_draws_without_warning`, and the proptests over `arb_source`.
- **AC-2** — Matrix columns are centred; `cases` columns are left-aligned with one growing brace.
  Tests: `tex_layout_matrix_columns_are_centred`, `tex_layout_matrix_column_is_as_wide_as_its_widest_cell`, `tex_layout_matrix_rows_are_a_strut_apart`, `tex_layout_tall_row_pushes_the_next_one_down`, `tex_layout_cases_columns_are_left_aligned`, `tex_layout_cases_rows_are_stretched`, `tex_layout_cases_has_one_growing_brace`.
- **AC-3** — The block is vertically centred on the math axis, and brackets use T89's scaling.
  Tests: `tex_layout_environment_is_centred_on_the_axis`, `tex_layout_matrix_brackets_match_left_right`, `tex_layout_plain_matrix_draws_no_brackets`, `tex_layout_environment_is_an_ordinary_atom_for_scripts`.
- **AC-4** — An unknown environment or missing `\end` is drawn in red and never panics.
  Tests: `tex_parse_unknown_environment_is_literal`, `tex_parse_environment_without_end_is_literal`, `tex_parse_end_without_begin_is_literal`, `tex_parse_mismatched_environment_names_are_literal`, `tex_parse_begin_without_a_name_is_literal`, `tex_parse_environment_across_a_group_is_literal`, `tex_parse_environment_across_a_left_pair_is_literal`, `tex_parse_environment_past_the_cell_limit_is_literal`, `tex_parse_deep_environments_are_literal_past_the_limit`, `tex_layout_invalid_environment_is_drawn_in_the_warning_colour`, and `tests/fuzz.rs`.
- **AC-5** — Rows of different lengths leave empty cells.
  Tests: `tex_layout_short_row_leaves_empty_cells`, `tex_layout_trailing_row_break_adds_no_row`, `tex_layout_empty_environment_draws_only_its_brackets`.

## Out of scope

- Source spread over several text lines.
- `array` with a column spec.

## Files owned

- `src/core/tex.rs`
- `docs/features/TeX formulas.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T90-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T90): specify matrices and cases`
- [ ] tests — `test(T90): add failing tests`
- [ ] models — `feat(T90): add types`
- [ ] behaviour — `feat(T90): implement`
- [ ] quality — `chore(T90): pass clippy and rustfmt`
- [ ] docs — `docs(T90): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
