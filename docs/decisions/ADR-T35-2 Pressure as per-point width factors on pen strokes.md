---
id: ADR-T35-2
title: Pressure as per-point width factors on pen strokes
status: proposed
kind: decision
date: 2026-10-20
task: T35
builds_on: ["[[ADR-0007 Anti-tremor pipeline]]", "[[ADR-0013 World-space widths and zoom limits]]", "[[ADR-T35-1 Vendored miniquad with a pen tablet patch]]"]
amends: ["[[ADR-T24-4 Strokes as one strip with sparse round joins]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T35-2 Pressure as per-point width factors on pen strokes

## Context

With pressure reaching the core ([[ADR-T35-1 Vendored miniquad with a pen tablet patch]]),
a pen stroke should get thicker where the pen presses harder. The owner
decided (2026-10-20): a normal press draws today's width, only the Pen
tool uses pressure, and a stroke keeps a width per point so it can taper.

## Decision

1. **Curve.** `core::pressure::width_factor(p) = clamp(2p, 0.25, 2.0)`.
   A half press (`p = 0.5`) is the chosen width, a light touch goes down to
   ¼ of it, and a hard press up to 2×. Non-finite pressure gives `1`.
2. **Model.** `Shape::Stroke` gets `widths: Option<Vec<f32>>`: one factor
   per point, multiplying `style.width`. It is `None` for strokes drawn
   without pressure (mouse, or a pen with no pressure data). A stroke is
   valid (`is_finite`) only when `widths` is `None`, or has one finite,
   positive factor per point. Bounds and hit-testing use the largest
   factor.
3. **Input.** `InputEvent::Pressure { pressure: Option<f32> }` sets the
   pressure of the following pointer events. The tap filter passes it
   through like a key. The editor keeps it clamped to `[0, 1]`, and
   non-finite values clear it. Every tool gets it in `Pointer::pressure`,
   and only the Pen tool reads it.
4. **Pen tool.** A stroke whose press has a pressure records a factor per
   point. A later point without a pressure repeats the last factor. A
   stroke pressed without a pressure has no widths.
5. **Smoothing.** `Smoother` carries the factors alongside the points:
   resampling keeps or drops both together, the EMA uses the same α on the
   factor, and Ramer–Douglas–Peucker keeps a point when its position is more
   than ε from the chord *or* its factor is more than `WIDTH_EPSILON` (0.1)
   from the factor linearly interpolated along the chord.
6. **Rendering.** `Batch::polyline_widths` takes `(point, width)` pairs. It
   is the ADR-T24-4 strip with each pair offset by its own half-width, and
   the round discs (caps and breaks) use that point's width.
   `Batch::polyline` calls it with a constant width, so mouse strokes get
   the same mesh as before. Each point's screen width is
   `screen_width(style.width × factor, zoom)`, at least 1 px.

## Alternatives considered

- **One width per stroke** (average pressure) — no model change, but no
  taper. The vertex count is the same either way.
- **Absolute widths per point** — `style.width` would stop meaning the
  stroke's width, and every place that reads it would need to know.
- **Pressure on shapes too** — the owner chose Pen only.

## Consequences

- A pen stroke with pressure costs 4 bytes more per point (8 → 12). Mouse
  strokes are unchanged.
- The strip mitres with the half-width of each point, so where the width
  changes fast across a turn the join is slightly uneven. Breaks and caps
  are round, as before.
- Code that builds `Shape::Stroke` must set `widths`.

## Rollback plan

Remove `widths` from `Shape::Stroke` (and with it `Smoother`'s factors and
`Batch::polyline_widths`), and make the editor ignore
`InputEvent::Pressure`. Record it as a rollback ADR reverting this one.
