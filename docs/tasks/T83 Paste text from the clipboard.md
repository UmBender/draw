---
id: T83
title: Paste text from the clipboard
status: todo
wave: 51
branch: task/T83-paste-text
depends_on: []
adrs: []
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

The spec step names the test for each criterion.

- **AC-1** — `Ctrl+V` in an open edit inserts the given text at the caret, and the caret ends after it.
- **AC-2** — Line breaks are kept (`\r\n` and `\r` become `\n`); other control characters are dropped; a tab becomes the same spaces as the `Tab` key.
- **AC-3** — A paste longer than 64 KiB is cut at a char boundary to 64 KiB.
- **AC-4** — An empty or unavailable clipboard changes nothing and never panics.
- **AC-5** — Cancelling the edit with `Ctrl+Z` also discards pasted text; outside an edit `Ctrl+V` still pastes shapes.
- **AC-6** — e2e case: text placed on the clipboard (`wl-copy` if installed, else draw's own `clipboard_set` in a test hook) is pasted into a block.

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
- this note, and new `ADR-T83-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T83): specify paste text from the clipboard`
- [ ] tests — `test(T83): add failing tests`
- [ ] models — `feat(T83): add types`
- [ ] behaviour — `feat(T83): implement`
- [ ] quality — `chore(T83): pass clippy and rustfmt`
- [ ] docs — `docs(T83): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
