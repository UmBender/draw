---
id: T05
title: Shape model
status: in-progress
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

All items live in `core::shape`; tests are unit tests in `shape::tests`
(`use super::*;`, proptest for properties). Floats are compared with
`geom::approx_eq`. Nothing in this module panics, for any input including
NaN/±∞; non-finite query points never hit and are never contained.

- **AC-1** — `Style { color: ColorId, width: f32 }` (`Debug, Clone, Copy, PartialEq`)
  and `Shape` enum (`Debug, Clone, PartialEq`) with public fields:
  `Stroke { points: Vec<Vec2>, style }`, `Line { a, b, style }`, `Arrow { a, b, style }`
  (head at `b`), `Rect { a, b, style, fill: Option<ColorId> }`,
  `Ellipse { a, b, style, fill: Option<ColorId> }` (`a`, `b` = opposite corners of the
  bounding box, any order). Accessors `style() -> Style`, `fill() -> Option<ColorId>`
  (`None` for open shapes), `is_closed()` (rect, ellipse).
  *Tests:* `construct_stroke_keeps_points_and_style`, `construct_line_and_arrow_keep_endpoints`,
  `construct_rect_and_ellipse_keep_corners_and_fill`, `accessors_report_style_fill_and_closedness`.
- **AC-2** — `bounds() -> Aabb` is the geometry box grown by `width / 2`
  (negative/NaN width counts as 0). Stroke: box of its finite points; an empty stroke
  (or one with no finite point) gives the zero-size box at the origin. Arrow: also
  covers the three `arrow_head` points. Rect/ellipse: box of `a`, `b`.
  *Tests:* `bounds_stroke_includes_half_width`, `bounds_empty_stroke_is_point_at_origin`,
  `bounds_line_includes_half_width`, `bounds_arrow_includes_head`,
  `bounds_rect_is_corner_order_independent`, `bounds_ellipse_matches_bounding_box`.
- **AC-3** — `hit(p, tol) -> bool`: true if `p` is within `reach = tol + width/2` of the
  outline. Stroke: any segment; single-point stroke: the point; empty stroke: never.
  Line: the segment. Arrow: the shaft, the head edges or inside the head triangle.
  Rect: any of the four edges. Ellipse: the outline, using the gradient-normalised
  distance `k0 (k0 − 1) / k1` (exact on the axes and on the outline; a degenerate
  ellipse with a zero radius is treated as its segment). Filled rect/ellipse also hit
  wherever `contains(p)`. AABB rejection first.
  *Tests:* `hit_stroke_near_segment_is_true`, `hit_stroke_far_is_false`,
  `hit_single_point_stroke_uses_distance_to_point`, `hit_empty_stroke_is_false`,
  `hit_line_edge_within_tolerance_plus_half_width`, `hit_arrow_shaft_and_head`,
  `hit_rect_outline_inside_outside`, `hit_filled_rect_interior_is_true`,
  `hit_ellipse_outline_inside_outside`, `hit_filled_ellipse_interior_is_true`,
  `hit_degenerate_ellipse_is_segment`, `hit_non_finite_point_is_false`.
- **AC-4** — `contains(p) -> bool`: rect interior (boundary inclusive), ellipse
  interior `((x−cx)/rx)² + ((y−cy)/ry)² <= 1` (false if a radius is 0); stroke, line,
  arrow always false. Ignores fill. *Tests:* `contains_rect_inside_and_boundary`,
  `contains_rect_outside_is_false`, `contains_ellipse_inside_and_outside`,
  `contains_degenerate_ellipse_is_false`, `contains_open_shapes_is_false`.
- **AC-5** — `translate(&mut self, delta)` moves every point (style and fill kept);
  `with_fill(self, Option<ColorId>) -> Shape` sets the fill of rect/ellipse and returns
  open shapes unchanged. *Tests:* `translate_moves_every_point`,
  `with_fill_sets_and_clears_fill_on_closed_shapes`, `with_fill_on_open_shape_is_noop`,
  proptest `translate_moves_bounds_by_delta`.
- **AC-6** — `arrow_head(a, b, width) -> [Vec2; 3]` = `[tip, left, right]`, tip at `b`,
  head length `max(ARROW_HEAD_MIN_LENGTH, ARROW_HEAD_LENGTH_PER_WIDTH * width)` along
  `b → a`, base half-width `ARROW_HEAD_HALF_WIDTH_RATIO * length`; degenerate `a ≈ b`
  gives `[b, b, b]`. Shared by renderer ([[T07 Renderer]]) and `bounds`.
  *Tests:* `arrow_head_tip_is_at_b`, `arrow_head_length_scales_with_width`,
  `arrow_head_has_min_size`, `arrow_head_is_symmetric_about_shaft`,
  `arrow_head_degenerate_collapses_to_tip`.
- **AC-7** — `is_finite()` true iff every coordinate and the width are finite
  (an empty stroke with finite width is finite). *Tests:* `is_finite_true_for_finite_shapes`,
  `is_finite_false_for_nan_or_inf_coordinate`, `is_finite_false_for_non_finite_width`.

## Out of scope

Rendering, storage, ids.

## Files owned

`src/core/shape.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 5 — requires step 1 (and step 2 for colours).

## Log
