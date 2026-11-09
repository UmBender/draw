---
id: T92
title: Export the view to PNG
status: in-progress
wave: 51
branch: task/T92-png-export
depends_on: []
adrs: [ADR-T92-1, ADR-T92-2]
feature: "[[PNG export]]"
tutorial: ""
tags: [task]
---

# T92 Export the view to PNG

## Goal

`Ctrl+E` writes the current view to a PNG file. (idea K4 of [[Feature proposal]])

## Known facts (2026-11-09)

- macroquad's `Image::export_png` panics when the write fails, and `panic = "abort"` would lose the drawing: do not use it.
- Owner's decision (2026-11-09): use the `png` crate directly. It is already in `Cargo.lock` through macroquad, so `ADR-T92-1` amends [[ADR-0014 Minimal dependencies and no unsafe]].
- Read-back gives rows bottom-up, as `export_png` shows.

## Spec

Design: [[ADR-T92-1 The png crate as a direct dependency]], [[ADR-T92-2 Ctrl+E reads back the document layer]].

- **AC-1** — `Ctrl+E` writes the drawing as seen in the view, without the toolbar, selection or previews, to `~/draw-<UTC date>-<time>.png`.
  *Tests:* `core::keymap::resolve_ctrl_e_exports_the_view`, `core::editor::export_view_when_idle_asks_once`, `shell::export::export_path_uses_the_utc_date_and_time`, `shell::export::view_pixels_crops_the_margin`; e2e case `export-view`, which exports with the toolbar and a selection showing and compares the file with the window after hiding both.
- **AC-2** — An existing file is not overwritten: a `-1`, `-2` suffix is added.
  *Tests:* `shell::export::write_new_existing_file_takes_the_next_suffix`, `shell::export::write_new_leaves_existing_files_untouched`, `shell::export::export_path_adds_the_attempt_as_a_suffix`.
- **AC-3** — A failed write (no `HOME`, read-only disk) never panics and goes to the diagnostics log; the path goes to the log on success too.
  *Tests:* `shell::export::home_dir_unset_or_empty_is_an_error`, `shell::export::write_new_in_an_unusable_directory_is_an_error`, `shell::export::save_without_home_is_an_error`, `shell::export::log_line_success_is_info_with_the_path`, `shell::export::log_line_failure_is_an_error_with_the_reason`.
- **AC-4** — Rows are in the right order. *Test:* a pure function that encodes RGBA to PNG bytes, decoded back in a test.
  *Tests:* `shell::export::encode_png_round_trips_rows_in_order`, `shell::export::encode_png_wrong_length_is_an_error`, `shell::export::encode_png_zero_size_is_an_error`, `shell::export::view_pixels_puts_the_rows_top_down`, `shell::export::view_pixels_makes_every_pixel_opaque`, `shell::export::view_pixels_wrong_length_is_an_error`, `shell::export::view_pixels_view_larger_than_layer_is_an_error`.
- **AC-5** — e2e case: `Ctrl+E` produces a decodable PNG of the window's size.
  *Tests:* e2e case `export-view` through the new `export` directive; harness self-tests `parse_input_reads_the_export_option`, `find_export_needs_exactly_one_file`, `decode_png_gives_the_rgb_rows_in_order`.
- **AC-6** — `Ctrl+E` is ignored during text edit and a gesture.
  *Tests:* `core::editor::export_view_during_text_edit_is_ignored`, `core::editor::export_view_command_during_text_edit_keeps_the_block_open`, `core::editor::export_view_during_gesture_is_ignored`, `shell::input_map::export_chord_repeat_is_dropped`; fuzz `export_requests_never_change_the_editor`.

## Out of scope

- Exporting the whole document beyond the view.
- SVG.
- A file dialog.

## Files owned

- `Cargo.toml`, `Cargo.lock`
- `src/shell/app.rs`, `src/shell/render.rs`, `src/shell/input_map.rs`
- new `src/shell/export.rs`, `src/shell/mod.rs`
- `tests/e2e/**`, `tests/e2e.rs`
- `docs/features/PNG export.md` (new)
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T92-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T92): specify export the view to png`
- [ ] tests — `test(T92): add failing tests`
- [ ] models — `feat(T92): add types`
- [ ] behaviour — `feat(T92): implement`
- [ ] quality — `chore(T92): pass clippy and rustfmt`
- [ ] docs — `docs(T92): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — UNATTENDED CHOICE: added `tests/e2e.rs` to *Files owned*; the harness needs an `export` directive to decode the file and compare it with the window.
- 2026-11-09 — UNATTENDED CHOICE: the file name reuses `diag::file::utc_stamp`, giving `draw-20261109-143005.png`, instead of a second date helper.
- 2026-11-09 — UNATTENDED CHOICE: the underlay dot grid is exported while grid snap is on, because AC-1 asks for the view as seen and does not list it among the exclusions (ADR-T92-2).
- 2026-11-09 — UNATTENDED CHOICE: auto-repeats of `Ctrl+E` are dropped in `Collector`, so that holding the keys writes one file and not one per repeat (ADR-T92-2).
