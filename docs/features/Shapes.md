---
title: Shapes
task: "[[T05 Shape model]]"
adrs: ["[[ADR-0004 Vector object model]]", "[[ADR-0013 World-space widths and zoom limits]]"]
tutorial: "[[05 Modelling shapes and hit-testing]]"
shortcuts: []
tags: [feature]
---

# Shapes

## What it does

Everything on the canvas is a vector shape: a freehand stroke, a line, an
arrow, a rectangle or an ellipse. Each has a palette colour and a width;
rectangles and ellipses can also be filled. Because shapes are vectors, they
stay sharp at any zoom and can be moved, copied, erased and filled as whole
objects.

## How to use

This note covers the **model** ([[T05 Shape model]]). Drawing shapes is added
by [[T09 Creation tools]] and rendering by [[T07 Renderer]], which extend this
note with their shortcuts.

| Action | Shortcut / gesture |
|--------|--------------------|
| *(tools added by T09)* | — |

## How it works

`src/core/shape.rs` ([[ADR-0004 Vector object model]]):

- `Style { color: ColorId, width: f32 }` — width in world units
  ([[ADR-0013 World-space widths and zoom limits]]).
- `Shape` — `Stroke { points }`, `Line { a, b }`, `Arrow { a, b }` (head at
  `b`), `Rect { a, b, fill }`, `Ellipse { a, b, fill }`; for rect and ellipse
  `a`, `b` are opposite corners of the bounding box in any order.
- `bounds()` — box grown by half the width, plus the arrow head; used for
  AABB rejection and the selection outline.
- `hit(p, tol)` — "is the cursor on this shape?" for the eraser and the
  selection tool: within `tol + width/2` of the outline, or inside a filled
  rect/ellipse. Rejects by bounds first, then measures to segments, polygon
  edges, the arrow head triangle or the ellipse outline.
- `contains(p)` — interior test of rect and ellipse, used by the bucket
  (which only fills closed shapes, ADR-0004).
- `translate(delta)`, `with_fill(fill)`, `is_finite()` — for move/paste,
  bucket, and the "all stored coordinates are finite" invariant.
- `arrow_head(a, b, width)` — the head triangle, shared by renderer and
  `bounds` so what you see is what you hit. Head length is
  `max(8, 4 × width)` world units, base half-width half of that.

## Limits

- Ellipse outline distance is an estimate (`k0 (k0 − 1) / k1`): exact on the
  axes and on the outline, slightly off elsewhere for very flat ellipses.
  Good enough for a click tolerance of a few pixels.
- Strokes are open polylines: a stroke that loops back on itself has no
  interior and cannot be filled (see ADR-0004 rollback plan).
- Hit-testing is a linear scan with AABB rejection; no spatial index.
