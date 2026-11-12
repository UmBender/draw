---
id: ADR-T133-1
title: Typed layer added beside the public API
status: accepted
kind: decision
date: 2026-11-12
task: T133
builds_on: [ADR-T132-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T133-1 Typed layer added beside the public API

## Context

T133 wants coordinates, ids and counts that cannot be mixed up. The
shell (`src/`) consumes `draw-core` with plain `Vec2`/`f32`,
`ShapeId(u64)` and `Vec<Vec2>` strokes, and it is not in T133's *Files
owned*. Changing those public signatures would stop the shell from
compiling, and no snapshot fixture may change.

## Decision

- `draw-types` gains `units` (`World`, `Screen`, `Px`, `Point<U>`,
  `Rect<U>`, `Len<U>`), `ids` (`Arena<T>`, `Id<T>`, `PageIx`,
  `PAGE_COUNT`), `zoom` (`Zoom`) and `nonempty` (`NonEmpty<T>`).
- A value changes unit only through `Camera`; the unit types have no
  public cast between units.
- `draw-core` migrates what it can without a breaking change: `Camera`
  stores a `Zoom` and gains typed methods next to the `Vec2` ones;
  `PAGE_COUNT` comes from `draw-types`; the snapshot reader validates
  stroke points through `NonEmpty`.
- Moving `ShapeId`, `Stroke` and the `Vec2` call sites to the typed
  layer is left for a follow-up that owns the shell.
- Compile-fail tests are `crates/draw-types/tests/compile_fail/*.rs`
  checked by `scripts/compile_fail.sh` with `rustc`; no new dependency.

## Alternatives considered

- **Break the public API now** — the shell would not build.
- **`trybuild`** — a new dependency; the owner ruled it out.

## Consequences

The new code is usable immediately; the old `Vec2` API stays until the
shell migrates, so AC-4 is met as "no behaviour change", not as "no
`Vec2` left in `draw-core`".

## Rollback plan

Revert the T133 commits; nothing else depends on them yet.
