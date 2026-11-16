---
id: T148
title: Multithreaded tessellation, only with a measured gain
status: done
wave: 72
branch: task/T148-multithreaded-tessellation-only-with-a-m
depends_on: [T146]
adrs: [ADR-T148-1]
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
  Refined: `perf_threaded_tessellation_is_measured` (ignored, release) prints
  both numbers for 5 120 strokes and for 100 shapes and asserts the 100-shape
  case does not lose (small inputs use the single path).
- **AC-3** — No shared mutable state: threads read `&[&Shape]` and return their
  own meshes (`ADR-T148-1`), so no `loom` model or lock-free structure exists.
  Test: `small_inputs_use_one_thread` (below `MIN_PARALLEL`, same result).

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-gfx/src/tessellate.rs`, `tests/perf.rs`
- this note and `ADR-T148-*`

## Subtasks (one commit each)

- [x] spec — `docs(T148): specify threaded tessellation`
- [x] tests — `test(T148): add the benchmark and the equivalence test`
- [x] behaviour — `feat(T148): tessellate chunks on scoped threads`
- [x] quality — `chore(T148): pass clippy and rustfmt`

## Log

- 2026-11-15 — done: `tessellate_many` on scoped threads, 3.30x at 5 120 strokes
  (see `ADR-T148-1`). Not wired into `chunks.rs` (not a file this task owns);
  a follow-up must call it from `ChunkCache::update`. No tutorial or feature note
  (frontmatter empty).
- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
