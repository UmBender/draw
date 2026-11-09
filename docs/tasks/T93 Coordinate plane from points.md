---
id: T93
title: Coordinate plane from points
status: review
wave: 54
branch: task/T93-coordinate-plane
depends_on: [T87]
adrs: [ADR-T93-1]
feature: "[[Shapes]]"
tutorial: ""
tags: [task]
---

# T93 Coordinate plane from points

## Goal

A selected text block of points becomes axes with ticks and plotted points. (idea C4 of [[Feature proposal]])

## Known facts (2026-11-09)

- It uses the same select-a-block-and-press-a-key pattern as T84 and T87.
- `Ctrl+P` is unbound.

## Spec

Rules: [[ADR-T93-1 Coordinate plane key and plotting rules]]. Tests live in `src/core/tools/layout.rs`, `src/core/editor.rs` (`plot_from_text_*`) and `src/core/keymap.rs`.

- **AC-1** — With one text block of `x y` lines selected, `Ctrl+P` replaces it with two axes, numbered ticks and one small filled circle per point, one undo step.
  Tests: `ctrl_p_builds_a_plot_from_text`, `plot_draws_axes_ticks_numbers_and_one_dot_per_point`, `plot_points_are_small_filled_circles`, `plot_points_sit_where_their_coordinates_say`, `plot_y_grows_upward`, `plot_puts_its_top_left_at_the_origin`, `plot_stacks_axes_ticks_numbers_points`, `plot_refuses_a_size_that_is_not_positive_and_finite`, `plot_from_text_replaces_the_block`, `plot_from_text_keeps_the_block_place_and_colour`, `plot_from_text_is_one_undo_step`, `plot_from_text_selects_the_result`, `plot_from_text_cancels_a_gesture`, `plot_from_text_in_open_edit_only_types`, `plot_from_text_shapes_are_ordinary`, `plot_from_text_survives_a_snapshot`, `plot_from_text_without_one_text_block_does_nothing`.
- **AC-2** — Tick steps are 1, 2 or 5 × 10ⁿ, with both axes on the same scale.
  Tests: `step_is_one_two_or_five_times_a_power_of_ten` (proptest), `step_rounds_up_to_the_next_nice_value`, `frame_uses_one_step_for_both_axes`, `plot_axes_share_one_scale`, `ticks_are_numbered_with_their_values`, `tick_labels_are_exact_decimals`, `tick_labels_use_an_exponent_when_long`, `long_numbers_widen_the_ticks`.
- **AC-3** — Axes are always drawn through the origin when it is in range, and at the edge otherwise.
  Tests: `axes_cross_at_the_origin_when_it_is_in_range`, `axes_sit_on_the_edge_nearest_the_origin`, `origin_is_numbered_once`.
- **AC-4** — Integers and decimals are accepted, with negatives; non-finite values or text that does not parse leave everything unchanged, with the reason in the diagnostics log.
  Tests: `parse_reads_integers_decimals_and_negatives`, `parse_blank_text_is_no_plot`, `parse_refuses_what_is_not_a_point`, `parse_refuses_too_much_plot_text`, `plot_error_names_the_line_not_its_text`, `plot_from_text_bad_text_changes_nothing_and_leaves_a_notice`. The shell writes `Editor::take_notice` with `diag::log` after every event (T84).
- **AC-5** — More than 1 000 points: nothing changes. A single point or all-equal coordinates still gives a valid plot.
  Tests: `parse_accepts_1000_points`, `parse_refuses_more_than_1000_points`, `plot_from_text_too_many_points_changes_nothing_and_leaves_a_notice`, `single_point_gives_a_valid_plot`, `equal_coordinates_give_a_valid_plot`, `one_flat_axis_gives_a_valid_plot`, `extreme_coordinates_give_a_valid_plot`, `plot_holds_for_any_points` (proptest), `parse_and_plot_hold_for_any_text` (proptest).

The fuzzer reaches `Ctrl+P` through the bound chords and through a session built for it (`arb_plot_session`, `plot_sessions_turn_text_blocks_into_plots` in `tests/fuzz.rs`).

## Out of scope

- Plotting functions.
- Polygons joined in order.

## Files owned

- `src/core/tools/layout.rs`, `src/core/tools/mod.rs`, `src/core/shape.rs`
- `docs/features/Shapes.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/architecture/Architecture.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T93-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T93): specify coordinate plane from points`
- [x] tests — `test(T93): add failing tests`
- [x] models — `feat(T93): add types`
- [x] behaviour — `feat(T93): implement`
- [x] quality — `chore(T93): pass clippy and rustfmt`
- [x] docs — `docs(T93): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — implemented on `task/T93-coordinate-plane`, branched from `task/T87-graph-from-edges`.
- 2026-11-09 — UNATTENDED CHOICE: a coordinate above `1e300` in size is refused like a non-finite one, because the span of `±1e308` overflows. Exponents such as `1e3` are accepted as numbers ([[ADR-T93-1 Coordinate plane key and plotting rules]]).
- 2026-11-09 — UNATTENDED CHOICE: when the origin is out of range an axis goes on the edge nearest the origin, not always the bottom or left, so the edge shows which way the origin lies. To change it, replace the `clamp` in `PointList::shapes`.
- 2026-11-09 — UNATTENDED CHOICE: when both axes pass through the origin it is numbered once, on the horizontal axis, to avoid two zeros side by side.
- 2026-11-09 — UNATTENDED CHOICE: the top-left of everything drawn, numbers included, is the block's top-left, and the result is selected, as for [[T87 Graph or tree from an edge list]].
- 2026-11-09 — UNATTENDED CHOICE: `docs/architecture/Architecture.md` was added to *Files owned* for the `core::tools::layout` row. `p_l_m_are_unbound` now excepts `Ctrl+P`, the one chord this task binds. `Editor::graph_from_selection` and the new plot key now share `block_to_shapes` instead of repeating it; the graph tests pass unchanged.
- 2026-11-09 — manual check left for the owner: plot a sample from a geometry problem with `Ctrl+P` and judge the look. Geometry is tested; nothing was viewed in the running app, and writing the refusal to `draw.log` was not exercised there.
