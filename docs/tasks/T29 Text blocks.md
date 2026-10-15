---
id: T29
title: Text blocks
status: in-progress
wave: 15
branch: task/T29-text
depends_on: [T14]
adrs: ["[[ADR-T29-1 Text shape with monospaced layout and an embedded font]]", "[[ADR-T29-2 Text edit mode]]", "[[ADR-T29-3 Resize text while typing]]"]
feature: "[[Text blocks]]"
tutorial: "[[29 Typing on a canvas without losing the shortcuts]]"
tags: [task]
---

# T29 Text blocks

## Goal

A **Text** tool (`T`) places free-standing text on the canvas: click,
type, leave. Text blocks are ordinary shapes — they move, copy, erase,
undo and snap like the others — and stay sharp at every zoom. Typing
must never fire a tool shortcut, and leaving text mode must be one key.
Text inside rectangles and grid cells is
[[T31 Text in rectangles and grid cells]]; formulas are
[[T30 Basic TeX formulas]].

## Design points (settle in the spec step)

- **Edit mode.** While a block is being edited every printable key is
  text; only `Esc` (commit and leave), `Enter` (new line;
  `Ctrl+Enter` commits?), `Backspace`, `Delete`, arrows, `Home`/`End`
  and `Ctrl+Z` act. Clicking outside commits. An empty block is
  discarded. Editing an existing block: Text tool click on it, or
  double-click with Select.
- **Undo.** Creating or editing a block is **one** history entry when
  the edit ends, not one per key; `Ctrl+Z` during an edit undoes typing
  inside the block first (or cancels the edit — pick one).
- **Model.** New `Shape::Text { origin, text, size, style }` in
  `core::shape` (ADR-0004 amended again, like ADR-T16-1). Size in world
  units, picked from the current width (`[` / `]`) so it zooms with the
  canvas.
- **Layout without macroquad.** `core` must stay free of macroquad, but
  bounds, hit-testing, caret placement and snapping need text metrics.
  Proposal: a **monospaced** embedded font, so layout is
  `chars × advance × size` and fully testable in `core`. ADR-T29-1.
- **Font.** macroquad's default font is ASCII-only. Embed one OFL
  monospaced font (`include_bytes!`, rendered through macroquad's own
  TTF loader, no new dependency) covering Latin, Greek and the math
  symbols T30 needs. Binary size grows by the font; ADR-T29-1 records
  the font, its license and the new size, and the current number labels
  may move to it for one consistent look.
- **Typed characters.** `core::input` gains a text event fed from
  miniquad's `char_event` (keyboard layout and dead keys handled by the
  system); `shell::input_map` collects it. IME composition is out of
  scope.

## Spec

How it works: [[ADR-T29-1 Text shape with monospaced layout and an embedded font]]
(model, layout, font) and [[ADR-T29-2 Text edit mode]] (input, commit,
undo, re-edit, caret). Unit tests live in `#[cfg(test)] mod tests` of the
module named; editor tests use `Editor::handle` with the events the shell
sends (`Char` then `KeyDown` for a printable key).

- **AC-1 — Place and type.** With the Text tool (`T`), a left press
  starts a block whose top-left corner is the press point (snapped like a
  shape start) with the caret at 0; `Char` events appear in
  `Editor::overlay` as typed; `Esc`, `Ctrl+Enter` or a press outside
  commits one `Shape::Text`; a blank block leaves no shape and no history
  entry. *Tests (`core::editor`):* `text_click_starts_block_at_pointer`,
  `text_typed_chars_appear_in_overlay`, `text_escape_commits_one_shape`,
  `text_ctrl_enter_commits`,
  `text_click_outside_commits_and_starts_new_block`,
  `text_empty_block_leaves_no_shape_and_no_history`,
  `text_whitespace_only_block_is_discarded`,
  `text_new_block_size_follows_width_and_zoom`,
  `text_new_block_uses_current_colour`.
- **AC-2 — Shortcuts are off while typing.** In edit mode every chord of
  `BINDINGS` runs no command (tool, style, helpers, view, toolbar,
  selection and document unchanged apart from the text); printable bare
  keys type their character (`P`, `1`, `[`, `Space` …), `Tab` types four
  spaces; `Ctrl` + a character inserts nothing; `Space` does not pan.
  After `Esc` the same keys act as shortcuts again.
  *Tests (`core::editor`):* `text_edit_mode_every_binding_types_instead_of_acting`,
  `text_after_escape_keys_act_as_shortcuts_again`,
  `text_ctrl_chars_are_not_inserted`, `text_tab_types_four_spaces`,
  `text_space_in_edit_mode_does_not_pan`,
  `text_char_outside_edit_mode_is_ignored`;
  *(`core::keymap`):* `t_selects_text_tool`, `editing_keys_are_unbound`;
  *(`shell::input_map`):* `map_key_editing_keys`,
  `collector_char_event_becomes_char`, `collector_skips_unmapped`.
- **AC-3 — Editing keys.** `←` `→` step one `char`, `↑` `↓` change line
  keeping the column (clamped), `Home`/`End` go to the line ends,
  `Backspace`/`Delete` remove one `char`, `Enter` inserts a line break —
  all on multi-byte text without panics; the caret stays on a `char`
  boundary for any sequence of edits. *Tests (`core::tools::text`):*
  `edit_insert_places_char_at_caret`,
  `edit_backspace_removes_multibyte_char`,
  `edit_delete_removes_char_after_caret`,
  `edit_left_right_step_over_multibyte_chars`,
  `edit_home_end_go_to_line_bounds`,
  `edit_up_down_keep_column_clamped`, `edit_enter_inserts_newline`,
  `edit_ends_of_text_are_noops`,
  `edit_random_operations_keep_caret_on_boundary` (proptest);
  *(`core::editor`):* `text_editing_keys_edit_the_block`.
- **AC-3b — Resize while typing.** `Ctrl+]` / `Ctrl+[` step the open
  block's size up / down the on-screen text ladder (14 … 76 px) at the
  current zoom, clamped at the ends; the size is committed with the edit
  and the stroke width is unchanged
  ([[ADR-T29-3 Resize text while typing]]). *Tests (`core::tools::text`):*
  `edit_step_size_walks_text_ladder`; *(`core::editor`):*
  `text_ctrl_brackets_resize_open_block`.
- **AC-4 — Re-edit.** A Text-tool press on a block re-opens it with the
  caret at the press; with Select a click on the block that is already the
  only selection re-opens it (a double-click does exactly this); a click
  on an unselected block only selects it; while re-editing the original is
  hidden and the edited copy drawn; a press inside the edited block moves
  the caret. *Tests (`core::editor`):*
  `text_tool_click_on_block_reopens_it_with_caret_at_click`,
  `select_click_on_selected_block_reopens_it`,
  `select_double_click_on_block_reopens_it`,
  `select_click_on_unselected_block_only_selects`,
  `text_reedit_hides_original_while_editing`,
  `text_click_inside_edited_block_moves_caret`.
- **AC-5 — Undo.** One history entry per finished edit (new block:
  insert; changed: replace; blanked: remove); undo/redo restore the text
  exactly; `Ctrl+Z` while editing cancels the edit; toolbar commands
  commit first. *Tests (`core::editor`):* `text_edit_is_one_history_entry`,
  `text_undo_redo_restore_text_exactly`,
  `text_ctrl_z_while_editing_cancels_edit`,
  `text_reedit_unchanged_adds_no_history`,
  `text_reedit_to_empty_removes_block_undoably`,
  `text_toolbar_command_commits_edit_first`,
  `text_commit_bumps_document_revision`.
- **AC-6 — A shape like the others.** Bounds from the layout, hit inside
  the box, open (no fill, bucket ignores it), translate, finite check;
  select, move, `Alt`-drag duplicate, copy/paste, eraser, fit-to-content;
  snapping to its box edges and centre and from its origin. The T13 fuzz
  harness generates `Char` events and Text-tool typing sessions; its
  invariants hold (text blocks finite, positive size, non-blank text).
  *Tests (`core::text_layout`):* `layout_bounds_empty_text_is_one_line_tall`,
  `layout_bounds_multiline_uses_longest_line`,
  `layout_bounds_counts_chars_not_bytes`,
  `layout_caret_point_after_newline_starts_next_line`,
  `layout_caret_from_point_rounds_to_nearest_column`,
  `layout_caret_from_point_clamps_outside_text`,
  `layout_glyphs_skip_newlines_and_advance_by_cells`,
  `layout_non_finite_size_counts_as_zero`,
  `layout_caret_from_point_is_char_boundary` (proptest),
  `layout_bounds_contain_every_glyph_cell` (proptest);
  *(`core::shape`):* `text_bounds_follow_layout`, `text_hit_inside_box`,
  `text_is_open_and_never_filled`, `text_translate_moves_origin`,
  `text_is_finite_checks_origin_and_size`, `text_has_no_label`;
  *(`core::snap`):* `targets_include_text_block_edges`;
  *(`core::editor`):* `text_click_snaps_origin_to_grid`,
  `text_block_moves_duplicates_and_erases_like_other_shapes`,
  `text_block_copy_paste_keeps_text`, `fit_view_includes_text_blocks`,
  `text_bucket_ignores_text_block`;
  *(`tests/fuzz.rs`):* `strategies_generate_all_variants`,
  `sessions_reach_text_edit_states`,
  `editor_never_panics_and_keeps_invariants`,
  `undo_all_then_redo_all_is_symmetric`.
- **AC-7 — Rendering.** Glyphs drawn per cell at their layout positions,
  on-screen size `size × zoom`, hidden below `TEXT_MIN_PX`, rasterized at
  the smallest `TEXT_RASTER_SIZES` entry at least that size (sharp at
  0.05×–20×); culled by bounds; committed blocks drawn in the document
  layer, the edited block and a non-blinking caret in the overlay; number
  labels use the same embedded font. *Tests (`shell::render`):*
  `text_screen_px_scales_with_zoom`, `text_screen_px_hidden_below_min`,
  `text_raster_picks_size_at_or_above`, `text_raster_caps_at_largest`;
  *(`shell::app`):* `embedded_font_is_truetype`;
  *(`core::editor`):* `text_caret_follows_typing`,
  `text_caret_none_when_not_editing`. Visual check of sharpness: manual.
- **AC-8 — Budgets.** 500 blocks of 20 characters: layout + culling of all
  blocks and one typed character (handle + overlay) each stay under 1 ms
  in release (`tests/perf.rs`: `perf_text_layout_and_cull_500_blocks_is_under_1ms`,
  `perf_typing_with_500_blocks_is_under_1ms`); redraw budgets
  ([[ADR-T24-5 Redraw budgets]], [[ADR-T26-2 Gesture frame budget]]) are
  checked by the owner on the target with `DRAW_FRAME_TIMES=1`. The
  release binary stays under 2 MB (measured, in the *Log*).
- **AC-9 — Docs.** README tool table, quick start and feature list;
  `docs/architecture/Keymap.md` gains `T` and the edit-mode keys. At
  integration the integrator adds the feature under `[Unreleased]` →
  `Added` in `CHANGELOG.md`.

## Out of scope

- Text inside rectangles and grid cells ([[T31 Text in rectangles and grid cells]]).
- Formulas ([[T30 Basic TeX formulas]]).
- Pasting system clipboard text, rich text, fonts or font choice, word
  wrap, alignment options, IME input.

## Files owned

- `src/core/shape.rs`, `src/core/input.rs`, `src/core/keymap.rs`,
  `src/core/editor.rs`, `src/core/command.rs`, `src/core/tools/text.rs`
  (new), `src/core/tools/mod.rs`, `src/core/text_layout.rs` (new),
  `src/core/snap.rs`, `src/core/mod.rs`
- `src/core/tools/shape_tool.rs` — only the two exhaustive `match`es
  that must name the new variants: `Kind::from_tool` (`Tool::Text` →
  `None`) and the `ends` test helper (`Shape::Text` panics) (added with
  the owner's approval during the spec step)
- `src/shell/input_map.rs`, `src/shell/render.rs`, `src/shell/toolbar.rs`,
  `src/shell/app.rs` (font loading)
- `assets/fonts/*` (new, with its license file)
- `tests/fuzz.rs`, `tests/perf.rs`, `README.md`
- `docs/architecture/Keymap.md`, `docs/architecture/Architecture.md`
- new ADRs `ADR-T29-*`, feature note `docs/features/Text blocks.md`,
  tutorial `docs/tutorials/29 Typing on a canvas without losing the shortcuts.md`

## Subtasks (one commit each)

- [x] spec — `docs(T29): specify text block acceptance criteria`
- [x] tests — `test(T29): add failing tests for text blocks`
- [x] models — `feat(T29): add text shape, edit session and input types`
- [x] behaviour — `feat(T29): implement text blocks and text edit mode`
- [x] quality — `refactor(T29): simplify line lookup and caret height`
- [x] docs — `docs(T29): add feature note and tutorial`

## Learning path

Step 29 — requires step 5/16 (shape model), step 8 (editor and input),
step 11 (keymap) and step 24 (layered redraw).

## Log

- 2026-10-14 — Created from the owner's proposal for text blocks and
  formulas.
- 2026-10-14 — Owner moved T29–T31 ahead of T27/T28 (waves 15–17).
- 2026-10-14 — Spec: ADR-T29-1 (model, monospaced layout, 124 KB
  Adwaita Mono subset) and ADR-T29-2 (edit mode). Files owned gain
  `shape_tool.rs` (test module, owner approved) and the two architecture
  notes.
- 2026-10-14 — Models: `Kind::from_tool` in `shape_tool.rs` (non-test
  code) is a second exhaustive `match` that must name `Tool::Text`; same
  file and cause as the approved change, so it was included and *Files
  owned* reworded. No other file outside the list was touched.
- 2026-10-14 — Behaviour: `layout_has_every_button_in_order` still
  expected 21 toolbar buttons; corrected to 22 in the behaviour commit
  (missed in the test step).
- 2026-10-14 — Quality: `scripts/check.sh` green; long fuzz
  (`PROPTEST_CASES=20000 cargo test --release --test fuzz`) green in 42 s.
  Perf (`cargo test --release --test perf -- --ignored`, this machine):
  text layout + cull of 500 blocks 95 µs, one keystroke with 500 blocks
  0.4 µs (budgets 1 ms); the T14 tests unchanged (delete 5k: 958 µs,
  close to its budget as before). Release binary 1 147 896 bytes
  (was 995 912; +152 KB, under 2 MB).
- 2026-10-14 — Open for the owner on the target: redraw budgets with
  `DRAW_FRAME_TIMES=1` ([[ADR-T24-5 Redraw budgets]],
  [[ADR-T26-2 Gesture frame budget]]) with ~500 text blocks, and a visual
  check of sharpness at 0.05×, 1× and 20× and of non-ASCII glyphs.
  `docs/features/Contest cheat sheet.md` does not list `T` yet (not in
  *Files owned*; for the integrator).
