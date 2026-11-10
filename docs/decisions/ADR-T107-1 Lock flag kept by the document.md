---
id: ADR-T107-1
title: Lock flag kept by the document
status: accepted
kind: decision
date: 2026-11-10
task: T107
builds_on: [ADR-T06-1, ADR-T74-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T107-1 Lock flag kept by the document

## Context

T107 locks shapes so a background grid or diagram cannot be selected,
moved, resized or erased by accident, while it still takes bucket fills
and text. Locking and unlocking must be undo steps, survive snapshots
and the autosave, and never let a locked shape vanish except through
undo. `Shape` is an enum of seven variants matched all over the code
base; a `locked: bool` in each variant would touch every pattern and
constructor for a flag no shape method needs.

## Decision

- **Storage.** The `Document` keeps the set of locked ids next to its
  shapes. `Document::is_locked(id)` reads it. A new shape is always
  unlocked. Document equality counts the set.
- **Edit.** A new reversible edit `Edit::Lock { id, locked }` sets the
  flag of a present shape; it is a `Mismatch` unless the flag currently
  differs, so its inverse (`locked` flipped) restores the state exactly.
  It holds no shape, so its heap estimate is zero.
- **Removal guard.** `Document::apply` rejects an `Edit::Remove` of a
  locked shape with `ApplyError::Locked`. `tx_remove` and `tx_clear` skip
  locked shapes and `tx_restack` leaves them where they are, so no
  command builds a transaction that could drop one. Undo still removes
  a shape after first undoing its lock, because the history is a stack.
- **Builders.** `tx_lock(doc, ids)` locks the unlocked, known ids of
  `ids` (each once, in z-order); `tx_unlock_all(doc)` unlocks every
  locked shape in z-order. Both are empty when there is nothing to do,
  so the history records nothing.
- **Selection.** A locked id is never selected: click and marquee
  selection skip locked shapes (`Document::topmost_unlocked_where`),
  `Ctrl+A` selects only unlocked ones, and the selection is pruned of
  locked ids after every command, undo and redo. Every selection
  command (delete, cut, copy, duplicate, move, resize, scale, align,
  distribute, restack, grid arrows, text to grid/graph/plot) therefore
  never sees a locked shape.
- **Eraser.** The eraser ignores locked shapes in both modes: they are
  neither marked for removal nor have their fills cleared, and they do
  not decide the mode of a press. Erasing over a locked table removes
  only the unlocked ink on top of it.
- **Clear canvas.** `Ctrl+Backspace` removes the unlocked shapes only;
  with nothing but locked shapes it records nothing.
- **Fills and text.** The bucket and the text tool still pick locked
  shapes (they use `topmost_where`), so a locked grid works as a table.
  Emptying a locked text block cannot remove it: the edit is dropped and
  the block keeps its old text.
- **Snapping.** Snap and smart-guide targets keep using every shape,
  locked or not.
- **No visible mark** on locked shapes (owner); the render path does not
  read the flag.
- **Snapshots.** Format version 2 (from 1). The `[document]` section
  gains one optional record after the shapes, `locked <id>…`, listing
  the locked ids in z-order, written only when any shape is locked; the
  history gains `lock <id>` and `unlock <id>` edit records. A locked id
  must be in the document and appear once; the selection and the
  select and eraser gesture ids must not be locked. Every committed
  fixture is regenerated (only its version line changes).

## Alternatives considered

- **A `locked` field in every `Shape` variant** — lock and unlock would
  be plain `Replace` edits and the flag would ride along every clone,
  but every match and constructor of `Shape` (hundreds) changes, and
  geometry code would carry a flag it never reads.
- **Unlock before remove inside `tx_remove`/`tx_clear`** (so clear
  canvas would also clear locked shapes) — keeps "clear" literal, but
  makes the protection depend on every caller; the owner wants locked
  shapes safe from accidental removal, and `Ctrl+Shift+K` then
  `Ctrl+Backspace` still clears everything.
- **Eraser clears fills of locked shapes** — handy for a table, but the
  eraser is the accident the lock guards against; the bucket still
  repaints cells.

## Consequences

- One more `Edit` variant: code matching `Edit` exhaustively (snapshot,
  clipboard) handles it.
- A stale autosave of version 1 is not read after the upgrade (the
  version check refuses it, as for every format change).
- Removing a locked shape needs `Ctrl+Shift+K` first.

## Rollback plan

Revert the T107 commits and write a rollback ADR; snapshots go back to
version 1 and the fixtures are regenerated again.
