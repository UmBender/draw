---
id: T05
title: Shape model
status: todo
wave: 2
branch: task/T05-shape
depends_on: [T01, T02]
adrs: ["[[ADR-0004 Vector object model]]"]
feature: "[[Shapes]]"
tutorial: "[[05 Modelling shapes and hit-testing]]"
tags: [task]
---

# T05 Shape model

## Goal

The vector shapes stored in the document, with the geometric queries tools need.

## Spec

- **AC-1** — `Style { color: ColorId, width: f32 }`; `Shape` enum:
  `Stroke { points: Vec<Vec2>, style }`, `Line { a, b, style }`, `Arrow { a, b, style }`,
  `Rect { a, b, style, fill: Option<ColorId> }`, `Ellipse { a, b, style, fill: Option<ColorId> }`
  (`a`, `b` = opposite corners of the bounding box). *Test:* compile + `shape::tests::construct_*`.
- **AC-2** — `bounds() -> Aabb` includes half the stroke width (and arrow head).
  *Tests:* `bounds_*` per variant.
- **AC-3** — `hit(p, tol) -> bool`: true if `p` is within `tol + width/2` of the outline
  (strokes: any segment; single-point stroke: the point). Filled shapes also hit in
  their interior. *Tests:* `hit_*` per variant, inside/outside/edge.
- **AC-4** — `contains(p) -> bool` for closed shapes (rect, ellipse interior; others
  false) — used by the bucket. *Tests:* `contains_*`.
- **AC-5** — `translate(delta)` moves every point; `with_fill(Option<ColorId>)` on
  closed shapes. *Tests:* `translate_*`, proptest `translate_moves_bounds_by_delta`.
- **AC-6** — `arrow_head(a, b, width) -> [Vec2; 3]` geometry (head length ∝ width,
  min size) shared by renderer and bounds. *Test:* `arrow_head_*`.
- **AC-7** — `is_finite()` true iff all coordinates and width are finite. *Test:* `is_finite_*`.

## Out of scope

Rendering, storage, ids.

## Files owned

`src/core/shape.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 5 — requires step 1 (and step 2 for colours).

## Log
