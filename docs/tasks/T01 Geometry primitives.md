---
id: T01
title: Geometry primitives
status: todo
wave: 1
branch: task/T01-geometry
depends_on: [T00]
adrs: ["[[ADR-0003 Headless core and thin shell]]"]
feature:
tutorial: "[[01 2D vectors, AABBs and point-segment distance]]"
tags: [task]
---

# T01 Geometry primitives

## Goal

Dependency-free 2D math used by every other core module.

## Spec

- **AC-1** — `Vec2 { x: f32, y: f32 }` is `Copy`, with `new`, `ZERO`, `+ - * (f32) / (f32)`,
  `length`, `length_sq`, `dot`, `distance`, `lerp`. *Tests:* `geom::tests::vec2_*`.
- **AC-2** — `Vec2::is_finite` and `Vec2::sanitize(self) -> Option<Vec2>` (None if any
  component is NaN/∞). *Tests:* `vec2_sanitize_rejects_nan_and_inf`.
- **AC-3** — `Aabb { min, max }`: `from_points(&[Vec2]) -> Option<Aabb>` (None if empty),
  `from_corners(a, b)` (any order), `expand(f32)`, `contains(Vec2)`, `intersects(&Aabb)`,
  `union`, `center`, `width`, `height`, `translate(Vec2)`. *Tests:* `aabb_*`.
- **AC-4** — `distance_to_segment(p, a, b)` correct for interior projection, both
  endpoints, and degenerate `a == b`. *Tests:* `segment_distance_*` + proptest
  `segment_distance_never_exceeds_endpoint_distance`.
- **AC-5** — `approx_eq(a, b, eps)` helper for tests and logic; no `==` on floats
  anywhere (clippy `float_cmp` deny). *Test:* `approx_eq_*`.

## Out of scope

Transforms (camera owns them), curves.

## Files owned

`src/core/geom.rs`

## Subtasks (one commit each)

- [ ] spec — `docs(T01): specify geometry acceptance criteria`
- [ ] tests — `test(T01): add failing geometry tests`
- [ ] models — `feat(T01): add Vec2 and Aabb types`
- [ ] behaviour — `feat(T01): implement vector, AABB and segment math`
- [ ] quality — `chore(T01): pass clippy and rustfmt`
- [ ] docs — `docs(T01): add geometry tutorial`

## Learning path

Step 1 — requires step 0.

## Log
