---
id: T04
title: Stroke smoothing
status: ready
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

- **AC-1** — `SmoothingLevel { Off, Low, Medium, High }`, `Default = Medium`,
  `next()` cycles, `params() -> SmoothingParams { min_dist_px, alpha, epsilon_px }`
  with the table values. *Tests:* `smoothing::tests::level_*`.
- **AC-2** — `Smoother::new(params, px_to_world: f32)`; `push(raw) -> bool` adds a point
  only if ≥ `min_dist` from the last *raw kept* point; kept points are EMA-filtered.
  First point is kept unfiltered. Non-finite points are ignored.
  *Tests:* `push_drops_close_points`, `push_ignores_non_finite`, `ema_reduces_jitter`
  (zig-zag input → smaller perpendicular variance).
- **AC-3** — `points()` exposes the live smoothed polyline (for preview).
- **AC-4** — `finish(self) -> Vec<Vec2>`: appends the last raw point, then runs RDP
  with `epsilon_px * px_to_world`. *Tests:* `finish_ends_at_last_raw_point`.
- **AC-5** — `simplify_rdp(&[Vec2], eps) -> Vec<Vec2>`: keeps first and last; every
  removed point is within `eps` of the result; never returns more points than input;
  straight line → 2 points. Iterative (no recursion depth risk). *Tests:* `rdp_*` +
  proptests `rdp_keeps_endpoints`, `rdp_error_bounded`, `rdp_idempotent`.
- **AC-6** — `Off` level returns input unchanged (except dropping exact duplicates).
  *Test:* `off_level_is_passthrough`.

## Out of scope

Curve fitting, pressure.

## Files owned

`src/core/smoothing.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 4 — requires step 1.

## Log
