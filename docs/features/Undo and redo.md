---
title: Undo and redo
task: "[[T06 Document and history]]"
adrs: ["[[ADR-0005 Undo via transaction log]]", "[[ADR-T06-1 Self-checking edits and rollback atomicity]]"]
tutorial: "[[06 Undo and redo with a transaction log]]"
shortcuts: []
tags: [feature]
---

# Undo and redo

## What it does

Every action on the canvas (drawing a shape, erasing, moving, pasting,
clearing) can be undone and redone, one whole action at a time. Undo goes back
up to 500 actions; starting a new action after undoing discards the redo steps.

## How to use

This note covers the **model** ([[T06 Document and history]]). The shortcuts
are bound by [[T11 Keymap and macros]] and the toolbar buttons by
[[T12 App shell and toolbar]], which extend this table.

| Action | Shortcut / gesture |
|--------|--------------------|
| Undo | `Ctrl+Z` |
| Redo | `Ctrl+Shift+Z` or `Ctrl+Y` |
| Undo / redo from the toolbar | Click the left / right arrow ([[Toolbar]]) |

## How it works

- `src/core/document.rs` — `Document` keeps `(ShapeId, Shape)` pairs in
  z-order. Ids come from a counter that only grows, so an id is never reused,
  even after undo. The only mutation is `Document::apply(&Transaction)`.
- A `Transaction` is a list of `Edit`s (`Insert`, `Remove`, `Replace`), each
  carrying enough data to build its inverse. `apply` checks every edit against
  the stored shapes and rolls back on the first invalid one, so a transaction
  is all-or-nothing ([[ADR-T06-1 Self-checking edits and rollback atomicity]]).
  Non-finite shapes are rejected.
- Builders `tx_insert`, `tx_remove`, `tx_replace`, `tx_clear` produce the
  common transactions for tools.
- `src/core/history.rs` — `History` applies a committed transaction and keeps
  it on an undo stack (capped at `HISTORY_LIMIT = 500`, oldest dropped); undo
  applies `Transaction::inverse()` and moves it to the redo stack
  ([[ADR-0005 Undo via transaction log]]).
- Property test `undo_all_redo_all_symmetry`: any commit sequence, undone
  entirely, gives the empty document; redone entirely, gives the same shapes
  and ids.

## Limits

- The history lives in memory only; it is not saved (persistence is out of
  scope).
- Selection is not part of the history.
- Beyond 500 actions the oldest ones can no longer be undone.
