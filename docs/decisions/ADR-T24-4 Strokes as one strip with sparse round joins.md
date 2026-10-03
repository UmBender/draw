---
id: ADR-T24-4
title: Strokes as one strip with sparse round joins
status: accepted
kind: decision
date: 2026-10-03
task: T24
builds_on: [ADR-T07-1, ADR-T24-3]
amends: [ADR-T07-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T24-4 Strokes as one strip with sparse round joins

## Context

After batching ([[ADR-T24-3 Batched meshes and append-only document updates]]),
a full re-render of 5 120 pen strokes takes ≈ 26 ms zoomed out and
≈ 210 ms at medium zoom on the target. `perf` puts ≈ 55 % of that in
copying vertices into GL buffers. The time grows with the number of
vertices, not the number of draw calls.

[[ADR-T07-1 Screen-space tessellation in the renderer]] draws a stroke as
one quad per segment (4 vertices). When the stroke is wider than
`JOINT_THRESHOLD_PX` on screen, it also draws a round disc at **every**
point (≥ 10 vertices each). Once zoomed in far enough for strokes to pass
2 px, each point costs ≈ 14 vertices instead of 4. Pen strokes keep
dozens of points after simplification.

## Decision

- `Batch::polyline` draws a polyline of screen points as **one triangle
  strip**: 2 vertices per point, offset by half the width along the mitre
  of the neighbouring segment normals, so consecutive segments share
  their vertices and the joins have no gaps.
- Points closer than `STROKE_MIN_STEP_PX` (¼ px, the chord tolerance) to
  the last kept point are skipped. The error stays within ¼ px, and a
  zoomed-out stroke loses most of its sub-pixel points.
- Where the mitre would be longer than `MITER_LIMIT` half-widths (a sharp
  turn), the strip ends at that point and a new one starts there.
- Strokes wider than `JOINT_THRESHOLD_PX` get a round disc at those breaks
  and at both ends (round caps). Gentle turns are mitred: the corner
  sticks out at most `MITER_LIMIT − 1` half-widths beyond a round join.
- A strip may be longer than one batch chunk. `Batch::strip` flushes and
  repeats the last pair, so the strip continues in the next chunk.
- Ellipse outlines keep using `Batch::strip`; discs, fills and segment
  counts are unchanged.

## Alternatives considered

- **Keep a disc at every point, with fewer segments** — still ≥ 6 vertices
  per point on top of the quad, and visibly polygonal joints.
- **Mitre every join with no break** — spikes at hairpin turns, where the
  mitre length goes to infinity.
- **Drawing the cached layer moved/scaled during a gesture** — removes the
  re-render during pan/zoom but not the one when it settles. It needs a
  timed wake-up (amending ADR-T12-1). It is a separate task if this is not
  enough.

## Consequences

- A thick stroke costs ≈ 2 vertices per point plus two caps, instead of
  ≈ 14 per point. A thin stroke costs 2 per point instead of 4.
- Gentle joins of thick strokes are mitred rather than round. The
  difference is at most `MITER_LIMIT − 1` half-widths at the outer
  corner.
- A translucent stroke blends once over most of its length. Before, it
  was darker wherever discs and quads overlapped. The palette is opaque
  today.

## Rollback plan

Make `draw_polyline` draw one `Batch::line` per segment again, with a
disc at every point when thick. Record it as a rollback ADR reverting
this one.
