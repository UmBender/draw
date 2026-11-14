---
id: ADR-T145-1
title: GL context, function table and Gpu over OpenGL ES 3.0
status: accepted
kind: decision
date: 2026-11-13
task: T145
builds_on: [ADR-T127-5, ADR-T140-1, ADR-T142-1, ADR-T159-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0

## Context

[[ADR-T127-5 OpenGL ES 3.0 first, Vulkan optional]] makes OpenGL ES 3.0 the
first own backend. The window backends (T143, T144) are not ready, so the
context must be testable without a window.

## Decision

- **Contexts live in `draw-sys`** (`egl.rs`, `glx.rs`), loaded with the T142
  `dlopen` loader (`libEGL.so.1`, `libGL.so.1`, `libX11.so.6`). Both
  implement the trait `GlContext` (`make_current`, `swap_buffers`,
  `proc_address`) and share one error value, `ContextError`. No call
  panics: every failure is an error value naming the EGL/GLX call.
- **EGL headless**: `EglContext::headless(width, height)` tries
  `EGL_MESA_platform_surfaceless` first (no surface, context made current
  with `EGL_NO_SURFACE`) and falls back to the default platform with a
  pbuffer. A window context is `unsafe fn EglContext::from_window` (the
  caller vouches for the native handles); the platform is `Wayland`, `X11`
  or `Default`.
- **GLX** is the fallback for X11: `GlxContext::pbuffer(display_name, w, h)`
  opens the display with `XOpenDisplay` (libX11) and makes a pbuffer
  current; `unsafe fn from_window` takes an existing window id.
  The integration test starts its own `Xvfb -displayfd` and fails, never
  skips, when `Xvfb` is missing.
- **The function table** (`Gl` in `draw-gfx/src/gpu/gl.rs`) is loaded once
  from any `Fn(&str) -> Option<*const c_void>`. Every entry point draw
  needs is checked at load: `GlError::MissingEntryPoints` lists all that are
  absent. Each call goes through a wrapper that, under `debug_assertions`
  only, drains `glGetError` and remembers the first error with the call name;
  `Gpu` methods turn it into `GpuError::Backend`.
- **`GlGpu<C: GlContext>`** implements `Gpu` over GLSL ES 3.00, one program
  per `Blend`. Handles hold GL names in an `Rc` and never call GL from
  `Drop`: a drop queues the name in a shared list the next `Gpu` call
  drains. A handle carries the id of its `GlGpu`; a foreign one gives
  `GpuError::UnknownHandle`.
- **Orientation** (ADR-T159-1): one projection for window and targets
  (`y` down, origin top left), so a target's texels are stored bottom row
  first and `read_texture` returns them in that order, unflipped.
- **Targets** with `samples > 1` draw to a multisampled renderbuffer and
  `EndPass` resolves it with `glBlitFramebuffer` into the texture; the sample
  count is clamped to `GL_MAX_SAMPLES`.
- `unsafe` is only in `crates/draw-sys/src/*` and `draw-gfx/src/gpu/gl.rs`,
  each block with `SAFETY:` and a `TEST:` marker (ADR-T142-1).

## Alternatives considered

- **Handles are indices into a slab owned by `GlGpu`** — no `Rc`, but
  resources leak until the `Gpu` drops (the trait has no delete).
- **Deleting GL objects in `Drop`** — needs the context current and alive;
  the queue avoids both.
- **`glow` or `gl` crates** — new dependencies, forbidden by ADR-T127-3.

## Consequences

GL tests run in `scripts/check.sh` and need a working EGL (Mesa llvmpipe is
enough); CI installs `libegl1 libgl1-mesa-dri xvfb`. A machine without EGL
fails the tests instead of skipping them.

## Rollback plan

Delete `egl.rs`, `glx.rs` and `gpu/gl.rs` and their module lines; nothing
else uses them until the cutover task.
