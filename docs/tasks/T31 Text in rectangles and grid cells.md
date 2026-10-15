---
id: T31
title: Text in rectangles and grid cells
status: done
wave: 19
branch: task/T31-shape-text
depends_on: [T29]
adrs: ["[[ADR-T31-1 Shape text prototype behind a switch]]"]
feature: "[[Text in shapes (experimental)]]"
tutorial: "[[31 Prototyping an interaction behind a switch]]"
tags: [task, experimental]
---

# T31 Text in rectangles and grid cells

## Goal

An **experimental** prototype for typing text inside rectangles,
ellipses and grid cells — e.g. filling a DP table cell by cell, or
naming a node — to find out whether it can be fast to use without
getting in the way. It is off by default and turned on with
`DRAW_EXPERIMENTAL=shape-text`. The result is a decision, recorded in an
ADR: promote it (follow-up task to make it default), rework it, or
remove it (rollback ADR + `git revert`, per [[Decision Protocol]]).

## Open questions (owner decides in the spec step)

1. **How to start typing.** Double-click a shape or cell with any tool?
   `Enter` on a selected shape? Text tool click inside a shape (T29's
   tool, so no new key)? Typing while hovering is rejected up front:
   it would turn every shortcut into a misfire.
2. **Moving between cells.** `Tab` / `Shift+Tab` and arrows move to the
   next cell while editing a grid, spreadsheet-style? Arrows already
   resize grids (T18) — only outside edit mode, but is that confusing?
3. **Numbers vs text.** A numbered shape (T19) has a label. Does text
   replace the number, sit beside it, or is a typed number just text
   (and numbering only the default content)? Same for grid indices
   (T22).
4. **Fit.** Shrink the font to fit the shape (like labels today), keep
   the block size and clip, or grow the shape?
5. **Attachment.** Text moves, copies and is erased with its shape;
   does the eraser on text clear only the text (like the fill eraser of
   T21)?
6. **Formulas.** If [[T30 Basic TeX formulas]] is done, `$…$` works in
   cells for free — keep or disable?

## Spec

Owner's answers (2026-10-15), recorded in
[[ADR-T31-1 Shape text prototype behind a switch]]:

1. **Start typing** — Text tool press inside a rectangle, ellipse or
   grid cell edits that shape's (cell's) text; with Select, a click on
   the already-only-selected shape edits it (the T29 re-edit rule).
2. **Cells** — `Tab` / `Shift+Tab` next / previous cell in reading
   order, wrapping; `Enter` / `Shift+Enter` cell below / above. Arrows
   move the caret only.
3. **Numbers** — text is separate from the label and drawn instead of
   it; blank text shows the number again. Counter and grid indices
   unchanged.
4. **Fit** — shrink to fit the shape's text frame, capped at the size
   the text was typed at; shapes never grow, nothing is clipped.
5. **Attachment** — text is part of its shape; the fill eraser clears
   fill and text together in one step.
6. **Formulas** — kept.

ADRs: **ADR-T31-1 — Shape text prototype** (accepted) and, after the
trial, **ADR-T31-2 — Verdict** (promote / rework / remove).

- **AC-1 — Off by default.** Without the variable nothing changes: same
  keys, same shapes, same rendering; the switch is a pure function over
  the environment. Documents never contain shape text unless the switch
  is on.
  Tests: `experimental::tests::from_env_value_unset_or_empty_is_all_off`,
  `from_env_value_shape_text_turns_it_on`,
  `from_env_value_is_a_case_insensitive_list_ignoring_unknown_names`;
  `editor::tests::shape_text_off_text_tool_in_rect_starts_free_block`,
  `shape_text_off_select_click_inside_selected_rect_clears_selection`;
  `fuzz::switch_off_sessions_never_create_shape_text`.
- **AC-2 — Type into shapes and cells.** With the switch on, a Text tool
  press inside a rectangle, ellipse or grid cell, or a Select click on
  the only selected one, opens edit mode on its text, reusing T29's edit
  mode (same keys, one undo entry per finished edit, `Ctrl+Z` cancels).
  Tests: `shape::tests::with_shape_text_sets_and_clears_rect_and_ellipse_text`,
  `with_cell_text_keeps_cells_sorted_unique_and_in_range`,
  `text_frame_is_scaled_box_inscribed_rect_or_cell`,
  `shape_texts_are_centred_and_fitted_in_their_frames`;
  `text_layout::tests::fit_shrinks_text_to_frame_and_caps_at_size`,
  `fit_centres_the_layout_in_the_frame`, `fit_of_blank_text_is_none`;
  `tools::text::tests::hosted_edit_refits_as_text_is_typed`;
  `editor::tests::shape_text_text_tool_click_in_rect_edits_its_text`,
  `shape_text_text_tool_click_in_ellipse_edits_its_text`,
  `shape_text_text_tool_click_in_grid_edits_that_cell`,
  `shape_text_text_block_on_top_of_rect_is_reopened`,
  `shape_text_select_second_click_edits_selected_shape`,
  `shape_text_edit_is_one_history_entry`,
  `shape_text_ctrl_z_cancels_edit`,
  `shape_text_blank_edit_clears_text`,
  `shape_text_overlay_hides_host_and_shows_edit_as_block`;
  `select::tests::shape_text_press_inside_only_selected_rect_keeps_it`.
- **AC-3 — Grid flow.** `Tab` / `Enter` fill a 5 × 5 grid without the
  mouse; undo restores cells one edit at a time.
  Tests: `shape::tests::grid_cell_step_follows_reading_order_on_screen`;
  `editor::tests::shape_text_tab_fills_5x5_grid_without_mouse`,
  `shape_text_shift_tab_goes_back_and_tab_wraps`,
  `shape_text_enter_moves_down_and_stays_at_bottom_row`,
  `shape_text_undo_restores_cells_one_edit_at_a_time`.
- **AC-4 — Shape behaviour.** Move, copy, duplicate and erase carry the
  text; the fill eraser clears fill and text in one step; text hides the
  label without touching the counter; the fuzz harness covers shape text
  with the switch on.
  Tests: `shape::tests::translate_keeps_shape_and_cell_text`;
  `tools::eraser::tests::clear_drag_clears_shape_and_cell_text`,
  `press_inside_text_only_shape_starts_clear_drag`;
  `editor::tests::shape_text_moves_copies_and_duplicates_with_shape`,
  `shape_text_erased_with_its_shape`,
  `shape_text_hides_label_and_keeps_counter`;
  `render::tests::label_is_hidden_by_shape_text`;
  `fuzz::switch_on_sessions_reach_shape_text` and the invariants (shape
  text non-blank with a finite positive size; cell texts sorted, unique,
  inside the grid) checked in every session.
- **AC-5 — Trial.** Owner on the target: fill a 5 × 5 DP table and label
  a 6-node graph, with and without the feature (T29 free text only);
  note times and every misfire or annoyance in the *Log*. (Manual.)
- **AC-6 — Verdict.** ADR-T31-2 records the decision from AC-5. At
  integration the integrator adds it under `[Unreleased]` → `Added` in
  `CHANGELOG.md`, marked `(experimental)`; if it is later removed, the
  rollback adds a `Removed` line.

## Out of scope

- Making it default (a follow-up task if promoted).
- Text in lines, arrows and strokes.
- Spreadsheet features (formulas over cells, copy a column, etc.).

## Files owned

- `src/core/shape.rs`, `src/core/editor.rs`, `src/core/tools/text.rs`,
  `src/core/tools/grid.rs`, `src/core/tools/select.rs`,
  `src/core/tools/eraser.rs`, `src/core/numbering.rs`,
  `src/core/text_layout.rs`, `src/core/experimental.rs` (new)
- `src/shell/render.rs`, `src/shell/app.rs` (reading the switch, drawing
  the open edit in the background colour on fills)
- `tests/fuzz.rs`, `README.md` (*Experimental* section)
- amended 2026-10-15 (owner): `src/core/tools/shape_tool.rs`,
  `src/core/clipboard.rs`, `src/core/snap.rs`, `tests/perf.rs` — only
  the new fields in `Rect` / `Ellipse` / `Grid` struct literals, no
  logic; `src/core/tools/bucket.rs` (one test literal) and
  `src/core/mod.rs` (`pub mod experimental;`) fall under the same
  amendment
- new ADRs `ADR-T31-*`, feature note
  `docs/features/Text in shapes (experimental).md`, tutorial
  `docs/tutorials/31 Prototyping an interaction behind a switch.md`

## Subtasks (one commit each)

- [x] spec — `docs(T31): …`
- [x] tests — `test(T31): …`
- [x] models — `feat(T31): …`
- [x] behaviour — `feat(T31): …`
- [x] quality — `chore(T31): …`
- [x] docs — `docs(T31): …`

## Learning path

Step 31 — requires step 29 (text blocks), step 18 (grid tool) and
step 19 (numbering).

## Log

- 2026-10-14 — Created as experimental at the owner's request; the
  interaction is still open (questions above). Depends only on T29 but
  runs after T30, which owns the same text layout and render files.
- 2026-10-15 — Spec: owner answered Q1–Q6 (all as recommended), see
  ADR-T31-1. Files owned amended with four files that construct
  `Rect` / `Ellipse` / `Grid` literals (new fields only).
- 2026-10-15 — Implemented on `task/T31-shape-text`. Every AC-1..AC-4
  test passes; `scripts/check.sh` green; fuzz at 2000 cases green.
  Additions beyond the spec's test list: `fit_source_places_empty_text_at_full_size`,
  `fit_always_inside_frame` (proptest), `hosted_edit_opens_existing_text_with_caret_at_end`,
  `hosted_edit_blank_text_is_none_and_wrong_slots_do_not_open`,
  `shape_text_new_rejects_blank_text_and_bad_sizes`,
  `text_slot_at_finds_interior_or_cell`, `with_experimental_sets_the_switch_in_helpers`,
  `shape_text_enter_in_rect_is_a_new_line`,
  `shape_text_press_inside_unselected_rect_does_not_pick_it`,
  fuzz `shape_text_sessions_never_panic_and_keep_invariants`,
  `shape_text_undo_all_then_redo_all_is_symmetric`. An edit on a
  filled shape or cell is drawn in the background colour (like its
  committed text), so it stays readable while typing.
- **Pending (owner):** AC-5 trial on the target — fill a 5 × 5 DP table
  and label a 6-node graph with and without `DRAW_EXPERIMENTAL=shape-text`,
  note times and misfires here — then AC-6, ADR-T31-2 *Verdict*.
- 2026-10-15 — Integrated into `main` as an experimental feature, off
  by default. Status → done. AC-5 (owner trial) and AC-6 (ADR-T31-2
  verdict) remain open; the verdict will promote, rework or remove it.
