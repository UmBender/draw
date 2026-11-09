---
id: T83
title: Paste text from the clipboard
status: done
wave: 51
branch: task/T83-paste-text
depends_on: []
adrs: ["[[ADR-T83-1 Paste text through an input event]]"]
feature: "[[Text blocks]]"
tutorial: ""
tags: [task]
---

# T83 Paste text from the clipboard

## Goal

`Ctrl+V` while a text block or shape text is open inserts the system clipboard's text at the caret. (idea K1 of [[Feature proposal]])

## Known facts (2026-11-09)

- `miniquad::window::clipboard_get()` exists in the vendored miniquad, on Wayland and X11. `src/shell` does not use it yet.
- While text is open, no normal binding fires ([[ADR-T29-2 Text edit mode]]), and `Ctrl+V` is not one of the edit-mode keys.
- `core` must stay free of macroquad, so the shell reads the clipboard and hands `core` a string, e.g. a new `InputEvent`.

## Spec

Design: [[ADR-T83-1 Paste text through an input event]].

- **AC-1** — `Ctrl+V` in an open edit inserts the given text at the caret, and the caret ends after it.
  Tests: `text::tests::edit_paste_inserts_at_caret_and_moves_past_it`, `editor::tests::paste_event_in_open_edit_inserts_at_caret`, `editor::tests::paste_event_in_shape_text_goes_to_the_shape`, `input_map::tests::wants_clipboard_only_for_ctrl_v_while_editing`.
- **AC-2** — Line breaks are kept (`\r\n` and `\r` become `\n`); other control characters are dropped; a tab becomes the same spaces as the `Tab` key.
  Tests: `text::tests::clean_paste_normalises_line_breaks`, `text::tests::clean_paste_drops_control_characters`, `text::tests::clean_paste_tab_is_the_tab_key_spaces`, `editor::tests::paste_event_in_grid_cell_keeps_line_breaks`.
- **AC-3** — A paste longer than 64 KiB is cut at a char boundary to 64 KiB. The limit counts the cleaned text.
  Tests: `text::tests::clean_paste_cuts_at_the_limit`, `text::tests::clean_paste_cuts_on_a_char_boundary`, `text::tests::clean_paste_any_text_is_bounded_and_clean`.
- **AC-4** — An empty or unavailable clipboard changes nothing and never panics.
  Tests: `text::tests::edit_paste_of_nothing_changes_nothing`, `editor::tests::paste_event_with_nothing_to_insert_changes_nothing`, `editor::tests::paste_event_without_open_edit_is_ignored`, `input_map::tests::paste_event_needs_some_text`, and `Paste` events in `tests/fuzz.rs`.
- **AC-5** — Cancelling the edit with `Ctrl+Z` also discards pasted text; outside an edit `Ctrl+V` still pastes shapes.
  Tests: `editor::tests::paste_event_then_ctrl_z_discards_the_new_block`, `editor::tests::paste_event_then_ctrl_z_keeps_the_old_text`, `editor::tests::ctrl_v_outside_edit_still_pastes_shapes`, `input_map::tests::wants_clipboard_only_for_ctrl_v_while_editing`.
- **AC-6** — e2e case: text placed on the clipboard (`wl-copy` if installed, else draw's own `clipboard_set` in a test hook) is pasted into a block.
  Tests: e2e case `tests/e2e/paste-text-clipboard/`, harness tests `self_tests::parse_input_clipboard_is_one_step`, `self_tests::parse_input_rejects_clipboard_text_that_looks_like_an_option`, hook tests `e2e::tests::hooks_take_clipboard_returns_the_text_once`, `e2e::tests::hooks_without_clipboard_take_nothing`.

## Out of scope

- Copying text out of a block; selecting text in a block.
- Pasting images.

## Files owned

- `src/core/tools/text.rs`
- `src/shell/input_map.rs`, `src/shell/app.rs`
- `tests/e2e/**`
- `docs/features/Text blocks.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- `src/core/tap.rs`, `src/core/snapshot.rs`, `src/shell/toolbar.rs`, `src/shell/diag/e2e.rs`, `tests/e2e.rs` (added, see *Log*)
- `vendor/miniquad/src/lib.rs`, `vendor/miniquad/PATCHED.md` (added, see *Log*)
- any other test that stops compiling because `InputEvent` is no longer `Copy`
- this note, and new `ADR-T83-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T83): specify paste text from the clipboard`
- [x] tests — `test(T83): add failing tests`
- [x] models — `feat(T83): add types`
- [x] behaviour — `feat(T83): implement`
- [x] quality — `chore(T83): pass clippy and rustfmt`
- [x] docs — `docs(T83): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — UNATTENDED CHOICE: `InputEvent::Paste(String)` makes `InputEvent` non-`Copy`, so `src/core/tap.rs`, `src/shell/toolbar.rs` (`Route` holds an event) and a test in `src/core/snapshot.rs` join *Files owned*.
- 2026-11-09 — UNATTENDED CHOICE: `tests/e2e.rs` (the `clipboard` directive) and `src/shell/diag/e2e.rs` (the `clipboard_set` hook) join *Files owned* for AC-6. The nightly CI job does not install `wl-copy`, so the hook is what runs there.
- 2026-11-09 — UNATTENDED CHOICE: `vendor/miniquad/src/lib.rs` and `PATCHED.md` join *Files owned*: `clipboard_get` held the display mutex across a Wayland round trip whose handlers lock it, so `Ctrl+V` with the mouse moving would hang (ADR-T83-1).
- 2026-11-09 — UNATTENDED CHOICE: the 64 KiB limit counts the cleaned text, and a tab that does not fit whole ends the paste there (ADR-T83-1).
- 2026-11-09 — UNATTENDED CHOICE: a pasted line break stays in a grid cell, although the `Enter` key walks to the next cell there (ADR-T83-1).
- 2026-11-09 — UNATTENDED CHOICE: `docs/architecture/Architecture.md` and `docs/features/End-to-end tests.md` join *Files owned*, for the data flow and the new `clipboard` directive.
- 2026-11-09 — `paste-text-clipboard` passed through both paths: `wl-copy`, which is installed here, and the hook, forced by temporarily renaming the tool in the harness. The nightly CI job has no `wl-copy`, so it takes the hook path.
- 2026-11-09 — not covered by an automated test: the vendored lock fix. `paste-text-clipboard` runs the read, but moving the pointer at the exact moment of the round trip cannot be timed in the suite.
- 2026-11-09 — manual check for the owner: copy text from a browser and paste it into a block on the desktop. Also press `Ctrl+V` while moving the mouse; the app must not freeze.
- 2026-11-09 — integrated. Left for the owner: copy text from a browser and paste it into a block.
