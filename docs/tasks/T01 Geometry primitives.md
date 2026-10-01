---
id: T01
title: Geometry primitives
status: review
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

- [x] spec — `docs(T01): specify geometry acceptance criteria` (7cacb17)
- [x] tests — `test(T01): add failing geometry tests` (138e714)
- [x] models — `feat(T01): add Vec2 and Aabb types` (05d03b9)
- [x] behaviour — `feat(T01): implement vector, AABB and segment math` (3c204ef)
- [x] quality — `refactor(T01): share component-wise min/max in Aabb constructors` (b074ee0)
- [x] docs — `docs(T01): add geometry tutorial`

## Learning path

Step 1 — requires step 0. Tutorial: [[01 2D vectors, AABBs and point-segment distance]].

## Log

- Spec refinements (no change of intent): `Vec2` also derives `PartialEq`,
  `Default` and gets unary `-`, `+=`, `-=` and `Vec2::approx_eq`;
  `Aabb::from_points` skips non-finite points; `expand` with a negative margin
  collapses an axis to its centre instead of inverting; `contains` and
  `intersects` are inclusive; `approx_eq` is `false` for any NaN/∞ operand.
- Red phase confirmed: tests commit failed to compile with 56 unresolved-name
  errors (`Vec2`, `Aabb`, `approx_eq`, `distance_to_segment`); models commit
  compiled with all 35 tests failing on `todo!()`.
- Behaviour commit: 35/35 tests green, including 2 proptests over
  coordinates in `[-1000, 1000]` (tolerance `1e-2` for f32 rounding).
- Quality: `scripts/check.sh` was already green after the behaviour step; the
  quality commit is a no-behaviour-change refactor (private component-wise
  min/max helpers, `then_some` in `sanitize`). Green again afterwards.
- Degenerate segment test uses `len_sq <= 0.0` (ordering comparison, allowed
  by `float_cmp`); non-finite `p`, `a` or `b` yield a non-finite distance
  rather than a panic — callers sanitize input first.
- No new ADRs; no files outside *Files owned* touched (besides this note and
  the tutorial).
