---
title: Anti-aliasing and multisampling
step: 15
requires: ["[[12 The app loop, idle redraw and UI]]"]
feature: "[[Anti-aliasing]]"
code: ["src/shell/app.rs"]
tags: [tutorial]
---

# Anti-aliasing and multisampling

> **Learning path step 15.** Requires: step 12 · Next: step 16

## Why this matters here

A competitive-programming sketch is mostly circles (tree nodes), diagonal
lines (edges) and thin strokes — exactly the shapes that look worst when
rasterised one sample per pixel. Jagged edges make a quick drawing harder to
read, and at small zoom thin lines break up into dashed pixels.

## The concept

**Aliasing.** The GPU decides for each pixel whether a triangle covers it by
testing *one* point, the pixel centre. A pixel is either fully in or fully
out, so a slanted edge becomes a staircase:

```
one sample per pixel        4 samples per pixel
 . . . # #                   . . ░ ▒ #
 . . # # #                   . ░ ▓ # #
 . # # # #                   ░ ▓ # # #
```

**Multisample anti-aliasing (MSAA).** Each pixel stores N coverage samples
(4 here) at different sub-pixel positions. The fragment shader still runs
about once per pixel, but the triangle test runs per sample. At the end the
samples are averaged — *resolved* — into an ordinary texture: a pixel half
covered by a black line becomes half-black.

Other approaches, and why draw does not use them:

- **Supersampling** — render at 2× resolution and shrink: correct, but 4× the
  shading cost and a linear-filtered downscale blurs 1-px lines.
- **Analytic AA** — add a ring of fading vertices around every outline: works
  on any GPU but every shape path in the renderer must do it right.
- **Post-process AA (FXAA)** — a blur that guesses edges; needs a custom
  shader and softens text.

## How draw implements it

T12 already draws the scene into a cached render target and blits it each
frame ([[ADR-T12-1 Blocking event loop with cached frame]]). T15 only changes
how that target is created, in `src/shell/app.rs`:

- `frame_target_params(samples)` returns `RenderTargetParams { sample_count:
  samples, depth: false }`.
- `CachedFrame::present` calls `render_target_ex` with it. With
  `sample_count > 1` macroquad attaches a multisampled renderbuffer plus a
  single-sample *resolve* texture, and `RenderTarget::texture` is the resolve
  texture — so the blit code did not change at all.
- `msaa_samples(std::env::var("DRAW_MSAA").ok().as_deref())` picks the count:
  `off`/`0`/`1` → 1, `2`/`4`/`8` → that count, anything else → 4. It is a pure
  function, so a proptest checks it returns a supported count for *any*
  string.

Why not detect GPU support automatically? miniquad knows
(`features.resolve_attachments`), but only through `unsafe fn
get_internal_gl`, and the crate forbids `unsafe`
([[ADR-0014 Minimal dependencies and no unsafe]]). The environment variable is
the escape hatch instead ([[ADR-T15-1 MSAA on the cached frame]]).

The cost is paid only when the scene is re-rendered; an idle app still sleeps
and blits nothing new.

## Try it

1. Draw a few circles and diagonal lines, then restart with
   `DRAW_MSAA=off cargo run --release` and compare the edges.
2. Zoom out to 0.05 with thin strokes: compare `DRAW_MSAA=2` and `8`.
3. Change `FRAME_FILTER` to `FilterMode::Linear` and run at a fractional DPI
   scale: the whole frame softens. Why does a 1:1 blit want `Nearest`?

## Further reading

- [Learn OpenGL — Anti Aliasing](https://learnopengl.com/Advanced-OpenGL/Anti-Aliasing)
  — MSAA, multisampled framebuffers and resolving with a blit.
- Khronos wiki, *Multisampling*.
