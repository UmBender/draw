---
id: T84
title: Grid from text
status: done
wave: 52
branch: task/T84-grid-from-text
depends_on: [T83]
adrs: [ADR-T84-1]
feature: "[[Grid tool]]"
tutorial: ""
tags: [task]
---

# T84 Grid from text

## Goal

A selected text block becomes a grid in one key: lines are rows, whitespace separates cells. The same works straight from the clipboard, so a DP table or sample matrix appears at once. (idea C1 of [[Feature proposal]])

## Known facts (2026-11-09)

- `Shape::Grid` has `texts: Vec<CellText>`, sorted, and at most 64 columns and rows (`src/core/shape.rs`).
- New keys need an ADR ([[Keymap]]). `Ctrl+T` and `Ctrl+Shift+V` are unbound outside text edit.

## Spec

Rules and placement: [[ADR-T84-1 Grid from text keys and table rules]]. Tests live in `src/core/tools/grid.rs` (`table_*`), `src/core/editor.rs` (`grid_from_*`), `src/core/keymap.rs` and `src/shell/input_map.rs`.

- **AC-1** — With exactly one text block selected, `Ctrl+T` replaces it with a grid at the block's top-left, one undo step.
  Tests: `ctrl_t_builds_a_grid_from_text`, `grid_from_text_replaces_the_block_at_its_top_left`, `grid_from_text_is_one_undo_step`, `grid_from_text_keeps_colour_size_and_selection`, `grid_from_text_cancels_a_gesture`.
- **AC-2** — Lines are rows and runs of whitespace separate cells; blank lines are ignored; short rows leave empty cells.
  Tests: `table_parse_splits_lines_and_whitespace`, `table_parse_ignores_blank_lines`, `table_parse_short_rows_leave_empty_cells`, `table_parse_handles_line_endings_and_controls`, `table_shape_places_tokens_in_sorted_cells`, `table_shape_cells_follow_the_longest_token`, `table_shape_digits_get_square_cells`, `table_shape_text_fits_at_full_size`, `table_parse_and_shape_hold_the_invariants` (proptest).
- **AC-3** — One line gives a 1 × n strip. Axis indices follow the `N` toggle, as for any new grid.
  Tests: `table_parse_one_line_is_a_strip`, `grid_from_text_axes_follow_numbering`.
- **AC-4** — `Ctrl+Shift+V` builds the same grid from the clipboard at the cursor, using the event from [[T83 Paste text from the clipboard]].
  Tests: `grid_from_clipboard_puts_the_corner_at_the_cursor`, `grid_from_clipboard_has_the_cells_of_ctrl_t`, `grid_from_clipboard_is_one_undo_step_and_selected`, `grid_from_clipboard_in_open_edit_is_ignored`, `wants_grid_clipboard_only_for_ctrl_shift_v_outside_edit`, `paste_grid_event_needs_some_text`.
- **AC-5** — More than 64 columns or rows: nothing changes (no truncation of the input) and the reason goes to the diagnostics log.
  Tests: `table_parse_at_the_limits_is_accepted`, `table_parse_over_the_limits_is_refused`, `table_parse_over_the_byte_limit_is_refused`, `grid_from_text_too_wide_changes_nothing_and_leaves_a_notice`, `grid_from_clipboard_too_tall_changes_nothing_and_leaves_a_notice`, `take_notice_is_empty_after_a_success`. The shell writes `Editor::take_notice` with `diag::log`.
- **AC-6** — With no text block selected, or only blank text, the key does nothing and adds no undo step.
  Tests: `table_parse_blank_text_is_no_table`, `grid_from_text_without_one_text_block_does_nothing`, `grid_from_text_with_blank_text_does_nothing`, `grid_from_clipboard_blank_does_nothing`.

The fuzzer sends `InputEvent::PasteGrid`, and `Ctrl+T` through the bound chords (`tests/fuzz.rs`).

## Out of scope

- Formulas that contain spaces are split like any text.
- Editing a grid as text again.

## Files owned

- `src/core/tools/grid.rs`, `src/core/shape.rs`
- `src/shell/input_map.rs`, `src/shell/app.rs`
- `docs/features/Grid tool.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T84-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T84): specify grid from text`
- [x] tests — `test(T84): add failing tests`
- [x] models — `feat(T84): add types`
- [x] behaviour — `feat(T84): implement`
- [x] quality — `chore(T84): pass clippy and rustfmt`
- [x] docs — `docs(T84): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — implemented on `task/T84-grid-from-text`, branched from `task/T83-paste-text`. `scripts/check.sh --fuzz` is green.
- 2026-11-09 — UNATTENDED CHOICE: `Ctrl+Shift+V` uses a new `InputEvent::PasteGrid`, built the same way as T83's event, because [[ADR-T83-1 Paste text through an input event]] and a T83 test fix that `Paste` with no open edit does nothing ([[ADR-T84-1 Grid from text keys and table rules]]).
- 2026-11-09 — UNATTENDED CHOICE: `Ctrl+T` keeps the block's own text size, so the text looks the same after the conversion; only `Ctrl+Shift+V` uses the current size for new text. The plan's default says "the current text size" without separating the two.
- 2026-11-09 — UNATTENDED CHOICE: the pasted grid's top-left corner goes at the cursor, not its centre as for pasted shapes, so a large table starts where the pointer is.
- 2026-11-09 — UNATTENDED CHOICE: cells are never narrower than they are tall, so a matrix of digits gets square cells, and a text over 64 KiB is refused whole like one over 64 columns or rows.
- 2026-11-09 — UNATTENDED CHOICE: `t_selects_text_tool` asserted that `Ctrl+T` was unbound; that assertion moved to the new `ctrl_t_builds_a_grid_from_text`. `docs/architecture/Architecture.md` is not in *Files owned* and its module map was left as it is; the integrator may add `TextTable` to the `core::tools::grid` row.
- 2026-11-09 — manual check left for the owner: paste a sample matrix with `Ctrl+Shift+V` and check the table. The chord check, the event and the editor are tested; reading the real clipboard for this chord and writing the refusal to `draw.log` were not exercised in the running app.
- 2026-11-09 — integrated. Left for the owner: paste a sample matrix with `Ctrl+Shift+V`.
