---
title: Rendering with macroquad and culling
step: 7
requires: ["[[03 Cameras - world space vs screen space]]", "[[05 Modelling shapes and hit-testing]]"]
feature: "[[Shapes]]"
code: ["src/shell/render.rs"]
tags: [tutorial]
---

# Rendering with macroquad and culling

> **Learning path step 7.** Requires: [[03 Cameras - world space vs screen space]]
> (and [[05 Modelling shapes and hit-testing]]) ·
> Next: *12 App shell and toolbar* (calls the renderer every dirty frame)

## Why this matters here

The core knows *what* is on the canvas; the renderer turns it into triangles.
On an integrated GPU the expensive part is rarely the GPU — it is the CPU
work per frame: walking every shape, transforming points, building geometry
and, worst of all, allocating. A renderer for a scratch canvas must therefore
do three things well: **skip** what is off-screen, **not allocate**, and keep
**thin lines visible** at any zoom.

## The concept

**Immediate mode.** macroquad has no scene graph. Every frame you call
`draw_line`, `draw_triangle`, `draw_rectangle`… and macroquad batches the
vertices into a buffer it flushes at `next_frame()`. "Drawing a shape" is
just calling those functions with the right coordinates, in back-to-front
order.

**Where to transform.** You can either give macroquad a `Camera2D` and draw
in world units, or transform each point yourself and draw in pixels. draw
does the latter ([[ADR-T07-1 Screen-space tessellation in the renderer]])
because two things are defined in pixels anyway: the 1 px minimum outline
width ([[ADR-0013 World-space widths and zoom limits]]) and the constant-width
selection outline.

```text
  world width w ──·zoom──▶ w·zoom px ──max(1)──▶ screen_width
  world point p ──(p − offset)·zoom──▶ pixel
```

**Culling.** A shape whose bounding box does not touch the visible world
rectangle cannot produce a visible pixel, so it is skipped with four float
comparisons. The view is grown by a couple of pixels so outlines widened to
1 px at the edge are not clipped.

```text
   ┌──────────── cull_rect (view + 2 px) ───────────┐
   │  ┌──────── visible_world_rect ─────────┐       │
   │  │   ▭ drawn          ◯ drawn        │ ▭ drawn (touches)
   │  └──────────────────────────────────────┘       │
   └──────────────────────────────────────────────────┘     ▭ skipped
```

**Tessellating curves.** A circle of radius `r` drawn as an `n`-gon is off by
at most the *sagitta* `r (1 − cos(π/n))`. Asking for at most ¼ px gives the
smallest `n = ⌈π / acos(1 − ¼/r)⌉`: 8 segments for a 1 px joint, ~45 for a
100 px circle, capped at 256. The points come from rotating `(1, 0)` by a fixed
angle each step — one `sin_cos` per shape instead of one per vertex.

## How draw implements it

All in `src/shell/render.rs`:

- Pure helpers, unit-tested without a window: `screen_width`, `is_visible`,
  `cull_rect`, `to_mq`, `to_mq_color`, `stroke_needs_joints`,
  `circle_segments`, `selection_rect`.
- `draw_shapes(shapes, camera, viewport)` computes `cull_rect` once, then for
  each shape tests `is_visible(shape.bounds(), view)` and calls the private
  `draw_shape`. It takes `impl IntoIterator<Item = &Shape>`, so the document
  is iterated in place — nothing is collected.
- `draw_shape` matches on the variant:
  - **Stroke** — `points.windows(2)` → `draw_line`; when wider than 2 px a
    disc at every point gives round joints and caps (thin strokes skip them:
    the gaps are sub-pixel). A single point is a dot.
  - **Line / Arrow** — one segment with round caps when thick; the arrow
    shaft stops at the head base and the head is the filled
    `shape::arrow_head` triangle, the same one `Shape::bounds` and
    `Shape::hit` use.
  - **Rect** — optional `draw_rectangle` fill, then four opaque bands centred
    on the edges (square corners, outline straddles the geometry).
  - **Ellipse** — optional triangle-fan fill, then a ring of quads between
    the inner and outer ellipse, so a thick outline has no gaps.
- Discs and ellipses use `draw_triangle` only. macroquad's `draw_circle` and
  `draw_ellipse` build two `Vec`s per call and always use 20 sides; with a
  disc per stroke point that would be thousands of allocations a frame.
- `draw_preview` draws one shape without culling; `draw_selection` draws
  `selection_rect` as a 1 px outline in `THEME.accent`, padded 4 px on
  screen whatever the zoom.

## Try it

1. Set `JOINT_THRESHOLD_PX` to `0.0`, draw a long thin stroke in the app
   ([[T12 App shell and toolbar]]) and compare frame times.
2. Change `CHORD_TOLERANCE_PX` to `2.0`, run
   `cargo test circle_segments` and look at a big ellipse: when do the facets
   become visible?
3. Replace the culling in `draw_shapes` with `true`, zoom far in on a large
   drawing and watch the CPU usage.
4. Replace `draw_disc` with macroquad's `draw_circle` and count allocations
   with a heap profiler (e.g. `heaptrack`).

## Further reading

- [macroquad shapes module](https://docs.rs/macroquad/latest/macroquad/shapes/index.html).
- Sagitta — [Wikipedia](https://en.wikipedia.org/wiki/Sagitta_(geometry)).
- rust-skills rules `perf-iter-lazy`, `perf-profile-first`,
  `num-saturating-clamp`, `num-float-compare`, `test-proptest-properties`.
