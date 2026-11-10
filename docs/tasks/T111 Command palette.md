---
id: T111
title: Command palette
status: todo
wave: 59
branch: task/T111-command-palette
depends_on: []
adrs: []
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
- **AC-2** — `↑`/`↓` move the highlight, `Enter` runs the highlighted command exactly as its key would and closes the palette, `Esc` closes it doing nothing. While open, keys never reach the tools.
- **AC-3** — The list is built from the keymap table, so every bound command appears and nothing else does (a test compares them).
- **AC-4** — Opening and closing records nothing in the history; the palette is drawn as an overlay only. Fuzzed with random typing.

## Out of scope

- Commands without a key, recently used ordering, arguments.

## Files owned

- a new `src/core/palette.rs`, `src/core/editor.rs`, `src/core/keymap.rs`, `src/core/command.rs`
- `src/shell/` (overlay drawing)
- `docs/features/Shortcuts and macros.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T111-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T111): specify the command palette`
- [ ] tests — `test(T111): add failing tests for the command palette`
- [ ] models — `feat(T111): add the palette model and fuzzy matching`
- [ ] behaviour — `feat(T111): open the palette and run commands from it`
- [ ] quality — `chore(T111): pass clippy and rustfmt`
- [ ] docs — `docs(T111): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from E10 of the [[Feature proposal]].
