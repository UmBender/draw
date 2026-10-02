---
id: ADR-T25-2
title: Id hasher and selection-scale budgets
status: accepted
kind: decision
date: 2026-10-02
task: T25
builds_on: [ADR-T25-1, ADR-T14-1, ADR-0014]
amends: [ADR-T14-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T25-2 Id hasher and selection-scale budgets

## Context

[[ADR-T25-1 Lazy position index and batched edit runs]] made selection
passes and large transactions linear. On the target, with 10 000 shapes and
5 000 selected, the medians were:

| Operation | Median |
|-----------|--------|
| Lookup | 0.29–0.49 ms |
| Delete | 0.48–1.37 ms |
| Move commit | 0.92–1.38 ms |
| Undo delete | 0.94–1.20 ms |

They sat on the 1 ms budget the T25 spec asked for and failed at random,
the problem ADR-T14-1 had with snapping. About half of the time was the
standard library's SipHash, hashing 10 000–15 000 ids per operation. SipHash
resists hash-flooding by untrusted keys. Shape ids are sequential integers
this process allocates.

## Decision

- `core::document::IdHasher`: one xor and multiply by the 64-bit golden
  ratio per `u64` (Fibonacci hashing). `IdBuildHasher` and `IdSet` aliases
  use it for the document's id set, its position map and `tx_remove`'s
  lookup set. It is written in the module, with no new dependency
  ([[ADR-0014 Minimal dependencies and no unsafe]]).
- New budgets in `tests/perf.rs`, all **< 1 ms** median, on the
  ADR-T14-1 benchmark document (10 000 shapes) with every other shape
  selected:
  - lookup of the 5 000 selected ids;
  - applying a move (5 000 `Replace` edits);
  - applying `tx_remove` of the 5 000;
  - applying its inverse (undo).
- `median_time` stops the clock before dropping the op's output, so an op
  that hands back a 10 000-shape document is not charged for freeing it.
  The existing budgets are unaffected: their outputs are numbers and
  options.

Measured after the change, three runs, load ≈ 1:

| Operation | Median |
|-----------|--------|
| Lookup | 0.09–0.12 ms |
| Delete | 0.26–0.27 ms |
| Move commit | 0.46–0.47 ms |
| Undo delete | 0.39–0.40 ms |

## Alternatives considered

- **2 ms budgets for the transactions, SipHash kept** — passes, but leaves
  half the cost in place for no benefit.
- **A hashing crate (`rustc-hash`, `ahash`)** — the same result for a new
  dependency, against ADR-0014.
- **`Vec` indexed by id** — no hashing at all, but memory grows with every
  id ever allocated (ids are never reused), including undone ones.

## Consequences

- Id hashing is no longer DoS-resistant. Ids never come from outside the
  process; if documents are ever loaded from files, ids are renumbered on
  load or this is revisited.
- Timings are for an idle target. Under heavy load (a parallel C++ build
  pushed the load average to 10) every budget in `tests/perf.rs` doubles,
  so run them on a quiet machine.

## Rollback plan

Replace the aliases with the std defaults (`HashSet<ShapeId>`,
`HashMap<ShapeId, usize>`) and raise the four budgets. Record it as a new
ADR amending this one.
