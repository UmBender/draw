---
id: ADR-T179-2
title: Parity metric against the screen-space drawing
status: accepted
kind: decision
date: 2026-11-16
task: T179
builds_on: [ADR-T179-1, ADR-T173-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T179-2 Parity metric against the screen-space drawing

## Context

The e2e tolerance (64 pixels, 8 per channel) was set for two renderings with
the same anti-aliasing. `draw_shape` into a single-sample target has hard
edges; the tessellated path feathers every edge by one pixel
(`Blend::AlphaFeathered`). The two can only agree away from edges, and in
how much ink they lay down.

## Decision

The parity test renders both paths on headless EGL into 64×64 targets on a
white background and requires:

1. every pixel that differs by more than 8 in some channel lies within one
   pixel (3×3 neighbourhood) of an edge of the reference, i.e. a pixel
   whose neighbourhood in the reference is not uniform;
2. the total ink (sum of `255 - luminance`) is within 4 % of the
   reference's.

Rule 1 is the e2e channel tolerance applied away from edges; rule 2 catches
a wrong width, a missing fill or a missing arrow head, which rule 1 hides.

## Consequences

- A stricter metric would need MSAA on the reference; T172 can tighten it
  when the app itself switches path.
