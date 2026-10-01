---
id: ADR-0013
title: World-space widths and zoom limits
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0004]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0013 World-space widths and zoom limits

## Context

With zoom, stroke width can either scale with the drawing or stay a fixed
number of screen pixels.

## Decision

- Stroke widths are stored in **world units** and scale with zoom, so a drawing
  looks the same at any zoom level (like paper).
- Rendering clamps the on-screen width to at least 1 px, so zoomed-out strokes
  stay visible.
- Zoom range `[0.05, 20.0]`, scroll step ×1.15 per notch, anchored at the cursor.
- New strokes are created with `width = chosen_px / zoom`, so the pen always
  *feels* the same size on screen.

## Alternatives considered

- **Screen-space widths** — zoomed-out diagrams look heavy and zoomed-in ones
  look hairline.

## Consequences

- Hit-test tolerances are computed in screen pixels then converted to world
  units with the camera.

## Rollback plan

Amending ADR changing `shell::render` width computation only.
