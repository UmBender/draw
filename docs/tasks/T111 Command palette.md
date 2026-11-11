---
id: T111
title: Command palette
status: in-progress
wave: 59
branch: task/T111-command-palette
depends_on: []
adrs: ["[[ADR-T111-1 Command palette as a modal editor state]]"]
feature: "[[Shortcuts and macros]]"
tutorial: ""
tags: [task]
---

# T111 Command palette

## Goal

`Ctrl+Shift+P` opens a prompt where typing part of a command's name lists matching commands with their keys; `Enter` runs the highlighted one, `Esc` closes it.

## Known facts (2026-11-10)

- `Ctrl+Shift+P` is unbound (`Ctrl+P` is text to plot). Every binding has a description in the keymap table (`src/core/keymap.rs`), which the cheat sheet test already checks.
- The proposal's risk: a second way to do everything in an app built on single keys, so the palette must stay an overlay with no lasting state.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `Ctrl+Shift+P` opens the palette over the canvas; typing filters the commands by their descriptions (case-insensitive, letters in order, best match first), each shown with its key.
  Tests: `palette_ctrl_shift_p_opens`, `palette_typing_filters_rows`, `palette_backspace_widens_the_filter`, `rank_matches_letters_in_order_ignoring_case`, `rank_rejects_letters_out_of_order`, `rank_prefers_contiguous_and_word_start_matches`, `rank_ties_keep_keymap_order`, `chord_label_spells_modifiers_and_keys`, `palette_opens_while_a_text_block_is_open`.
- **AC-2** — `↑`/`↓` move the highlight, `Enter` runs the highlighted command exactly as its key would and closes the palette, `Esc` closes it doing nothing. While open, keys never reach the tools.
  Tests: `palette_arrows_move_the_highlight`, `palette_enter_runs_the_highlighted_command`, `palette_escape_closes_doing_nothing`, `palette_keys_and_pointer_do_not_reach_the_tools`, `palette_ignores_commands_while_open`, `visible_window_follows_the_highlight`.
- **AC-3** — The list is built from the keymap table, so every bound command appears and nothing else does (a test compares them).
  Tests: `rows_are_exactly_the_keymap_bindings`, `palette_chord_is_not_a_binding`.
- **AC-4** — Opening and closing records nothing in the history; the palette is drawn as an overlay only. Fuzzed with random typing.
  Tests: `palette_open_and_close_record_nothing`, fuzz `palette_sessions_never_panic_and_change_nothing`.

Decisions beyond the owner's: [[ADR-T111-1 Command palette as a modal editor state]]. `UNATTENDED CHOICE:` the module is `command_palette.rs` (`palette.rs` holds the drawing colours), so `src/core/mod.rs` is added to the files owned.

## Out of scope

- Commands without a key, recently used ordering, arguments.

## Files owned

- a new `src/core/command_palette.rs`, `src/core/mod.rs`, `src/core/editor.rs`, `src/core/keymap.rs`, `src/core/command.rs`
- `src/shell/` (overlay drawing)
- `docs/features/Shortcuts and macros.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T111-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T111): specify the command palette`
- [ ] tests — `test(T111): add failing tests for the command palette`
- [ ] models — `feat(T111): add the palette model and fuzzy matching`
- [ ] behaviour — `feat(T111): open the palette and run commands from it`
- [ ] quality — `chore(T111): pass clippy and rustfmt`
- [ ] docs — `docs(T111): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from E10 of the [[Feature proposal]].
