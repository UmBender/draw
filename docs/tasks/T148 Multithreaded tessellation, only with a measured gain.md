---
id: T148
title: Multithreaded tessellation, only with a measured gain
status: todo
wave: 72
branch: task/T148-multithreaded-tessellation-only-with-a-m
depends_on: [T146]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T148 Multithreaded tessellation, only with a measured gain

## Goal

Chunk tessellation runs on scoped threads if, and only if, a benchmark on the target shows the gain the ADR names.

## Known facts (2026-11-12)

- [[2.0 Architecture proposal]] §7. Scoped `std::thread`, no new dependency. A threaded path ships with a benchmark or it is removed.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — A benchmark of the 5k-stroke scene on the target (i5-8250U, 4 cores) compares one thread with N; the ADR states the minimum gain (proposed: 1.5x at 5k shapes with no loss below 100 shapes).
  `tests/perf.rs` records both numbers.
- **AC-2** — The threaded result is bit-identical to the single-thread one.
  Test: `threads_give_identical_geometry`.
- **AC-3** — Shared state has the thread sanitizer in the nightly job and `loom` models for any lock-free structure.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-gfx/src/tessellate.rs`, `tests/perf.rs`
- this note and `ADR-T148-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T148): specify threaded tessellation`
- [ ] tests — `test(T148): add the benchmark and the equivalence test`
- [ ] behaviour — `feat(T148): tessellate chunks on scoped threads`
- [ ] quality — `chore(T148): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
