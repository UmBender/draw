---
id: ADR-T25-1
title: Lazy position index and batched edit runs
status: accepted
kind: decision
date: 2026-10-02
task: T25
builds_on: [ADR-0004, ADR-0005, ADR-T06-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T25-1 Lazy position index and batched edit runs

## Context

`Document` stores shapes in a `Vec` in z-order plus a `HashSet` of ids.
`get` and `index_of` find a shape with a linear scan. Code that walks a
selection calls them once per selected id, so those passes are
O(selection × document). With 2 560 of 5 120 shapes selected they cost
10–45 ms per click or release
([[T25 Linear-time selection with large documents]]).

`apply` also runs every edit on its own:

- a `Replace` scans for its id;
- a `Remove` or `Insert` shifts the tail of the `Vec`.

So deleting k shapes, or undoing that delete, is O(n · k).

## Decision

- **Lazy position index.** `Document` keeps a
  `OnceCell<HashMap<ShapeId, usize>>`:
  - it is built in one O(n) pass on the first lookup after a change, and
    `get`/`index_of` read it in O(1);
  - `Insert` and `Remove` reset it, while `Replace` keeps it (positions
    do not move);
  - a failed transaction is rolled back, so the next lookup rebuilds and
    sees the same document as before.

  The `HashSet` of ids stays for the duplicate-id check.
- **Batched runs in `apply`.** `apply` splits a transaction into runs and
  applies each run in one pass:
  - consecutive `Remove`s with strictly decreasing indices — what
    `tx_remove` and `tx_clear` build. Each index then still names its
    shape in the state before the run;
  - consecutive `Insert`s with strictly increasing indices — their
    inverses, i.e. undo of a delete. Each index is then the shape's final
    position.

  A run is checked completely, with the same rules and in the same order
  as one-by-one application, before anything changes. It then rebuilds the
  `Vec` in one O(n + k) pass. Any other edit runs on its own as before.
  Results and errors are identical to applying the edits one at a time;
  a proptest checks that.

## Alternatives considered

- **Eager index updated on every edit** — inserts and removes in the
  middle shift the position of every later shape, which is O(n) per edit
  again.
- **Keep shapes sorted by id and binary-search** — z-order is not id
  order once undo reinserts a shape at its old position.
- **Change the editor and tools to avoid lookups** — they do not own the
  data structure. One fix in `Document` covers every caller, and the
  public API does not change.

## Consequences

- Lookups after a structural change pay one O(n) rebuild, then O(1).
  Memory: one map entry per shape.
- Deleting, erasing, clearing and undoing them are linear in the document.
- `Document` is no longer `Sync` (because of `OnceCell`). Nothing shares
  it across threads.

## Rollback plan

Remove the index field and the run batching in `core::document`; `get`
and `index_of` scan again. Record as a rollback ADR reverting this one.
