---
id: T07
title: Renderer
status: ready
wave: 3
branch: task/T07-renderer
depends_on: [T03, T05]
adrs: ["[[ADR-0002 Rust and macroquad]]", "[[ADR-0013 World-space widths and zoom limits]]"]
feature: "[[Shapes]]"
tutorial: "[[07 Rendering with macroquad and culling]]"
tags: [task]
---

# T07 Renderer

## Goal

Draw shapes, a preview shape and a selection outline through the camera with
macroquad, skipping anything off-screen.

## Spec

Pure helpers (unit-tested, no window):

- **AC-1** — `screen_width(world_width, zoom) -> f32` ≥ 1.0 px. *Test:* `render::tests::screen_width_*`.
- **AC-2** — `is_visible(bounds: Aabb, view: Aabb) -> bool`. *Test:* `is_visible_*`.
- **AC-3** — `to_mq(Vec2) -> macroquad::math::Vec2` and `to_mq_color(Rgba)`. *Test:* `conversion_*`.
- **AC-4** — `stroke_needs_joints(width_px) -> bool` (round joints only when > 2 px,
  keeps thin strokes cheap). *Test:* `joints_threshold`.

Drawing (verified by hand in [[T12 App shell and toolbar]] / [[T14 Performance and release validation]]):

- **AC-5** — `draw_shapes<'a>(shapes: impl Iterator<Item = &'a Shape>, camera, viewport)`
  draws every variant (strokes as segments + round joints, arrow head filled, fill under outline).
- **AC-6** — `draw_preview(shape, camera)` and `draw_selection(bounds, camera)`
  (dashed or accent-coloured outline, constant screen width).
- **AC-7** — No allocation per frame in the hot path (iterate, don't collect).

## Out of scope

Toolbar, window, input.

## Files owned

`src/shell/render.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 7 — requires step 3 (and 5).

## Log
