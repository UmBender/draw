---
id: ADR-T127-5
title: OpenGL ES 3.0 first, Vulkan optional
status: accepted
kind: decision
date: 2026-11-12
task: T127
builds_on: [ADR-T127-1, ADR-T127-4]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, 2.0, gpu]
---

# ADR-T127-5 OpenGL ES 3.0 first, Vulkan optional

## Context

The own backend ([[ADR-T127-4 An own backend replaces miniquad and macroquad]])
needs a GPU API. The owner asked for "OpenGL or Vulkan" to be weighed on
2026-11-12; the proposal's section 6 gives engineering judgement, **not a
measurement**: the measured bottleneck of [[ADR-T24-5 Redraw budgets]] is CPU
re-tessellation, not driver overhead. Those numbers predate T26 and the
re-measurement on the i5-8250U is still open ([[2.0 baseline]], section 1).

## Decision

- 2.0.0 uses **OpenGL ES 3.0 over EGL, with GLX where EGL is missing**.
- **Vulkan is an optional later task** (T150), taken only if a measurement on
  the target shows driver overhead or frame pacing as the ceiling, or the
  owner drops hardware without Vulkan. It does not block 2.0.0.
- The `Gpu` trait is written for both: explicit typed resources (buffers,
  pipelines, textures, render targets), a recorded command list submitted
  once per frame and no hidden global state; it is reviewed against Vulkan's
  model in the spec step before any backend. A Vulkan backend is a second
  interpreter, monomorphised, with no `dyn`.
- Analytic anti-aliasing replaces 4x MSAA only if measured better; MSAA is
  the fallback.

## Alternatives considered

- **Vulkan first** — much more `unsafe` code (device, swapchain, our own
  allocator, sync), swapchain edge cases on both platforms, and no reach on
  older GPUs that the X11 users have.
- **Desktop OpenGL 4.x** — narrower driver reach than GLES 3.0 for no gain.

## Consequences

- Smaller first backend; Mesa's GL is mature on the UHD 620.
- A badly shaped `Gpu` trait would make Vulkan expensive later: hence the
  review.

## Rollback plan

Swap the `Gpu` interpreter chosen in `main`; the trait does not change.
If GLES proves a ceiling, promote T150 and record an ADR that builds on this
one.
