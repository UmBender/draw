---
id: T06
title: Document and history
status: in-progress
wave: 3
branch: task/T06-document
depends_on: [T05]
adrs: ["[[ADR-0005 Undo via transaction log]]", "[[ADR-T06-1 Self-checking edits and rollback atomicity]]"]
feature: "[[Undo and redo]]"
tutorial: "[[06 Undo and redo with a transaction log]]"
tags: [task]
---

# T06 Document and history

## Goal

The ordered shape store and the transactional undo/redo log. The only way to
mutate shapes.

## Spec

`ShapeId`, `Document`, `Edit`, `Transaction`, `ApplyError` and the `tx_*` builders
live in `core::document`; `History` lives in `core::history`. Tests are unit tests
in `document::tests` and `history::tests` (`use super::*;`, proptest for the
property). Nothing here panics on any input; edits are checked against the
document before they take effect ([[ADR-T06-1 Self-checking edits and rollback atomicity]]).

- **AC-1** — `ShapeId(u64)` newtype (`Debug, Clone, Copy, PartialEq, Eq, Hash,
  PartialOrd, Ord`, `get() -> u64`). `Document::next_id(&mut self)` hands out strictly
  increasing ids; the counter is never rewound (undo does not give ids back), and
  applying an `Insert` with an id `>=` the counter bumps it past that id.
  *Tests:* `document::tests::ids_never_reused`, `ids_are_strictly_increasing`,
  `insert_with_foreign_id_bumps_counter`.
- **AC-2** — `Document` (`Debug, Clone, Default`) stores `Vec<(ShapeId, Shape)>` in
  z-order (index 0 = bottom). Queries: `shapes()` (double-ended, exact-size iterator of
  `(ShapeId, &Shape)` bottom to top), `get(id) -> Option<&Shape>`,
  `index_of(id) -> Option<usize>`, `len()`, `is_empty()`,
  `topmost_where(pred: FnMut(&Shape) -> bool) -> Option<ShapeId>` (searches top down).
  *Tests:* `new_document_is_empty`, `shapes_iterates_bottom_to_top`,
  `get_known_id_returns_shape`, `get_unknown_id_is_none`, `index_of_reports_z_order`,
  `topmost_where_returns_highest_match`, `topmost_where_no_match_is_none`.
- **AC-3** — `Edit` (`Debug, Clone, PartialEq`) =
  `Insert { index, id, shape }` | `Remove { index, id, shape }` | `Replace { id, before, after }`;
  `Edit::inverse()` swaps Insert↔Remove (same index, id, shape) and swaps
  `before`/`after`. `Transaction(pub Vec<Edit>)` (`Debug, Clone, Default, PartialEq`)
  with `is_empty`, `len`, `edits()` and `inverse()` (inverse edits in reverse order).
  *Tests:* `edit_inverse_insert_is_remove`, `edit_inverse_remove_is_insert`,
  `edit_inverse_replace_swaps_before_after`, `edit_inverse_is_involution`,
  `transaction_inverse_reverses_order`.
- **AC-4** — `Document::apply(&Transaction) -> Result<(), ApplyError>` applies the
  edits in order and is atomic: on the first invalid edit, already applied edits are
  rolled back with their inverses and the shapes and id counter are unchanged.
  An edit is invalid if: `Insert` index `> len`, id already present, or shape
  non-finite; `Remove` index `>= len` or the stored `(id, shape)` at that index
  differs; `Replace` id unknown, `before` differs from the stored shape, or `after`
  non-finite. `ApplyError` (`Debug, Clone, PartialEq, Eq`, implements `Display` and
  `Error`) = `IndexOutOfRange { index, len }` | `UnknownId(ShapeId)` |
  `DuplicateId(ShapeId)` | `Mismatch(ShapeId)` | `NonFiniteShape(ShapeId)`.
  *Tests:* `apply_insert_remove_replace`, `apply_is_atomic_on_error`,
  `apply_rejects_non_finite_shape`, `apply_rejects_bad_index`,
  `apply_rejects_duplicate_id`, `apply_rejects_unknown_id`, `apply_rejects_mismatch`,
  `apply_then_inverse_restores_document`.
- **AC-5** — `History` (`Debug, Clone`, `Default` = `new()`) with
  `HISTORY_LIMIT = 500`, `new()`, `with_limit(n)` (`n` clamped to at least 1).
  `commit(&mut self, doc, tx) -> Result<(), ApplyError>` applies `tx` and pushes it on
  the undo stack, clears redo, and drops the oldest entry beyond the limit; an empty
  transaction is ignored (no-op, redo kept); a failing transaction leaves doc and
  history unchanged. `undo(doc) -> bool` / `redo(doc) -> bool` return `false` when
  there is nothing to do (or the doc was changed behind the history's back — then the
  stacks are unchanged). `can_undo`, `can_redo`, `undo_len`, `redo_len`, `clear`.
  *Tests:* `undo_redo_single_insert`, `undo_redo_empty_stacks_return_false`,
  `commit_clears_redo`, `commit_empty_transaction_is_ignored`,
  `commit_failing_transaction_changes_nothing`, `history_capped`.
- **AC-6** — Builders (return a `Transaction`, possibly empty, never touch the shape
  list): `tx_insert(&mut doc, shapes)` allocates ids and appends on top in order;
  `tx_remove(&doc, &[ShapeId])` removes the known ids (unknown and duplicate ids
  skipped) in descending index order; `tx_replace(&doc, id, new)` is one `Replace`
  (empty if the id is unknown); `tx_clear(&doc)` removes everything top down.
  *Tests:* `tx_builders_insert_appends_on_top`, `tx_builders_remove_skips_unknown_and_duplicates`,
  `tx_builders_remove_keeps_order_of_rest`, `tx_builders_replace_swaps_shape`,
  `tx_builders_replace_unknown_is_empty`, `tx_builders_clear_empties_document`.
- **AC-7** — Property: any sequence of committed builder transactions followed by
  undo-all yields the empty document; then redo-all yields the same `(id, shape)` list.
  *Test:* proptest `history::tests::undo_all_redo_all_symmetry`.

## Out of scope

Selection, persistence.

## Files owned

`src/core/document.rs`, `src/core/history.rs`

## Subtasks (one commit each)

- [x] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 6 — requires step 5.

## Log
