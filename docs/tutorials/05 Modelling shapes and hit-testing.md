---
title: Modelling shapes and hit-testing
step: 5
requires: ["[[01 2D vectors, AABBs and point-segment distance]]", "[[02 Palettes and design tokens]]"]
feature: "[[Shapes]]"
code: ["src/core/shape.rs"]
tags: [tutorial]
---

# Modelling shapes and hit-testing

> **Learning path step 5.** Requires: [[01 2D vectors, AABBs and point-segment distance]]
> (and [[02 Palettes and design tokens]] for colours) · Next: *6 Document and undo*

## Why this matters here

A vector editor constantly asks geometric questions about its objects: *how
big is it?* (to skip drawing it off-screen, to outline a selection), *did the
cursor touch it?* (eraser, select), *is this point inside it?* (bucket). One
enum with a handful of methods answers all of them, and every tool reuses it.

## The concept

**A sum type for shapes.** Rust enums with data are "tagged unions": a
`Shape` is exactly one of five variants, and `match` forces every query to
handle all of them. Add a variant and the compiler lists every place to update.

**Bounds first, exact test second.** An axis-aligned box (AABB) is a cheap
over-approximation. If the point is not inside the box grown by the
tolerance, it cannot hit the shape, so the expensive per-segment loop is
skipped for almost every shape on screen.

**A thick outline is a distance test.** A line of width `w` covers every
point within `w/2` of its centre line. Add the click tolerance and the test is
just `distance(p, outline) <= tol + w/2`:

```
          tol + w/2
        |<-------->|
  ------+==========+------   outline (centre line)
        |  hit     |  miss
```

For polylines and rectangles the outline is a set of segments, so the
distance is the minimum of `distance_to_segment` over them.

**Ellipses need an estimate.** The exact distance to an ellipse needs solving
a quartic. Instead, take the implicit function `f(q) = |q / r| − 1` (zero on
the outline) and divide by its gradient length — a first-order Taylor step:
`d ≈ k0 (k0 − 1) / k1` with `k0 = |q / r|` and `k1 = |q / r²|`. It is exact on
both axes and zero on the outline, which is all a click test needs.

**Inside a triangle** (arrow head): the 2D cross product `u × w` tells on
which side of an edge a point lies. A point is inside when it is on the same
side of all three edges.

## How draw implements it

In `src/core/shape.rs`:

- `Shape::bounds` builds the geometry box (`Aabb::from_points` for strokes,
  `Aabb::from_corners` otherwise), unions the `arrow_head` points for arrows,
  then calls `expand(half_width(width))`.
- `Shape::hit` computes `reach = tol + width/2`, rejects non-finite input and
  points outside `bounds().expand(tol)`, accepts filled shapes whose
  `contains(p)` is true, then matches on the variant: strokes use
  `windows(2)` over the points, rectangles use `polygon_edge_distance` over
  their four corners, arrows add `point_in_triangle` on the head, ellipses use
  `ellipse_outline_distance`.
- `Shape::contains` is `Aabb::contains` for rectangles and
  `(x/rx)² + (y/ry)² <= 1` for ellipses; open shapes return `false`.
- `arrow_head` walks back from `b` along the unit shaft direction and steps
  sideways along its perpendicular `(-y, x)`. The renderer draws exactly this
  triangle, so bounds and hits match the picture.
- Degenerate input is handled explicitly: an empty stroke has no hits, a
  zero-length arrow has head `[b, b, b]` (and a zero-area triangle contains
  nothing), a zero-radius ellipse is treated as its segment, and NaN widths
  count as 0. Negated float comparisons are written as `x.is_nan() || x <= …`
  so NaN takes the safe branch.

## Try it

1. Change `ARROW_HEAD_MIN_LENGTH` to `2.0` and run
   `cargo test shape::tests::bounds_arrow` — the test asserts the head sticks
   out past the half width and now fails. Put it back.
2. Add a test `hit_rect_corner_uses_euclidean_distance`: for a 10×10 rect with
   width 0, the point `(11, 11)` is `√2` from the corner, so it hits with
   `tol = 1.5` but not with `tol = 1.3`.
3. Replace the ellipse estimate with a 64-segment polygon and
   `polygon_edge_distance`. Which tests still pass? How large is the error for
   a radius of 1000?

## Further reading

- Inigo Quilez, "Ellipse - distance" (distance estimation by gradient).
- Christer Ericson, *Real-Time Collision Detection*, ch. 5 (closest points,
  point in triangle).
- The Rust Book, ch. 6 "Enums and Pattern Matching".
