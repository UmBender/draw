---
id: ADR-T173-1
title: Camera command and feathered blend in the Gpu trait
status: accepted
kind: decision
date: 2026-11-15
task: T173
builds_on: [ADR-T140-1, ADR-T159-1]
amends: [ADR-T140-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T173-1 Camera command and feathered blend in the Gpu trait

## Context

[[ADR-T140-1 The Gpu and Platform traits]] has no camera: a pan or zoom of
retained chunks would rebuild their vertices. [[T146 Retained renderer in draw-gfx]]
tessellates in world coordinates, so the camera must reach the vertex shader
as a uniform. Anti-aliasing today is MSAA only.

## Decision

- `Camera { origin: [f32; 2], scale: f32 }` maps a world point `p` to view
  pixels as `(p - origin) * scale`. No rotation. `Camera::IDENTITY` is the
  default.
- `Command::SetCamera(Camera)` is valid inside a pass and applies to the draws
  after it. `BeginPass` resets the camera to identity (a pass never inherits
  one, as ADR-T159-1 says of the window). Existing command lists, which never
  say `SetCamera`, draw exactly as before.
- `Blend::AlphaFeathered` is straight alpha whose fragment alpha is also
  multiplied by a coverage computed from `Vertex::uv[0]`, the distance in
  world units from the fragment to the shape's edge (positive inside, `0` at
  the edge). Coverage is `clamp(uv.x * scale + 0.5, 0, 1)`: a feather of about
  one pixel at any zoom. Vertices away from an edge carry a large `uv[0]`.
- MSAA stays selectable through `TargetDesc::samples`; both can be combined.
- Producing feathered geometry from shapes is [[T172 Retained renderer in the app]].

## Consequences

- `RecordingGpu` records `SetCamera` as a command; `NullGpu` ignores it;
  `MqGpu` applies it on the CPU to the vertex positions and draws
  `AlphaFeathered` as `Alpha` (no coverage, same colour; macroquad is removed
  by T149).
- The app passes no `SetCamera`: its pixels do not change.
