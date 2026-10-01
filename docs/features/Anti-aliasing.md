---
title: Anti-aliasing
task: "[[T15 Anti-aliasing]]"
adrs: ["[[ADR-T15-1 MSAA on the cached frame]]", "[[ADR-T12-1 Blocking event loop with cached frame]]"]
tutorial: "[[15 Anti-aliasing and multisampling]]"
shortcuts: []
tags: [feature]
---

# Anti-aliasing

## What it does

Circles, diagonal lines, arrow heads and freehand strokes are drawn with
smooth edges instead of stair-stepped pixels. It is on by default and costs
nothing while the app is idle.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Default (4× multisampling) | Nothing to do |
| Turn it off (old GPU, GL 2 only) | Start with `DRAW_MSAA=off draw` |
| Pick the quality | `DRAW_MSAA=2`, `4` or `8` |

## How it works

The scene is rendered into a cached render target only when something
changes (ADR-T12-1). Since T15 that target is multisampled: each pixel keeps
4 coverage samples, and macroquad *resolves* (averages) them into a normal
texture at the end of the pass. The per-frame blit of that texture is
unchanged, so the extra cost is paid only on re-render.

`src/shell/app.rs`: `msaa_samples` parses `DRAW_MSAA`,
`frame_target_params` builds the render-target parameters, and
`CachedFrame::present` creates the target with `render_target_ex`.
Rationale and alternatives: [[ADR-T15-1 MSAA on the cached frame]].

## Limits

- No runtime detection of GPU support (it would need `unsafe`); on a GL 2
  machine start with `DRAW_MSAA=off`.
- The cached frame uses about 5× the memory of a single-sampled one
  (≈ 40 MB at 2560×1600).
- Not toggleable while running.
- Costs more CPU while drawing (≈ 20 % vs ≈ 8 % without MSAA when spamming
  circles). Ways to cut it: [[Rendering performance options]].
