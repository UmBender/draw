---
title: Snapping to outlines
step: 20
requires: ["[[17 Snapping and alignment guides]]"]
feature: "[[Snapping]]"
code: ["src/core/snap.rs"]
tags: [tutorial]
---

# Snapping to outlines

> **Learning path step 20.** Requires: step 17 · Next: step 21

## Why this matters here

Graphs are circles joined by arrows. An arrow that stops short of a node,
or pokes into it, reads as sloppy; boxes in a diagram should touch, not
overlap. Alignment (step 17) only knows straight lines through edges and
centres, so it cannot put a point *on a circle*. Outline snapping can.

## The concept

**Nearest point on a curve.** For a target outline `O` and a dragged
point `p`, find the point `q ∈ O` with the smallest `|p − q|`:

- *Rectangle* — the nearest of four segments. For a segment `a → b`,
  project and clamp: `t = clamp((p − a)·(b − a) / |b − a|², 0, 1)`,
  `q = a + t(b − a)`.
- *Grid* — the same over all its lines.
- *Circle* — exact: `q = c + r·(p − c)/|p − c|`. At `p = c` every point is
  nearest; pick one.
- *Ellipse* — no closed form. `q` is where the normal of the ellipse passes
  through `p`, which reduces to finding the root of one monotone function
  of a single variable `s`:

  ```
  F(s) = (r0·z0 / (s + r0))² + (z1 / (s + 1))² − 1
  ```

  (Eberly, *Distance from a point to an ellipse*). `F` is decreasing on a
  known bracket, so **bisection** always converges — also for `p` inside
  the ellipse, on an axis, or at the centre, where Newton's method can
  jump out of the bracket or stall.

**Touching, not overlapping.** Strokes have width. If the arrow tip sat
exactly on `q`, half of the arrow stroke and half of the circle stroke
would overlap. So the point moves on from `q` by
`w_target/2 + w_dragged/2`, along `n = (p − q)/|p − q|` — the side `p`
came from:

```
        circle stroke   arrow stroke
            ▕██▏▕██▏
      ──────▕██▏▕██▏◀── p
            ▕██▏▕██▏
             q   q + n·offset
```

**Priority.** Snaps compete. The pipeline from step 17 gains one step:

```
grid ──▶ size ──▶ align ──▶ outline ──▶ round
                    │          │
                    └ matched? ┘ skip outline   └ fired? skip round
```

Alignment is the more explicit intent, so it wins; outline beats round,
because a circle-shaped box that touches a neighbour matters more than
being exactly round.

## How draw implements it

- `snap::Targets::from_shapes` adds one `snap::Outline` (kind, box, stroke
  width) per rectangle, ellipse and grid.
- `snap::Outline::nearest` returns a `snap::Nearest` (point and unit
  normal). It works in `f64` (`P64`) so boxes near `f32::MAX` neither
  overflow nor lose precision, then clamps the result back into the box.
  `nearest_on_segments` serves rects and grids, `nearest_on_ellipse` and
  `ellipse_quadrant`/`ellipse_root` implement the circle and Eberly cases.
- `snap::snap_outline` picks the nearest outline within
  `tolerance + offset` and applies the offset; when `p` is exactly on the
  outline it uses the outward normal.
- `snap_point` and `snap_drag` call it only when `align_*_matched` reports
  that no anchor matched; `snap_drag` then skips `snap_round` and appends
  `outline_mark` — an × the renderer draws like any other guide.
- The dragged width is `camera.world_len(style.width_px)`, passed in by
  `snap_start`/`snap_end`, so the shape tool did not change.

## Try it

1. Set `half_width` to always return `0.0` and run
   `cargo test snap::tests::outline_offset` — what do the strokes do now?
2. In `snap_drag`, run outline even when alignment matched. Which test
   fails, and which behaviour would a user notice?
3. Cap `ELLIPSE_MAX_ITERATIONS` at `3` and run
   `cargo test snap::tests::outline_ellipse` — the proptest that compares
   with 360 sampled points finds the error.

## Further reading

- [[ADR-T20-1 Outline snapping]]
- [[17 Snapping and alignment guides]] — the pipeline this extends
- David Eberly, *Distance from a Point to an Ellipse, an Ellipsoid, or a
  Hyperellipsoid* (Geometric Tools)
