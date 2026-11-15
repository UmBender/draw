---
id: T173
title: Camera uniform and analytic anti-aliasing over Gpu
status: todo
wave: 72
branch: task/T173-camera-uniform-and-analytic-anti-aliasi
depends_on: [T146]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T173 Camera uniform and analytic anti-aliasing over Gpu

## Goal

The `Gpu` trait carries the camera, so pan and zoom of retained chunks change one uniform; the GL interpreter anti-aliases analytically in the fragment shader, with MSAA kept as a fallback.

## Known facts (2026-11-15)

- Split out of [[T146 Retained renderer in draw-gfx]] by the owner after the batch blocked it: `Command`/`View`/`Geometry` in the `Gpu` trait carry no camera, and changing them touches every interpreter (T146 Log).
- The trait changes through an ADR amending [[ADR-T140-1 The Gpu and Platform traits]] (as [[ADR-T159-1 Texture filter and readback in the Gpu trait]] did).
- The app does not draw through the retained renderer yet ([[T172 Retained renderer in the app]]): the shell passes the identity camera and its pixels do not change.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — The camera (origin and scale, or a 3×3 matrix) is part of a pass in the `Gpu` trait; every interpreter (`gl.rs`, `gpu_mq.rs`, `RecordingGpu`, `NullGpu`) applies or records it. Tests: on the recorder, a pan or zoom over retained chunks issues the same geometry ids and only a new camera; ADR amending ADR-T140-1.
- **AC-2** — Analytic anti-aliasing in the GL fragment shader (distance-based feather, about 1 px at any zoom); MSAA stays selectable. Pixel tests read back from headless EGL on lines, circles and thin strokes at zoom 0.25, 1 and 4.
- **AC-3** — `scripts/e2e.sh --backend all` green: no pixel change in the app.

## Out of scope

- Drawing the app through the retained renderer ([[T172 Retained renderer in the app]]).

## Files owned

- `crates/draw-gfx/src/**`, `crates/draw-gfx/tests/**`
- `src/shell/gpu_mq.rs`, `src/shell/recording.rs`, `src/shell/render_ctx.rs`, `tests/recorder.rs`, and the other call sites of the changed trait items
- `ADR-T173-*` (amends ADR-T140-1)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T173): specify the camera and the anti-aliasing`
- [ ] tests — `test(T173): add failing tests for the camera and the anti-aliasing`
- [ ] models — `feat(T173): add the camera to the Gpu trait`
- [ ] behaviour — `feat(T173): implement the camera uniform and analytic anti-aliasing`
- [ ] quality — `chore(T173): pass clippy and rustfmt`

## Log

- 2026-11-15 — split from T146 (AC-2 camera part, AC-3) by the owner.
