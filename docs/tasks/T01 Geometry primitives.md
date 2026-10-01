---
id: T01
title: Geometry primitives
status: in-progress
wave: 1
branch: task/T01-geometry
depends_on: [T00]
adrs: ["[[ADR-0003 Headless core and thin shell]]"]
feature:
tutorial: "[[01 2D vectors, AABBs and point-segment distance]]"
tags: [task]
---

# T01 Geometry primitives

## Goal

Dependency-free 2D math used by every other core module.

## Spec

All items live in `core::geom`; tests are unit tests in `geom::tests`.

- **AC-1** — `Vec2 { x: f32, y: f32 }` derives `Debug, Clone, Copy, PartialEq, Default`,
  with `new`, `ZERO`, operators `+ - * (f32) / (f32)`, unary `-`, `+=`, `-=`,
  `length`, `length_sq`, `dot`, `distance`, `lerp(self, other, t)` (`t = 0 → self`,
  `t = 1 → other`, not clamped) and `approx_eq(self, other, eps)` (component-wise).
  *Tests:* `vec2_new_and_zero_set_components`, `vec2_add_sub_are_component_wise`,
  `vec2_mul_div_by_scalar_scale_components`, `vec2_neg_and_assign_ops_match_binary_ops`,
  `vec2_length_of_3_4_is_5`, `vec2_length_sq_avoids_sqrt`, `vec2_dot_of_perpendicular_is_zero`,
  `vec2_distance_is_symmetric`, `vec2_lerp_hits_endpoints_and_midpoint`,
  `vec2_approx_eq_uses_per_component_tolerance`.
- **AC-2** — `Vec2::is_finite` (both components finite) and
  `Vec2::sanitize(self) -> Option<Vec2>` (`None` if any component is NaN/±∞, else `Some(self)`).
  *Tests:* `vec2_is_finite_detects_nan_and_inf`, `vec2_sanitize_rejects_nan_and_inf`,
  `vec2_sanitize_keeps_finite_values`.
- **AC-3** — `Aabb { min, max }` with invariant `min <= max` component-wise:
  - `from_points(&[Vec2]) -> Option<Aabb>` — tightest box; non-finite points are
    skipped; `None` if no finite point remains (including empty input).
  - `from_corners(a, b)` — any two opposite corners, any order.
  - `expand(margin)` — grows every side by `margin`; a negative margin shrinks,
    collapsing to the centre instead of inverting.
  - `contains(p)` — inclusive of the boundary.
  - `intersects(&other)` — inclusive (touching boxes intersect).
  - `union(&other)`, `center`, `width`, `height`, `translate(delta)`.
  *Tests:* `aabb_from_points_empty_is_none`, `aabb_from_points_spans_all_points`,
  `aabb_from_points_skips_non_finite`, `aabb_from_corners_is_order_independent`,
  `aabb_expand_grows_each_side`, `aabb_expand_negative_collapses_to_center`,
  `aabb_contains_is_inclusive`, `aabb_intersects_overlapping_and_touching`,
  `aabb_intersects_disjoint_is_false`, `aabb_union_covers_both`,
  `aabb_center_width_height`, `aabb_translate_moves_both_corners`.
- **AC-4** — `distance_to_segment(p, a, b) -> f32` is the Euclidean distance from
  `p` to the closed segment `ab`: correct for interior projection, beyond either
  endpoint, and degenerate `a == b` (distance to the point; no division by zero).
  *Tests:* `segment_distance_interior_projection_is_perpendicular`,
  `segment_distance_before_start_is_distance_to_a`,
  `segment_distance_after_end_is_distance_to_b`,
  `segment_distance_degenerate_segment_is_point_distance`,
  `segment_distance_point_on_segment_is_zero`, and proptests
  `segment_distance_never_exceeds_endpoint_distance`,
  `segment_distance_is_non_negative_and_finite`.
- **AC-5** — `approx_eq(a, b, eps) -> bool` is `|a − b| <= eps`; any NaN/∞ operand
  yields `false`. No `==` on floats anywhere (clippy `float_cmp` deny).
  *Tests:* `approx_eq_within_eps_is_true`, `approx_eq_outside_eps_is_false`,
  `approx_eq_nan_or_inf_is_false`.

All pure functions are `#[must_use]`; nothing in this module panics.

## Out of scope

Transforms (camera owns them), curves.

## Files owned

`src/core/geom.rs`

## Subtasks (one commit each)

- [ ] spec — `docs(T01): specify geometry acceptance criteria`
- [ ] tests — `test(T01): add failing geometry tests`
- [ ] models — `feat(T01): add Vec2 and Aabb types`
- [ ] behaviour — `feat(T01): implement vector, AABB and segment math`
- [ ] quality — `chore(T01): pass clippy and rustfmt`
- [ ] docs — `docs(T01): add geometry tutorial`

## Learning path

Step 1 — requires step 0.

## Log
