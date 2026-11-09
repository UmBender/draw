---
id: T91
title: Resize handles
status: in-progress
wave: 51
branch: task/T91-resize-handles
depends_on: []
adrs: [ADR-T91-1, ADR-T91-2]
feature: "[[Editing tools]]"
tutorial: ""
tags: [task]
---

# T91 Resize handles

## Goal

A selection can be resized by dragging a corner or with the keyboard. (idea E4 of [[Feature proposal]])

## Known facts (2026-11-09)

- Moving is the only transform today.
- Selection moves were made linear in [[T25 Linear-time selection with large documents]], and the overlay budget is under 2 ms.

## Spec

Tests are in `src/core/tools/select.rs` unless named otherwise. Rules
for the minimum size, text and uneven scales are in
[[ADR-T91-1 Resize handles and scale rules]].

- **AC-1** — With a selection, four corner handles show at a constant screen size; dragging one scales about the opposite corner, one undo step.
  `resize_drag_scales_about_opposite_corner`,
  `resize_is_one_undo_step`,
  `resize_preview_shows_scaled_and_hides_originals`,
  `resize_cancel_keeps_document`,
  `resize_click_on_handle_changes_nothing`,
  `handle_centers_keep_their_screen_offset_at_any_zoom`,
  `handle_centers_none_without_anything_to_scale`;
  `editor::tests::selection_handles_show_only_when_idle_in_select`,
  `editor::tests::resize_gesture_survives_a_snapshot`;
  `render::tests::selection_handle_rect_has_constant_screen_size`,
  `render::tests::handle_centers_are_the_selection_rect_corners`.
- **AC-2** — `Alt+[` and `Alt+]` scale the selection by 10 % about its centre, one undo step each.
  `scale_up_grows_ten_percent_about_centre`,
  `scale_down_shrinks_ten_percent_about_centre`,
  `scale_key_is_one_undo_step`,
  `scale_key_without_selection_does_nothing`;
  `editor::tests::alt_brackets_scale_the_selection`;
  `keymap::tests::every_documented_binding`.
- **AC-3** — `Shift` while dragging keeps the proportions.
  `resize_shift_keeps_proportions`,
  `resize_shift_pressed_mid_drag_applies`.
- **AC-4** — Every shape kind scales: points of strokes, lines and arrows, boxes of rectangles, ellipses and grids, and the size of text. Stroke widths do not change.
  `resize_scales_a_mixed_selection`;
  `shape::tests::scaled_about_moves_stroke_points_and_keeps_widths`,
  `shape::tests::scaled_about_moves_line_and_arrow_ends`,
  `shape::tests::scaled_about_moves_boxes_and_keeps_cells`,
  `shape::tests::scaled_about_scales_text_block_origin_and_size`,
  `shape::tests::scaled_about_scales_shape_and_cell_text`,
  `shape::tests::scaled_about_uneven_text_follows_smaller_factor`,
  `shape::tests::geometry_bounds_leave_out_outline_and_axes`.
- **AC-5** — A drag cannot flip a shape, or make it smaller than 4 world units; non-finite scales are ignored.
  `resize_past_opposite_corner_stops_at_minimum`,
  `resize_minimum_holds_for_the_smallest_selected_shape`,
  `scale_down_stops_at_minimum`,
  `scale_up_overflow_changes_nothing`;
  `shape::tests::min_scale_limits_each_side_of_a_box`,
  `shape::tests::min_scale_limits_only_the_longest_side_of_a_line`,
  `shape::tests::min_scale_limits_the_size_of_text`,
  `shape::tests::min_scale_keeps_small_shapes_from_shrinking`,
  `shape::tests::scaled_about_rejects_bad_factors`,
  `shape::tests::scaled_about_rejects_overflow`;
  fuzz: `arb_resize_session` in `tests/fuzz.rs`.
- **AC-6** — Handles take priority over move only on the handle itself; `Alt`-drag duplicate still works.
  `press_on_handle_wins_over_shape_below`,
  `press_beside_handle_moves_the_shape`,
  `alt_press_on_handle_duplicates`,
  `shift_press_on_handle_resizes`.

## Out of scope

- Rotation.
- Edge-only handles.
- Changing a grid's cell count (T85).

## Files owned

- `src/core/tools/select.rs`, `src/core/shape.rs`, `src/core/document.rs`, `src/core/history.rs`
- `src/shell/render.rs`
- `docs/features/Editing tools.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `src/shell/app.rs` (draws the handles; only `draw_overlay` changes)
- `tests/fuzz.rs`, `tests/fuzz.proptest-regressions`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T91-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T91): specify resize handles`
- [ ] tests — `test(T91): add failing tests`
- [ ] models — `feat(T91): add types`
- [ ] behaviour — `feat(T91): implement`
- [ ] quality — `chore(T91): pass clippy and rustfmt`
- [ ] docs — `docs(T91): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — UNATTENDED CHOICE: added `src/shell/app.rs` to *Files owned*. `draw_selection` also outlines the open text block, so the handles need their own call in `draw_overlay`.
- 2026-11-09 — UNATTENDED CHOICE: the rules the spec leaves open (which shapes the 4-unit minimum binds, text under uneven scales, handles only in the Select tool, no snapping) are recorded in [[ADR-T91-1 Resize handles and scale rules]].
- 2026-11-09 — UNATTENDED CHOICE: the fuzzer showed a resize could push text past `snapshot::MAX_TEXT_SIZE`, which snapshots refuse. The size is now capped at it ([[ADR-T91-2 Scaled text stays within the largest text size]]). The bug only existed in this step's uncommitted code, so its test `scaled_about_caps_text_at_the_largest_size` went in with the behaviour commit, and `scaled_about_rejects_overflow` now tests the text's origin instead of its box.
- 2026-11-09 — UNATTENDED CHOICE: added `tests/fuzz.proptest-regressions` to *Files owned*: the shrunk failing session of that bug stays as a regression seed, as [[Quality Gates]] asks.
