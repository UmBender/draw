---
id: ADR-T148-1
title: Threaded tessellation only with a measured gain
status: proposed
kind: decision
date: 2026-11-15
task: T148
builds_on: []
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T148-1 Threaded tessellation only with a measured gain

## Context

[[T146 Retained renderer in draw-gfx]] tessellates chunks on one thread. The
2.0 proposal allows scoped threads only if a benchmark on the target shows a
gain. The target is the owner's machine (i5-8250U, 4 cores, 8 threads).

## Decision

- `tessellate_many(shapes, threads)` in `tessellate.rs` splits the shapes into
  contiguous runs and tessellates each on a `std::thread::scope` thread. With
  `threads <= 1` it takes the single-thread path. The result is bit-identical
  and in the same order for every `threads`.
- No new dependency, no lock-free structure, no `loom`: the threads share only
  `&[&Shape]` (read) and each returns its own `Vec<Mesh>`.
- Threshold to wire it into the chunk cache: at least 1.5x on the 5 120-stroke
  scene, and no loss on a 100-shape scene (small inputs take the single path
  below `MIN_PARALLEL` shapes).
- Measured numbers are recorded in the *Result* section below.

## Alternatives considered

- **A thread pool or `rayon`** — new dependency.
- **Lock-free queues** — would need `loom` models and a new dependency.

## Consequences

If the gain is below the threshold the function stays as a tested, unused
helper and the cache keeps the single path. Wiring it into `chunks.rs` is a
separate step (that file is not owned by this task).

## Result

To be filled by the measurement.

## Rollback plan

Remove `tessellate_many` and the perf test; nothing else depends on them.
