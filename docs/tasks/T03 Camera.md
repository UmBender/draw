---
id: T03
title: Camera
status: in-progress
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

All items live in `core::camera`; tests are unit tests in `camera::tests`.
Transform: `screen = (world − offset) · zoom`, `world = screen / zoom + offset`.
Screen space has its origin at the top-left of the viewport, in pixels.

- **AC-1** — `Camera { offset: Vec2 (world point at screen origin), zoom: f32 }`
  derives `Debug, Clone, Copy, PartialEq`. Fields are private so the invariant
  (finite `offset`, `zoom` finite and in `[ZOOM_MIN, ZOOM_MAX]`) always holds;
  read them with `offset()` / `zoom()`. `Camera::new(offset, zoom)` clamps zoom to
  the range and replaces a non-finite offset by `Vec2::ZERO` / a non-finite zoom
  by `1.0`. `Default` = offset 0, zoom 1. Constants `ZOOM_MIN = 0.05`,
  `ZOOM_MAX = 20.0`, `ZOOM_STEP = 1.15` (from [[ADR-0013 World-space widths and zoom limits]]).
  *Tests:* `camera::tests::default_is_identity`, `new_clamps_zoom_and_rejects_non_finite`.
- **AC-2** — `world_to_screen` / `screen_to_world` are inverses (within f32
  rounding, tolerance relative to the magnitudes involved). Non-finite points
  propagate to a non-finite result (no panic). *Tests:* `transform_examples_match_formula`,
  proptest `round_trip_within_tolerance` (coordinates in `[-1e4, 1e4]`, any valid zoom).
- **AC-3** — `pan_by_screen(delta)` moves content by exactly `delta` on screen
  (`offset −= delta / zoom`). *Test:* `pan_moves_content_by_screen_delta`.
- **AC-4** — `zoom_at(screen_point, notches: f32)` multiplies zoom by
  `ZOOM_STEP^notches` (positive = zoom in), clamps it to the range, and keeps the
  world point under `screen_point` fixed — also when clamped.
  *Tests:* `zoom_one_notch_multiplies_by_step`, `zoom_keeps_anchor_fixed`,
  `zoom_clamped_to_range`, proptests `zoom_keeps_anchor_fixed_for_any_input`,
  `zoom_stays_in_range_for_any_sequence`.
- **AC-5** — Non-finite inputs to any mutating method (`pan_by_screen`, `zoom_at`,
  `fit`) leave the camera unchanged; so does a finite input whose result would
  overflow to a non-finite offset. *Tests:* `non_finite_input_is_ignored`,
  `overflowing_pan_is_ignored`.
- **AC-6** — `visible_world_rect(viewport: Vec2) -> Aabb` is the world box seen
  through a viewport of `viewport` pixels (a non-finite viewport counts as
  zero-sized). `world_len(px) = px / zoom` and `screen_len(world) = world · zoom`
  convert tolerances and widths. *Tests:* `visible_rect_default_matches_viewport`,
  `visible_rect_follows_offset_and_zoom`, `visible_rect_non_finite_viewport_is_degenerate`,
  `len_conversion_scales_with_zoom`, `len_conversion_round_trip`.
- **AC-7** — `fit(bounds: Aabb, viewport: Vec2, margin_px: f32)` centres `bounds`
  in the viewport and picks the largest zoom (clamped) at which `bounds` plus
  `margin_px` on every side fits. A degenerate axis (zero extent) is ignored;
  if both are degenerate, zoom is kept and the camera only centres. A negative
  margin counts as 0, the usable area never drops below 1 px per axis, and a
  viewport with a non-positive or non-finite component is ignored.
  `reset()` returns to `Camera::default()`. *Tests:* `fit_centres_bounds`,
  `fit_zooms_to_show_bounds_with_margin`, `fit_clamps_zoom`,
  `fit_degenerate_bounds_keeps_zoom`, `fit_invalid_viewport_is_ignored`,
  `reset_returns_default`.

All queries are `#[must_use]`; nothing in this module panics. Floats are
compared with `geom::approx_eq`, never `==`.

## Out of scope

Animation/inertia, rotation.

## Files owned

`src/core/camera.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 3 — requires step 1.

## Log
