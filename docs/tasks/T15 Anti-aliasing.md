---
id: T15
title: Anti-aliasing
status: ready
wave: 7
branch: task/T15-antialiasing
depends_on: [T12]
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-T12-1 Blocking event loop with cached frame]]"]
feature: "[[Anti-aliasing]]"
tutorial: "[[15 Anti-aliasing and multisampling]]"
tags: [task]
---

# T15 Anti-aliasing

## Goal

Strokes, circles and diagonal lines are drawn with smooth edges instead of
stair-stepped pixels, without breaking the idle-friendly loop or the cached
frame (ADR-T12-1).

## Spec

To be refined at the spec step; starting point:

- **AC-1** — The cached scene render target is multisampled (MSAA, 4× via
  `RenderTargetParams::sample_count`) and resolved before it is blitted, so the
  cost is paid only when the scene is re-rendered, not on every blit.
  *Test:* pure helper, e.g. `app::tests::msaa_params_request_four_samples`.
- **AC-2** — Fallback: if the GL context does not support multisampled targets
  (GL2/GLES2), the app falls back to `sample_count: 1` and still renders.
  The decision is a pure function of the reported capability.
  *Test:* `app::tests::msaa_falls_back_without_support`.
- **AC-3** — The blit keeps the 1:1 physical-pixel mapping (no blur from linear
  filtering on the resolved texture at DPI 1 and 2).
  *Test:* `app::tests::physical_size_*` (existing) plus a manual check.
- **AC-4** — Manual check on the target (Wayland, i5-8th gen iGPU): edges are
  smooth at zoom 0.05, 1 and 20; idle CPU stays ≈ 0 %; redraw latency while
  drawing is not noticeably worse. Results in the log.

ADR to write at the spec step: `ADR-T15-1 MSAA on the cached frame`
(builds on ADR-0006, ADR-T12-1), including why MSAA was chosen over
analytic edge feathering in the tessellator, and the fallback.

## Out of scope

Text anti-aliasing (handled by the font rasterizer), per-shape AA toggles.

## Files owned

`src/shell/app.rs`, new `ADR-T15-*`.

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 15 — requires step 12.

## Log
