---
title: Indexing a z-ordered store
step: 25
requires: ["[[06 Undo and redo with a transaction log]]", "[[14 Profiling and shipping a release build]]"]
feature:
code: ["src/core/document.rs", "src/core/tools/eraser.rs", "tests/perf.rs"]
tags: [tutorial]
---

# Indexing a z-ordered store

> **Learning path step 25.** Requires: steps 6 and 14 · Next: —

## Why this matters here

Select all, duplicate, and you have 2 560 of 5 120 shapes selected. Every
click then did 2 560 lookups, and each lookup scanned the shape list: about
6.5 million comparisons, 10–45 ms. Nothing looked wrong in the code. Every
line was "just a lookup". The cost only shows when a loop over one
collection calls a linear search over another: O(selection × document).

## The concept

**Hidden quadratics.** `for id in selection { doc.get(id) }` reads as
linear, but with a linear `get` it is quadratic. Profiles show it as time
in an innocent function: here, `ShapeId`'s `==`.

**Order and lookup are two jobs.** The document needs a *sequence* (z-order:
what is drawn on top) and a *map* (id → shape). A `Vec` is a great sequence
and a bad map. A `HashMap<ShapeId, usize>` from id to position is the map.
The catch: inserting or removing in the middle of the `Vec` shifts every
later position.

**Build it lazily.** Instead of keeping the map up to date on every edit
(O(n) per insert or remove again), throw it away on a structural change and
rebuild it in one O(n) pass on the next lookup (`OnceCell`). A pass of k
lookups then costs O(n + k). A `Replace` does not move anything, so it
keeps the map.

**Batch what you can prove is safe.** Deleting k shapes one by one is k
`Vec::remove`s, each shifting the tail. When the indices strictly decrease
(top-down, as `tx_remove` builds them), each index still names its shape in
the original list. So you can:

1. check every edit first;
2. drop all of them in one `retain` pass.

Undo applies the inverse: inserts with strictly increasing indices, where
each index is the shape's *final* position. That makes it a single merge
of two sorted sequences.

**Keep an oracle.** Fast paths are easy to get subtly wrong, and errors
must match too: the same `ApplyError` from the same edit. The tests keep a
ten-line reference that applies edits one by one to a plain `Vec`, and a
proptest throws random valid and invalid transactions at both.

## How draw implements it

- `Document::positions` (`src/core/document.rs`) builds the map on first
  use. `apply_edit` resets it on `Insert`/`Remove`. `get` and `index_of` go
  through it.
- `Run::at` splits a transaction into `Removes(n)` / `Inserts(n)` / `Single`.
  `apply_remove_run` and `apply_insert_run` check in edit order, reporting
  the same `len` a one-by-one apply would see, then rewrite the `Vec` once.
  `roll_back` replays the inverses through the same runs, so a failed long
  transaction also rolls back in linear time.
- The eraser (`src/core/tools/eraser.rs`) keeps `marked_ids` and
  `cleared_at` next to its ordered lists, so "already marked?" is O(1) while
  the preview keeps touch order.
- `tests/perf.rs` has four budgets on 10 000 shapes with 5 000 selected:
  lookup, move commit, delete, and undo of the delete. `median_time` stops
  the clock before dropping an op's output, so returning the document is
  not timed.

## Try it

1. Run `cargo test --release --test perf -- --ignored --nocapture
   --test-threads=1` and note the four T25 lines.
2. In `index_of`, go back to `self.shapes.iter().position(...)` and run them
   again: the lookup and move-commit lines jump by more than 10×.
3. In `Run::at`, return `Self::Single` always: delete and undo get slower,
   but every correctness test stays green. That is what the oracle is for.
4. Break the batched path on purpose (for example, report `len` instead of
   `len - done` in `apply_remove_run`) and watch
   `apply_matches_edit_by_edit_application` shrink to a minimal failing
   transaction.

## Further reading

- [[ADR-T25-1 Lazy position index and batched edit runs]].
- [[ADR-T06-1 Self-checking edits and rollback atomicity]] — why every edit
  carries what it expects to find.
