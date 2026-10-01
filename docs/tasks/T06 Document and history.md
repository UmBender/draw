---
id: T06
title: Document and history
status: todo
wave: 3
branch: task/T06-document
depends_on: [T05]
adrs: ["[[ADR-0005 Undo via transaction log]]"]
feature: "[[Undo and redo]]"
tutorial: "[[06 Undo and redo with a transaction log]]"
tags: [task]
---

# T06 Document and history

## Goal

The ordered shape store and the transactional undo/redo log. The only way to
mutate shapes.

## Spec

- **AC-1** — `ShapeId(u64)` newtype, monotonically allocated by `Document::next_id()`,
  never reused (even after undo). *Test:* `document::tests::ids_never_reused`.
- **AC-2** — `Document` stores `Vec<(ShapeId, Shape)>` in z-order; `shapes()` iterator,
  `get(id)`, `index_of(id)`, `len`, `is_empty`, `topmost_where(pred) -> Option<ShapeId>`.
  *Tests:* `get_*`, `topmost_*`.
- **AC-3** — `Edit { Insert { index, id, shape }, Remove { index, id, shape }, Replace { id, before, after } }`
  and `Transaction(Vec<Edit>)`; `Edit::inverse()`. *Tests:* `edit_inverse_*`.
- **AC-4** — `Document::apply(&Transaction) -> Result<(), ApplyError>` is atomic:
  on any invalid edit (bad index, unknown id, non-finite shape) nothing changes.
  *Tests:* `apply_is_atomic_on_error`, `apply_rejects_non_finite_shape`.
- **AC-5** — `History::commit(doc, tx)` applies and pushes to undo, clears redo; empty
  transactions are ignored. `undo(doc)` / `redo(doc)` return `bool`. Cap = 500 (oldest dropped).
  *Tests:* `undo_redo_*`, `commit_clears_redo`, `history_capped`.
- **AC-6** — Helpers building common transactions: `tx_insert(doc, shapes)`,
  `tx_remove(doc, ids)`, `tx_replace(doc, id, new)`, `tx_clear(doc)`.
  *Tests:* `tx_builders_*`.
- **AC-7** — Property: any sequence of commits followed by undo-all yields the empty
  document; then redo-all yields the same shapes and ids. *Test:* proptest `undo_all_redo_all_symmetry`.

## Out of scope

Selection, persistence.

## Files owned

`src/core/document.rs`, `src/core/history.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 6 — requires step 5.

## Log
