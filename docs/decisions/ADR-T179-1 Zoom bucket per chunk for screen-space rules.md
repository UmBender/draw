---
id: ADR-T179-1
title: Zoom bucket per chunk for screen-space rules
status: accepted
kind: decision
date: 2026-11-16
task: T179
builds_on: [ADR-T173-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T179-1 Zoom bucket per chunk for screen-space rules

## Context

`draw_shape` works in pixels: the minimum screen width, the joint threshold
and the circle segment counts depend on the zoom. The camera uniform of
[[ADR-T173-1 Camera command and feathered blend in the Gpu trait]] only
scales and offsets vertices; a vertex shader cannot change a vertex count
or clamp a width per shape.

## Decision

- `tessellate_at(shape, zoom)` applies the minimum width
  (`1 px / zoom`), the joint rule (round discs only above 2 px) and the
  segment counts (chord error 0.25 px) at `zoom`. `tessellate(shape)` is
  `tessellate_at(shape, 1.0)`.
- The caller passes a **bucket**: `zoom_bucket(zoom)` rounds the zoom to the
  nearest power of two in the log domain, so 0.25, 1 and 4 are their own
  buckets. A chunk is tessellated for its bucket.
- `ChunkCache::set_zoom(zoom)` drops the chunks when the bucket changes and
  keeps them otherwise. A pan never changes the bucket, so a pan never
  tessellates; a zoom re-tessellates only on crossing a bucket edge.
- Inside a bucket the on-screen width of a thin line is off by at most a
  factor of √2 from the exact rule, and only where the minimum width
  applies. The feather (1 px, from the shader) is exact at any zoom.

## Consequences

- No shader or `Gpu` change; the world-space vertices stay camera
  independent inside a bucket, so the T146 cache proptest holds.
- Exact parity with `draw_shape` is at bucket zooms; between them the
  difference is bounded as above.
