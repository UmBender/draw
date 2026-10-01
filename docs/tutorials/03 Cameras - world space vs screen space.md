---
title: Cameras - world space vs screen space
step: 3
requires: ["[[01 2D vectors, AABBs and point-segment distance]]"]
feature: "[[Infinite canvas, pan and zoom]]"
code: ["src/core/camera.rs"]
tags: [tutorial]
---

# Cameras - world space vs screen space

> **Learning path step 3.** Requires: [[01 2D vectors, AABBs and point-segment distance]] ·
> Next: *7 Rendering with macroquad and culling* (draws through the camera),
> *8 An editor as an input-driven state machine* (wires wheel and drags to it)

## Why this matters here

An "infinite canvas" is just a window onto a coordinate system that never
ends. Shapes are stored in **world space** and never move when you pan or zoom;
only the *camera* changes. The mouse, on the other hand, reports **screen
space** pixels. Every tool therefore starts with "convert the cursor to world
space", and the renderer ends with "convert world points to pixels". Getting
this one transform right — and keeping it invertible — makes the rest of the
editor simple.

## The concept

Two numbers describe the view: `offset`, the world point that appears at the
screen's top-left corner, and `zoom`, how many pixels one world unit covers.

```text
            world space                         screen space (pixels)
   ┌──────────────────────────┐
   │      offset ●──────┐     │                 (0,0)●──────────┐
   │             │ view │     │   (w−offset)·z       │  window  │
   │             └──────┘     │  ─────────────▶      └──────────┘
   └──────────────────────────┘
      screen = (world − offset) · zoom
      world  = screen / zoom + offset
```

The two formulas undo each other, so `screen_to_world(world_to_screen(p)) ≈ p`.

**Pan.** Dragging the mouse by `Δ` pixels should drag the drawing by `Δ`
pixels. A world point `w` is drawn at `(w − offset)·zoom`; to add `Δ` to that,
`offset` must shrink by `Δ / zoom`. Note the division: at zoom 4 a 40 px drag
moves the camera only 10 world units.

**Zoom at the cursor.** Naively multiplying `zoom` would zoom around the
top-left corner and the content would slide away from the mouse. Instead:

1. remember the world point under the cursor `c`: `a = c / zoom + offset`;
2. change zoom: `zoom' = clamp(zoom · 1.15^notches)`;
3. solve `c / zoom' + offset' = a` for the new offset: `offset' = a − c / zoom'`.

Because step 3 uses the *clamped* zoom, the anchor stays put even when you hit
the zoom limit. Using `1.15^notches` (not `+0.15`) makes zoom multiplicative:
one notch in then one notch out returns exactly where you started, and the
"speed" feels the same at 5 % and at 2000 %.

**Lengths.** Distances transform without the offset: `screen_len = world · zoom`,
`world_len = px / zoom`. That is how a "6 px click tolerance" becomes a world
tolerance for hit-testing, and how new strokes get `width = width_px / zoom`
([[ADR-0013 World-space widths and zoom limits]]).

## How draw implements it

Everything is in `src/core/camera.rs`:

- `Camera { offset, zoom }` with **private** fields. The invariant (finite
  offset, zoom in `[ZOOM_MIN, ZOOM_MAX]`) is enforced by `Camera::new`, which
  clamps the zoom and replaces non-finite values, and by every mutator. Callers
  read the state through `offset()` and `zoom()`. This is the
  "make invalid states unrepresentable" idea from rust-skills `api-newtype-safety`.
- `world_to_screen` / `screen_to_world` are one-liners on top of the `Vec2`
  operators from step 1.
- `pan_by_screen` and `zoom_at` first `sanitize()` their inputs (NaN/∞ → return
  early), then compute a candidate offset and store it through the private
  `set_offset`, which only accepts a finite value. So a NaN from a broken input
  device, or a pan that would overflow `f32`, simply does nothing.
- `zoom_at` relies on `f32::clamp` handling `+∞` and `0.0` (from `powf`
  overflow/underflow) — both land on a limit.
- `fit` computes a per-axis zoom `usable_px / extent` with the helper
  `axis_zoom`, which returns `None` for a zero extent (a horizontal line does
  not constrain vertical zoom, a single point keeps the current zoom), takes the
  smaller one, clamps it, and centres `bounds.center()` in the viewport.
- `visible_world_rect` maps the two viewport corners to world space; the
  renderer uses it to skip shapes whose `Aabb` does not intersect it.

The tests mirror the spec. `assert_camera_unchanged` compares with tolerance
`0.0` via `geom::approx_eq` — exact equality without the forbidden float `==`.
Three proptests cover the round trip, the zoom anchor for random notches, and a
random sequence of pans and zooms that must keep zoom in range and offset
finite. Their tolerances are *relative*: a few ulps of the largest number
involved, because `f32` error grows with magnitude.

## Try it

1. In `zoom_at`, compute the new offset with the *unclamped* zoom. Run
   `cargo test zoom_clamped` — the anchor drifts once the limit is hit.
2. Replace `ZOOM_STEP.powf(notches)` with `1.0 + 0.15 * notches`. Which tests
   fail? Try `notches = -10` by hand: what zoom would you get?
3. Remove the `sanitize()` in `pan_by_screen` and run
   `cargo test non_finite`. The camera becomes NaN forever — every later
   transform returns NaN too.
4. Shrink the proptest tolerance to `f32::EPSILON` (absolute) and see which
   coordinates fail. Why do large offsets need a larger tolerance?

## Further reading

- *Fundamentals of Computer Graphics* (Marschner & Shirley), ch. 7 — viewing
  transforms.
- [Zoom to mouse position](https://stackoverflow.com/questions/2916081/zoom-in-on-a-point-using-scale-and-translate)
  — the same derivation for canvas APIs.
- rust-skills rules `num-float-compare`, `api-newtype-safety`, `api-must-use`,
  `test-proptest-properties`, `test-arrange-act-assert`.
