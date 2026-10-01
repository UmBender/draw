---
id: T07
title: Renderer
status: review
wave: 3
branch: task/T07-renderer
depends_on: [T03, T05]
adrs: ["[[ADR-0002 Rust and macroquad]]", "[[ADR-0006 Redraw on demand]]", "[[ADR-0013 World-space widths and zoom limits]]", "[[ADR-T07-1 Screen-space tessellation in the renderer]]"]
feature: "[[Shapes]]"
tutorial: "[[07 Rendering with macroquad and culling]]"
tags: [task]
---

# T07 Renderer

## Goal

Draw shapes, a preview shape and a selection outline through the camera with
macroquad, skipping anything off-screen.

## Spec

Everything lives in `shell::render` and draws in **screen space** with
macroquad's default camera; world points go through `Camera::world_to_screen`
([[ADR-T07-1 Screen-space tessellation in the renderer]]). Pure helpers are
unit-tested in `render::tests` (`use super::*;`, proptest for properties,
floats compared with `geom::approx_eq`) and need no window. Nothing panics on
any input, including NaN/±∞.

Public constants: `MIN_SCREEN_WIDTH_PX = 1.0`, `JOINT_THRESHOLD_PX = 2.0`,
`CULL_MARGIN_PX = 2.0`, `CHORD_TOLERANCE_PX = 0.25`, `MIN_SEGMENTS = 8`,
`MAX_SEGMENTS = 256`, `SELECTION_WIDTH_PX = 1.0`, `SELECTION_PAD_PX = 4.0`.

Pure helpers:

- **AC-1** — `screen_width(world_width, zoom) -> f32` = `world_width * zoom`,
  at least `MIN_SCREEN_WIDTH_PX`; a NaN, infinite or negative product gives
  `MIN_SCREEN_WIDTH_PX` ([[ADR-0013 World-space widths and zoom limits]]).
  *Tests:* `screen_width_scales_with_zoom`, `screen_width_clamps_to_one_pixel`,
  `screen_width_non_finite_or_negative_is_one_pixel`, proptest
  `screen_width_is_at_least_one_pixel_and_finite`.
- **AC-2** — `is_visible(bounds: Aabb, view: Aabb) -> bool`: the boxes overlap
  or touch; any NaN coordinate gives `false`. `cull_rect(camera, viewport) -> Aabb`
  is `camera.visible_world_rect(viewport)` grown by `CULL_MARGIN_PX` converted to
  world units (covers the 1 px minimum width overhang).
  *Tests:* `is_visible_overlapping_is_true`, `is_visible_touching_edge_is_true`,
  `is_visible_disjoint_is_false`, `is_visible_bounds_containing_view_is_true`,
  `is_visible_nan_bounds_is_false`, `cull_rect_is_view_grown_by_margin`.
- **AC-3** — `to_mq(Vec2) -> macroquad::math::Vec2` keeps coordinates;
  `to_mq_color(Rgba) -> macroquad::color::Color` scales channels to `0..=1`.
  *Tests:* `conversion_vec2_keeps_coordinates`, `conversion_color_scales_channels`,
  `conversion_palette_ink_is_opaque`.
- **AC-4** — `stroke_needs_joints(width_px) -> bool` is `width_px > JOINT_THRESHOLD_PX`
  (NaN → `false`), so thin strokes skip round joints. *Test:* `joints_threshold`.
- **AC-8** — `circle_segments(radius_px) -> u16`: the smallest segment count whose
  chord error `r (1 − cos(π/n))` is ≤ `CHORD_TOLERANCE_PX`, clamped to
  `MIN_SEGMENTS..=MAX_SEGMENTS`; non-finite or non-positive radius gives
  `MIN_SEGMENTS`. *Tests:* `circle_segments_small_radius_is_minimum`,
  `circle_segments_grows_with_radius`, `circle_segments_huge_radius_is_capped`,
  `circle_segments_non_finite_is_minimum`, proptest
  `circle_segments_chord_error_within_tolerance`.
- **AC-9** — `selection_rect(bounds, camera) -> Aabb`: `bounds` mapped to screen
  pixels and grown by `SELECTION_PAD_PX` (padding is constant on screen, not
  scaled by zoom). *Tests:* `selection_rect_pads_screen_bounds`,
  `selection_rect_padding_is_independent_of_zoom`.

Drawing (needs a window; verified by hand in [[T12 App shell and toolbar]] /
[[T14 Performance and release validation]]):

- **AC-5** — `draw_shapes<'a>(shapes: impl IntoIterator<Item = &'a Shape>, camera: &Camera, viewport: Vec2)`
  skips shapes whose `bounds()` are not `is_visible` in `cull_rect`, and draws
  every variant in palette colour with `screen_width`: strokes as segments plus
  round joints when `stroke_needs_joints` (a single-point stroke is a dot);
  lines (round caps when thick); arrows as a shaft to the head base plus the
  filled `shape::arrow_head` triangle; rect/ellipse fill first, outline on top
  (rect outline = four bands centred on the edges, ellipse outline = ring of
  `circle_segments` quads). Non-finite shapes are skipped.
- **AC-6** — `draw_preview(shape, camera)` draws one shape like `draw_shapes`
  (no culling); `draw_selection(bounds, camera)` draws `selection_rect` as an
  outline `SELECTION_WIDTH_PX` wide in `THEME.accent`.
- **AC-7** — No allocation per frame in our code: shapes are iterated, never
  collected; discs and ellipses are tessellated with `draw_triangle`/`draw_line`
  (macroquad's `draw_circle`/`draw_ellipse` allocate). Verified by review: no
  `Vec`, `collect`, `format!` or `to_owned` in `render.rs` outside tests.

## Out of scope

Toolbar, window, input.

## Files owned

`src/shell/render.rs`

## Subtasks (one commit each)

- [x] spec — `docs(T07): specify renderer acceptance criteria` (db16369)
- [x] tests — `test(T07): add failing tests for renderer helpers` (a5f764e)
- [x] models — `feat(T07): add renderer constants and helper signatures` (af3568e)
- [x] behaviour — `feat(T07): implement culling, widths and shape tessellation` (f581209)
- [x] quality — `chore(T07): pass clippy and rustfmt` (e0ba821)
- [x] docs — `docs(T07): add feature note and tutorial`

## Learning path

Step 7 — requires step 3 (and 5). Tutorial:
[[07 Rendering with macroquad and culling]].

## Log

- Red phase confirmed: tests commit failed to compile (51 errors, all E0425:
  no `screen_width`, `is_visible`, `cull_rect`, `to_mq`, `to_mq_color`,
  `stroke_needs_joints`, `circle_segments`, `selection_rect` or constants);
  models commit built and all 21 renderer tests failed on `todo!()`;
  behaviour commit turned them green (one fix during the step: an infinite
  radius in `circle_segments` returns `MIN_SEGMENTS`, as specified).
  `PROPTEST_CASES=20000` run of the renderer proptests passes.
- Added beyond the original spec (stated in the refined *Spec*): pure
  helpers `cull_rect` (AC-2), `circle_segments` (AC-8) and `selection_rect`
  (AC-9) so culling margin, tessellation and the selection outline are
  testable without a window; the public constants listed in *Spec*.
- `draw_shapes` takes `impl IntoIterator<Item = &Shape>` instead of
  `impl Iterator` (accepts `&Vec<Shape>`, slices and iterators alike); camera
  is passed as `&Camera`, viewport as a pixel `Vec2`.
- Selection outline is solid `THEME.accent` (not dashed), 1 px, padded 4 px.
- AC-5/AC-6 drawing is not unit-tested (needs a window); to be checked by
  hand in [[T12 App shell and toolbar]] / [[T14 Performance and release validation]].
  AC-7 verified by review: no `Vec`, `collect`, `format!` or `to_owned`
  outside tests; macroquad's own allocating `draw_circle`/`draw_ellipse` are
  not used.
- New ADR: [[ADR-T07-1 Screen-space tessellation in the renderer]].
- Quality: clippy pedantic `many_single_char_names` in `to_mq_color`;
  rustfmt reflowed tests.
- Integrator: [[Decision Log]] needs ADR-T07-1; [[Learning Path]] row 7 and
  [[Feature Index]] need [[07 Rendering with macroquad and culling]].
