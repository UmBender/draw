---
id: T04
title: Stroke smoothing
status: in-progress
wave: 2
branch: task/T04-smoothing
depends_on: [T01]
adrs: ["[[ADR-0007 Anti-tremor pipeline]]"]
feature: "[[Anti-tremor strokes]]"
tutorial: "[[04 Taming shaky input]]"
tags: [task]
---

# T04 Stroke smoothing

## Goal

The anti-tremor pipeline from [[ADR-0007 Anti-tremor pipeline]] as pure
functions plus a streaming `Smoother`.

## Spec

All items live in `core::smoothing`; tests are unit tests and proptests in
`smoothing::tests`. Floats are compared with `geom::approx_eq` only. Screen-pixel
tolerances are converted to world units by multiplying with `px_to_world`
(= `1 / zoom`), as [[ADR-0007 Anti-tremor pipeline]] requires.

- **AC-1** — `SmoothingLevel { Off, Low, Medium, High }` (`Copy`, `Eq`, `Hash`),
  `Default = Medium`, `next()` cycles `Off → Low → Medium → High → Off`,
  `label()` returns a lowercase name for the UI (`"off"`, `"low"`, `"medium"`,
  `"high"`), `params() -> SmoothingParams { min_dist_px, alpha, epsilon_px }` with
  the ADR-0007 table values (Off 0/1.0/0, Low 1.5/0.6/0.8, Medium 2.5/0.4/1.5,
  High 4.0/0.25/2.5).
  *Tests:* `level_default_is_medium`, `level_next_cycles_through_all`,
  `level_params_match_adr_table`, `level_labels_are_distinct`.
- **AC-2** — `Smoother::new(params, px_to_world: f32)`; `push(raw) -> bool` returns
  whether `raw` was kept. A point is kept only if its distance to the last *raw kept*
  point is ≥ `min_dist_px * px_to_world` **and** > 0 (exact duplicates are always
  dropped). Kept points are EMA-filtered: `s = s_prev + α (raw − s_prev)`; with
  `α ≥ 1` the raw point is stored exactly. The first point is kept unfiltered.
  Non-finite points are ignored (return `false`, change nothing).
  Bad construction input never panics: a non-finite or non-positive `px_to_world`
  is treated as `1.0`; `alpha` is clamped to `(0, 1]` (non-finite → `1.0`, ≤ 0 →
  smallest positive step); negative or non-finite `min_dist_px`/`epsilon_px` → `0`.
  *Tests:* `push_first_point_is_kept_unfiltered`, `push_drops_close_points`,
  `push_min_dist_scales_with_px_to_world`, `push_ignores_non_finite`,
  `push_drops_exact_duplicates`, `ema_reduces_jitter` (zig-zag input → smaller
  perpendicular variance), `new_sanitizes_bad_parameters`.
- **AC-3** — `points(&self) -> &[Vec2]` exposes the live smoothed polyline (for
  preview); it never contains the unfiltered trailing raw point.
  *Tests:* `points_exposes_live_smoothed_polyline`.
- **AC-4** — `finish(self) -> Vec<Vec2>`: appends the last finite raw point pushed
  (even if resampling dropped it) unless it equals the last smoothed point, then runs
  `simplify_rdp` with `epsilon_px * px_to_world`. No points pushed → empty; a single
  point → that point (a dot).
  *Tests:* `finish_ends_at_last_raw_point`, `finish_starts_at_first_raw_point`,
  `finish_empty_is_empty`, `finish_single_point_is_dot`,
  `finish_simplifies_straight_stroke_to_two_points`.
- **AC-5** — `simplify_rdp(points: &[Vec2], eps: f32) -> Vec<Vec2>`
  (Ramer–Douglas–Peucker): the result is a subsequence of the input that keeps the
  first and last point; every removed point is within `eps` of the result polyline;
  it never returns more points than the input; a straight line → 2 points. Inputs
  of ≤ 2 points, and `eps` that is NaN or ≤ 0, return the input unchanged.
  Iterative with an explicit stack (no recursion depth risk, 100 000-point input
  must not overflow the stack). Finite input is assumed for the error bound;
  non-finite points never panic.
  *Tests:* `rdp_straight_line_returns_two_points`, `rdp_keeps_corner_beyond_eps`,
  `rdp_drops_bump_within_eps`, `rdp_short_input_is_unchanged`,
  `rdp_non_positive_eps_is_identity`, `rdp_closed_loop_keeps_shape`,
  `rdp_huge_input_does_not_overflow_stack`, plus proptests `rdp_keeps_endpoints`,
  `rdp_error_bounded`, `rdp_idempotent`, `rdp_is_subsequence_and_never_grows`.
- **AC-6** — The `Off` level returns the input unchanged except for dropping exact
  duplicates (consecutive or later repeats of the last kept point).
  *Test:* `off_level_is_passthrough`, proptest `smoother_output_is_finite`.

## Out of scope

Curve fitting, pressure.

## Files owned

`src/core/smoothing.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 4 — requires step 1.

## Log
