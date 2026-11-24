---
id: T189
title: Direct3D 11 Gpu
status: todo
wave: 79
branch: task/T189-direct3d-11-gpu
depends_on: [T188]
adrs: []
feature: "[[Windows]]"
tutorial: ""
tags: [task, windows]
---

# T189 Direct3D 11 Gpu

## Goal

`draw-gfx` has `D3d11Gpu`, a second interpreter of the `Gpu` trait. It
renders the same command lists as the GL backend on a hardware device or
on WARP, presents through a flip-model swap chain with a frame-latency
waitable object on the window of T188, and survives a device loss.

## Known facts (2026-11-24)

- Decisions: [[Windows port proposal]] §3.1, as an ADR written in
  [[T187 Windows ADRs and portable seams]].
- The `Gpu` trait (`crates/draw-gfx/src/gpu.rs`) is API-neutral: vertex
  and index buffers, RGBA textures, render targets with a sample count
  and a filter, blend pipelines, read-back, one `CommandList` per frame
  with `BeginPass`, `SetCamera`, `Draw`, `EndPass`.
- `gl.rs` has three GLSL ES 3.00 programs (the plain fragment shader and
  the feathered one of the retained path); the HLSL ports must give the
  same pixels within the parity tolerance.
- Flip-model back buffers cannot be multisampled: `samples > 1` renders
  to an off-screen target resolved with `ResolveSubresource`.
- Shaders are compiled offline with `fxc` (Windows SDK, present on the
  `windows-latest` runner) and the bytecode is checked in; the Windows
  job recompiles and compares.
- The D3D11 debug layer needs the "Graphics Tools" optional feature; the
  spec step checks whether the runner has it or the job installs it.
- `GpuError` gains `DeviceLost` (amends [[ADR-T140-1 The Gpu and Platform traits]]);
  the app drops and rebuilds its GPU caches from the document. That path
  is testable on Linux with `RecordingGpu`.
- Windows tests run only on the Windows CI job (proposal, question 7).

## Spec

Draft; the spec step names the final tests.

- **AC-1** — Device creation: hardware first, WARP when it fails or when
  asked; the debug layer in tests.
  Tests: `d3d11_creates_a_device_or_warp`, `d3d11_warp_when_asked`.
- **AC-2** — Resources: buffers, textures, partial texture writes,
  targets, filters and read-back.
  Tests: `d3d11_texture_write_and_read_back_round_trip`,
  `d3d11_target_texture_reads_what_was_drawn`,
  `d3d11_msaa_target_resolves`.
- **AC-3** — Parity: the parity scenes rendered on WARP match goldens
  produced by the GL backend on Linux (llvmpipe), within a stated
  tolerance; the goldens are checked in.
  Test: `d3d11_matches_the_gl_goldens`.
- **AC-4** — The swap chain on the T188 window presents, follows resizes
  and DPI changes, and its frame-latency handle joins the platform's
  wait (at most one frame queued).
  Tests: `d3d11_swap_chain_presents_and_resizes`,
  `d3d11_frame_latency_wait_paces_frames`.
- **AC-5** — Shaders: the checked-in DXBC matches `fxc` on the HLSL
  sources. Test: `hlsl_bytecode_matches_the_sources` (Windows job).
- **AC-6** — Device loss: the removed and reset `HRESULT`s map to
  `GpuError::DeviceLost`; the app rebuilds its layers and draws the
  next frame.
  Tests: `device_removed_hresults_map_to_device_lost` (pure),
  `device_lost_rebuilds_the_gpu_caches` (`RecordingGpu`, on Linux).

## Out of scope

- Using it from `main` ([[T192 Windows platform in main and the e2e suite]]).
- Direct3D 12, Vulkan ([[T150 Optional Vulkan backend for the Gpu trait]]).

## Files owned

- `crates/draw-gfx/src/gpu/d3d11.rs` and `crates/draw-gfx/src/gpu/d3d11/**` (new)
- `crates/draw-gfx/shaders/**` (new: HLSL and DXBC)
- `crates/draw-gfx/src/gpu.rs` (`DeviceLost`, module line), `crates/draw-gfx/src/lib.rs`
- `crates/draw-gfx/Cargo.toml` (`windows` features), `Cargo.lock`
- `crates/draw-gfx/tests/d3d11*.rs`, the GL golden images for parity
  (`crates/draw-gfx/tests/goldens/**`)
- `crates/draw-sys/src/win32/platform.rs` (handing out the window and
  taking the frame-latency handle)
- `src/shell/app.rs` and `src/shell/recording.rs` (the device-loss path)
- `tests/unsafe_levels.rs` (`gpu/d3d11.rs` allowed)
- `.github/workflows/ci.yml` (Graphics Tools, the shader check)
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T189): specify the Direct3D 11 Gpu`
- [ ] tests — `test(T189): add failing tests for the Direct3D 11 Gpu`
- [ ] models — `feat(T189): add D3d11Gpu and GpuError::DeviceLost`
- [ ] behaviour — `feat(T189): implement the Direct3D 11 Gpu`
- [ ] quality — `chore(T189): pass clippy and rustfmt`

## Log

- 2026-11-24 — written from the Windows port proposal.
