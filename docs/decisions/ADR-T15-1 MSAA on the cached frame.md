---
id: ADR-T15-1
title: MSAA on the cached frame
status: accepted
kind: decision
date: 2026-10-01
task: T15
builds_on: [ADR-0006, ADR-T12-1, ADR-0014]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T15-1 MSAA on the cached frame

## Context

Shapes are tessellated into triangles and rasterised with one sample per
pixel, so circles, diagonal lines and thin strokes show stair-stepped edges.
The scene is rendered into a cached render target and only re-rendered when
something changed (ADR-T12-1); every woken frame blits that texture.

macroquad 0.4 can create a multisampled render target
(`RenderTargetParams::sample_count`) and resolves it into a plain texture at
the end of the pass, so the blit stays one textured quad. Multisampled render
targets need GL ≥ 3 / GLES 3; miniquad reports support in
`ContextInfo::features.resolve_attachments`, but that is only reachable
through `unsafe fn get_internal_gl`, and `unsafe` is forbidden (ADR-0014).

## Decision

- The cached frame is a multisampled render target with **4 samples** by
  default. The resolved texture keeps `FilterMode::Nearest` and the 1:1
  physical-pixel blit from ADR-T12-1.
- The sample count can be overridden with the environment variable
  **`DRAW_MSAA`**: `1`, `0` or `off` disable multisampling; `2`, `4`, `8`
  select that count; anything else (or unset) means the default. Parsing is a
  pure function in `shell::app`.
- There is no runtime capability query. The target (Linux desktop, Mesa,
  GL 4.x) supports multisampled render targets; `DRAW_MSAA=off` is the escape
  hatch for a GL 2 machine.

## Alternatives considered

- **Window MSAA (`Conf::sample_count`)** — antialiases the default
  framebuffer only; our scene is drawn into an off-screen target, so it would
  have no effect on shapes.
- **Analytic edge feathering in the tessellator** (an extra ring of
  transparent vertices per outline) — works on GL 2, but touches every shape
  path in `shell::render`, doubles vertex counts and needs per-shape care for
  joins and fills. MSAA gets the same visual result for every primitive,
  including toolbar icons, with no renderer change.
- **Query `resolve_attachments` via `get_internal_gl`** — needs `unsafe`.
- **Supersampling (render at 2× and downscale)** — 4× the fill cost and
  memory, and a linear-filtered downscale blurs 1-px lines.

## Consequences

- Smooth edges for everything drawn into the scene; cost is paid only on
  re-render (the blit is unchanged), so idle CPU stays ≈ 0 %.
- The cached frame uses ~5× the memory of a single-sampled target (4-sample
  colour renderbuffer + resolve texture): about 40 MB at 2560×1600.
- On a GL 2 context the app may fail to create the target unless started
  with `DRAW_MSAA=off`.

## Rollback plan

Set the default sample count to 1 (one constant in `shell::app`) or revert the
T15 behaviour commit; add a rollback ADR reverting this one.
