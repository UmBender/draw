---
id: T03
title: Camera
status: todo
wave: 2
branch: task/T03-camera
depends_on: [T01]
adrs: ["[[ADR-0013 World-space widths and zoom limits]]"]
feature: "[[Infinite canvas, pan and zoom]]"
tutorial: "[[03 Cameras - world space vs screen space]]"
tags: [task]
---

# T03 Camera

## Goal

The transform that makes the canvas infinite: pan, zoom anchored at the cursor,
conversion between world and screen coordinates.

## Spec

- **AC-1** — `Camera { offset: Vec2 (world point at screen origin), zoom: f32 }`,
  `Default` = offset 0, zoom 1. Constants `ZOOM_MIN = 0.05`, `ZOOM_MAX = 20.0`,
  `ZOOM_STEP = 1.15`. *Test:* `camera::tests::default_is_identity`.
- **AC-2** — `world_to_screen` / `screen_to_world` are inverses. *Test:* proptest
  `round_trip_within_tolerance` (finite inputs, any valid zoom).
- **AC-3** — `pan_by_screen(delta)` moves content by exactly `delta` on screen.
  *Test:* `pan_moves_content_by_screen_delta`.
- **AC-4** — `zoom_at(screen_point, notches: f32)` keeps the world point under
  `screen_point` fixed and clamps zoom to the range. *Tests:* `zoom_keeps_anchor_fixed`,
  `zoom_clamped_to_range`.
- **AC-5** — Non-finite inputs to any method leave the camera unchanged.
  *Test:* `non_finite_input_is_ignored`.
- **AC-6** — `visible_world_rect(viewport: Vec2) -> Aabb`, `world_len(px)` /
  `screen_len(world)` for tolerance conversion. *Tests:* `visible_rect_*`, `len_conversion_*`.
- **AC-7** — `fit(bounds: Aabb, viewport: Vec2, margin_px)` centres and zooms to show
  bounds (clamped). `reset()` returns to default. *Tests:* `fit_*`, `reset_*`.

## Out of scope

Animation/inertia, rotation.

## Files owned

`src/core/camera.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 3 — requires step 1.

## Log
