---
title: Undo and redo with a transaction log
step: 6
requires: ["[[05 Modelling shapes and hit-testing]]"]
feature: "[[Undo and redo]]"
code: ["src/core/document.rs", "src/core/history.rs"]
tags: [tutorial]
---

# Undo and redo with a transaction log

> **Learning path step 6.** Requires: [[05 Modelling shapes and hit-testing]]
> · Next: *7 Rendering*

## Why this matters here

A scratch canvas is used fast and carelessly: you draw, erase the wrong thing,
clear the board by accident. Undo must always work, for every action, and it
must be cheap on a weak laptop with thousands of strokes.

## The concept

**Record changes, not states.** Instead of copying the whole document before
each action (a snapshot), record *what changed*. Each change is an `Edit`
that knows how to reverse itself:

| Edit | Inverse |
|------|---------|
| `Insert { index, id, shape }` | `Remove { index, id, shape }` |
| `Remove { index, id, shape }` | `Insert { index, id, shape }` |
| `Replace { id, before, after }` | `Replace { id, before: after, after: before }` |

One user action can touch many shapes (clear, paste), so edits are grouped
into a **transaction**. To undo a transaction, apply the inverse of each edit
*in reverse order*: like taking off socks and shoes, the last thing done is
the first thing undone.

```
commit  T1, T2, T3       undo: [T1 T2 T3]   redo: []
undo    (apply T3⁻¹)     undo: [T1 T2]      redo: [T3]
undo    (apply T2⁻¹)     undo: [T1]         redo: [T3 T2]
commit  T4               undo: [T1 T4]      redo: []   ← new action clears redo
```

**Atomicity.** A transaction either applies completely or not at all, the way
a database transaction does. If the third edit of five is invalid, the first
two are rolled back with their inverses.

**Stable ids.** Shapes have a `ShapeId` from a counter that only grows. Undo
does not give ids back, so a redo or a later insert can never confuse two
shapes.

## How draw implements it

- `Edit::inverse` and `Transaction::inverse` in `src/core/document.rs` build
  the reverse change; `inverse().inverse()` is the original edit.
- `Document::apply` loops over the edits calling the private `apply_edit`. On
  the first error it calls `roll_back` on the edits already applied and
  restores the id counter. `apply_edit` checks that a `Remove` finds exactly
  the `(id, shape)` it expects and that a `Replace` finds `before`, so a stale
  transaction fails loudly instead of corrupting the document
  ([[ADR-T06-1 Self-checking edits and rollback atomicity]]).
- `tx_remove` and `tx_clear` emit removals **top down** (descending index), so
  removing one shape never shifts the index of a shape still to be removed.
- `History` in `src/core/history.rs` keeps the undo stack as a `VecDeque` so
  the oldest entry can be dropped in O(1) when the cap of 500 is reached.
  `undo` pops a transaction, applies its inverse and pushes it to `redo`; if the
  document was changed behind the history's back, the apply fails, the
  transaction goes back where it was and `undo` returns `false`.

## Try it

1. In `history::tests`, write a test that commits `tx_insert` of three shapes,
   then `tx_clear`, then undoes once. Check that all three shapes are back with
   the **same ids** in the same order.
2. Change `tx_clear` to remove bottom up (drop the `.rev()` in
   `removals_top_down`). Which tests fail, and why does the index of every
   later removal become wrong?
3. Run `PROPTEST_CASES=20000 cargo test undo_all_redo_all_symmetry` and read
   how the property generates random action sequences.

## Further reading

- [[ADR-0005 Undo via transaction log]] — why a log instead of snapshots.
- Command pattern (Gamma et al., *Design Patterns*) — the classic object-
  oriented form of the same idea.
- Write-ahead logging in databases — the same atomicity and undo idea at a
  larger scale.
