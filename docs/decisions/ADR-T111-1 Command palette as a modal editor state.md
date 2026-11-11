---
id: ADR-T111-1
title: Command palette as a modal editor state
status: accepted
kind: decision
date: 2026-11-11
task: T111
builds_on: ["[[ADR-T11-1 Exact modifier matching]]", "[[ADR-0006 Redraw on demand]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T111-1 Command palette as a modal editor state

## Context

`Ctrl+Shift+P` should open a prompt that lists commands by description with
their keys. The palette must leave no lasting state, and the shortcut sheet
(T113) will need the same "overlay that captures the keys" plumbing.

## Decision

- The editor holds `palette: Option<CommandPalette>`. It is not in snapshots,
  the history or the document: opening and closing change nothing that is
  saved. While it is `Some`, `Editor::handle` sends key, character, paste and
  pointer events to it and `Editor::apply` ignores commands (a toolbar click
  cannot act under the overlay); `Resize`, `Pressure` and `KeyUp` still work.
  T113 can reuse this by making the state an enum of overlays.
- `Ctrl+Shift+P` is **not** a row of `keymap::BINDINGS`: the editor catches it
  itself, also while a text block is open (which it commits first, like any
  command). So the keymap's rule "no binding fires while editing" and the
  existing tests on `BINDINGS` stay true, and the palette never lists itself.
- The rows are `keymap::BINDINGS`, one per binding (a command with two keys
  has two rows), filtered by description.
- Matching: case-insensitive subsequence of the description. Score: 1 per
  matched letter, +3 when it follows the previous matched letter directly,
  +4 at a word start, +2 more at the first letter; best alignment by dynamic
  programming. Sorted by score, ties by keymap order.
- Keys while open: letters through `Char` events edit the query (at most 64
  characters), `Backspace` deletes, `↑`/`↓` move the highlight (clamped),
  `Enter` closes and runs the highlighted command with `apply`, `Esc` closes.
  The highlight returns to the first row when the query changes.
- The new source file is `src/core/command_palette.rs`: `core::palette`
  already holds the drawing colours.
- Drawing: the shell paints a panel with at most 10 rows (a window that
  follows the highlight) from the editor's palette state, with the same text
  font as the toolbar, only while the palette is open. No frames are requested
  for it.

## Alternatives considered

- **Bind `Ctrl+Shift+P` in `BINDINGS`** — it would list itself and fire in the
  text-edit test of every binding.
- **Palette in the shell only** — the keys and the match logic would be
  untestable without a window.

## Consequences

Matching and key handling are pure and fuzzed. One more optional field in the
editor; the shell reads it each frame it draws anyway.

## Rollback plan

Remove `command_palette.rs`, the editor field and the shell overlay.
