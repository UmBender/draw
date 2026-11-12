---
title: Undo and redo
task: "[[T06 Document and history]]"
adrs: ["[[ADR-0005 Undo via transaction log]]", "[[ADR-T06-1 Self-checking edits and rollback atomicity]]", "[[ADR-T49-1 Undo history capped by bytes]]"]
tutorial: "[[06 Undo and redo with a transaction log]]"
shortcuts: []
tags: [feature]
---

# Undo and redo

## What it does

Every action on the canvas (drawing a shape, erasing, moving, pasting,
clearing) can be undone and redone, one whole action at a time. Undo goes back
up to 500 actions, fewer when actions on very large selections fill its
64 MiB memory budget; starting a new action after undoing discards the redo
steps.

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
- Both stacks together also stay within `HISTORY_BYTES = 64 MiB`, estimated
  with `Transaction::heap_bytes` / `Shape::heap_bytes`: past it the oldest
  entries go, but the newest is always kept. `History::changes` counts every
  commit, undo and redo that changed the document; the editor's document
  revision follows it ([[ADR-T49-1 Undo history capped by bytes]]).
- `tests/soak.rs` drives long sessions through the editor and checks the
  budget holds (T49).
- Property test `undo_all_redo_all_symmetry`: any commit sequence, undone
  entirely, gives the empty document; redone entirely, gives the same shapes
  and ids.

## Pages (T116)

Each page has its own history: `Ctrl+Z` and `Ctrl+Shift+Z` act on the
current page only, and switching pages with `Alt+1`…`Alt+6` is never an
undo step. The 500-action limit and the 64 MiB budget are per page.

## Limits

- The history lives in memory. The only copy on disk is the autosave, which
  brings it back after `draw --recover` ([[Crash recovery]]).
- Selection is not part of the history.
- Beyond 500 actions, or 64 MiB of history, the oldest ones can no longer
  be undone. Moving a 5 000-stroke selection a few times can already use
  that budget.
