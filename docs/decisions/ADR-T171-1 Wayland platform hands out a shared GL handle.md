---
id: ADR-T171-1
title: WaylandPlatform hands out a shared GL handle; the platform swaps the buffers
status: accepted
kind: decision
date: 2026-11-16
task: T171
builds_on: [ADR-T178-1, ADR-T145-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T171-1 WaylandPlatform hands out a shared GL handle; the platform swaps the buffers

## Context

The same problem as [[ADR-T178-1 X11 platform hands out a shared GL handle]]:
`GlGpu::new` takes its `GlContext` by value, `WaylandPlatform` owns the
`EglContext` inside its window and swaps in `next_frame` (it also paces on
the frame callback), and `main` forbids `unsafe`.

## Decision

- The window keeps its `EglContext` in an `Rc`; `WaylandPlatform::share_gl()`
  returns `SharedGl` (`Rc` clone of the context plus a shared `Cell` with the
  size in physical pixels, logical size × buffer scale, updated whenever the
  platform applies a configure). `SharedGl` implements `GlContext`.
- The **platform swaps**; the Gpu never does.
- `src/shell/gpu_x11.rs`'s `X11Gpu` becomes the generic `SizedGpu<C>` over a
  local trait `WindowSize`; `X11Gpu` and `WaylandGpu` are aliases. The close
  wrapper is `ClosingWayland`, like `ClosingX11`.
- Drop order as in ADR-T178-1: every handle is dropped before the platform.
- `--platform native` picks `WaylandPlatform` when the backend is Wayland and
  `X11Platform` when it is X11.

## Consequences

One Gpu wrapper serves both native platforms. T147 replaces the swap in
`next_frame` for both at once.
