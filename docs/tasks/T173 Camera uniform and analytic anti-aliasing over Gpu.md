---
id: T173
title: Camera uniform and analytic anti-aliasing over Gpu
status: done
wave: 72
branch: task/T173-camera-uniform-and-analytic-anti-aliasi
depends_on: [T146]
adrs: [ADR-T173-1]
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

Refined (ADR-T173-1): `Command::SetCamera(Camera)` inside a pass (`Camera { origin, scale }`, reset by `BeginPass`) and `Blend::AlphaFeathered` (coverage from `uv[0]`, the world-unit distance to the edge).

- AC-1 tests: `camera_maps_world_to_view_pixels`, `camera_default_is_identity` (unit, `gpu.rs`); `recorder_pan_and_zoom_change_only_the_camera` and `null_gpu_accepts_set_camera` (`tests/recorder.rs`); `gl_camera_pan_moves_the_triangle` and `gl_camera_resets_at_begin_pass` (`gl_gpu.rs`); `mq_camera_offsets_vertices` (`gpu_mq.rs` unit, on the pure helper).
- AC-2 tests (`gl_gpu.rs`, headless EGL): `feather_edge_is_one_pixel_wide_at_zoom_quarter`, `..._one`, `..._four`, `feather_interior_is_opaque`.
- AC-3: `scripts/e2e.sh --backend all`.

## Out of scope

- Drawing the app through the retained renderer ([[T172 Retained renderer in the app]]).

## Files owned

- `crates/draw-gfx/src/**`, `crates/draw-gfx/tests/**`
- `src/shell/gpu_mq.rs`, `src/shell/recording.rs`, `src/shell/render_ctx.rs`, `tests/recorder.rs`, and the other call sites of the changed trait items
- `ADR-T173-*` (amends ADR-T140-1)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T173): specify the camera and the anti-aliasing`
- [x] tests — `test(T173): add failing tests for the camera and the anti-aliasing`
- [x] models — `feat(T173): add the camera to the Gpu trait`
- [x] behaviour — `feat(T173): implement the camera uniform and analytic anti-aliasing`
- [x] quality — `chore(T173): pass clippy and rustfmt`

## Log

- 2026-11-15 — split from T146 (AC-2 camera part, AC-3) by the owner.
- 2026-11-15 — done (batch). The tests step does not compile before the models step (new types), as in T163/T165. ADR-T173-1: `Command::SetCamera(Camera)` (origin + scale, reset by `BeginPass`) and `Blend::AlphaFeathered` (a separate GL program; coverage `clamp(uv[0] * scale + 0.5, 0, 1)` from the world-unit edge distance in `uv[0]`). `MqGpu` moves vertices through the camera on the CPU and draws feathered as plain alpha. MSAA is unchanged (`TargetDesc::samples`). `scripts/check.sh` green; `scripts/e2e.sh --backend all` green (the app sends no `SetCamera`).
- 2026-11-15 — deviation, for the owner: the tessellator still emits `uv` zero and no feather ring, so no shape is drawn feathered yet; producing feathered geometry is for T172. No feature or tutorial note (`feature` and `tutorial` are empty).
