---
id: ADR-T178-1
title: X11Platform hands out a shared GL handle; the platform swaps the buffers
status: accepted
kind: decision
date: 2026-11-15
task: T178
builds_on: [ADR-T145-1, ADR-T127-2, ADR-T140-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T178-1 X11Platform hands out a shared GL handle; the platform swaps the buffers

## Context

`app::run::<X11Platform, GlGpu<_>>` needs a `GlGpu`, and `GlGpu::new` takes
its `GlContext` by value. `X11Platform` owns its `GlSurface` privately and
swaps the buffers itself in `next_frame`. `main` forbids `unsafe`, so it
cannot alias the context. `GlGpu` also fixes the size of the default
framebuffer at creation, while the window is resized by the user.

## Decision

- `X11Platform::share_gl(&self) -> SharedGl`: a cheap clone handle (`Rc`
  inside) that implements `GlContext`.
- The **platform swaps** the buffers (`next_frame`, as before); the Gpu's
  own `swap_buffers` is not called by `app::run`. One side swaps, never
  both.
- `SharedGl::window_size()` reports the platform's current size in pixels
  (a shared `Cell` the platform updates on every resize). `main` wraps the
  `GlGpu` in `X11Gpu` (`src/shell/gpu_x11.rs`), a `Gpu` that sets the
  window size on the inner `GlGpu` before each `submit`. Nothing else
  changes in `draw-gfx`.
- Drop order: every handle must be dropped before the platform (the GL
  window goes before the X window). `app::run` drops the Gpu, and so its
  handle, before the platform; the doc of `share_gl` says so. The last
  handle to drop destroys the context.
- `--platform macroquad|native` (default `macroquad`) picks the
  interpreters. `native` on an X11 session runs
  `app::run::<X11Platform, X11Gpu>`. On a Wayland session `native` is T171.

## Consequences

The Wayland code is untouched. A later task that makes the Gpu swap
(T147) replaces `next_frame`'s swap with a call on the Gpu.
