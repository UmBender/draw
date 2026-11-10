---
id: T95
title: Labels on plotted points
status: in-progress
wave: 54
branch: task/T95-point-labels
depends_on: [T93]
adrs: [ADR-T95-1]
feature: "[[Shapes]]"
tutorial: ""
tags: [task]
---

# T95 Labels on plotted points

## Goal

A line of a `Ctrl+P` point list can name its point, so `3 4 A` plots the point and writes `A` beside it.

## Known facts (2026-11-09)

- [[T93 Coordinate plane from points]] refuses any line that is not exactly two numbers.
- A plotted point's text is an ordinary text block, so it can hold `$…$` markup like any other.

## Spec

Rules: [[ADR-T95-1 Plotted point labels]]. Tests live in `src/core/tools/layout.rs` and `src/core/editor.rs`.

- **AC-1** — A line `x y label` plots the point and keeps `label`, the rest of the line without its outer whitespace, which may contain spaces. Lines without a label keep working.
  Tests: `parse_reads_an_optional_label`, `parse_keeps_the_spaces_inside_a_label`, `parse_refuses_what_is_not_a_point`.
- **AC-2** — Each label is a text block at the block's text size, up and to the right of its point and clear of the dot. Labels are drawn last.
  Tests: `plot_draws_one_text_per_label`, `plot_puts_a_label_up_and_right_of_its_point`, `plot_draws_labels_last`.
- **AC-3** — Labels never change the axes, ticks or dots of a plot. The top-left of everything drawn, labels included, is still the block's top-left.
  Tests: `plot_labels_leave_the_rest_of_the_plot_alone`, `plot_top_left_counts_its_labels`.
- **AC-4** — A label may be a formula.
  Test: `plot_label_may_be_a_formula`.
- **AC-5** — `Ctrl+P` on a labelled list is one undo step, and bad text still changes nothing.
  Tests: `plot_from_text_keeps_the_labels_of_the_points`, `plot_from_text_undoes_a_labelled_plot_in_one_step`.

The fuzzer's point lists include labels (`arb_point_text` in `tests/fuzz.rs`).

## Out of scope

- Moving a label away from its point automatically when two labels overlap.
- Labels on `Ctrl+L` graph nodes.

## Files owned

- `src/core/tools/layout.rs`, `src/core/editor.rs`
- `docs/features/Shapes.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T95-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T95): specify labels on plotted points`
- [ ] tests — `test(T95): add failing tests`
- [ ] models — `feat(T95): add types`
- [ ] behaviour — `feat(T95): implement`
- [ ] quality — `chore(T95): pass clippy and rustfmt`
- [ ] docs — `docs(T95): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's request: labels go on the coordinate plane's points, as a third column.
