---
id: ADR-0004
title: Vector object model
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0003]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0004 Vector object model

## Context

The canvas must be infinite, zoomable, and support moving and copying blocks.
Pixel canvases make all of those expensive.

## Decision

The document is an ordered list of **vector shapes** in world coordinates:
`Stroke` (polyline), `Line`, `Arrow`, `Rect`, `Ellipse`. Each has a style
(palette colour index, width) and closed shapes have an optional fill.

- **Eraser** removes whole shapes it touches (no partial erasing).
- **Bucket** sets the fill of the topmost closed shape (`Rect` or `Ellipse`)
  under the cursor. Clicking empty space does nothing. No pixel flood fill.
- z-order = list order; new shapes go on top.

## Alternatives considered

- **Raster tiles** — supports true flood fill but costs memory, makes
  zoom blurry, and makes move/copy hard.
- **Partial stroke erasing** — splitting polylines adds complexity for little
  value on a scratchpad.

## Consequences

- Infinite canvas and zoom come for free; memory grows with the number of
  shapes, not the canvas size.
- Hit-testing is a linear scan with AABB rejection, fine for thousands of
  shapes. Revisit only if profiling shows a problem.

## Rollback plan

Adding a closed freehand-region fill later would be an amending ADR (new
shape kind), not a replacement.
