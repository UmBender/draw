---
id: T158
title: Render batch over Gpu
status: in-progress
wave: 68
branch: task/T158-render-batch-over-gpu
depends_on: [T157]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T158 Render batch over Gpu

## Goal

Slice S2 of [[T155 Gpu migration of the drawing]]: `render.rs`: `Batch` emits `Vertex` and `Command::Draw`; `GlyphTextures` holds `Gpu` texture handles; `draw_guides` and `draw_selection*` become tessellated quads. Test: `scene_over_recording_gpu_emits_begin_draws_end`, `batch_over_recording_gpu_splits_chunks_at_the_limits`.

## Known facts (2026-11-13)

- Split out of T155 after the batch blocked it (about 7,000 lines; every slice needs `e2e.sh --backend all`). The design and the slice list are in the T155 note; the traits are in [[ADR-T140-1 The Gpu and Platform traits]].
- No pixel change allowed.

## Spec

Design (decided here, see the Log): `MeshSink::draw(&mut self, vertices: &[Vertex], indices: &[u32], page: Option<u32>)` with `draw_gfx::gpu::Vertex`; `Batch` keeps its `Color` API and converts colours with macroquad's own byte conversion, so pixels do not change. `GpuSink<G>` collects `Command::Draw` (page → texture handle) and wraps them in one pass. `GlSink` submits each chunk through one thread-local `MqGpu`. `GlyphTextures<G: Gpu>` holds `G::Texture` handles. `BeginPass { target: None }` keeps the current macroquad camera (T159 restores the reset).

- **AC-1** — `Batch` emits `Vertex` and `u32` indices with the colour bytes macroquad used, in chunks within the limits. Tests: the existing `batch_*` tests (their `Recorder` now takes `Vertex`), `batch_over_recording_gpu_splits_chunks_at_the_limits`.
- **AC-2** — A batch over `GpuSink<RecordingGpu>` yields `BeginPass`, one `Draw` per chunk bound to its page texture, `EndPass`. Tests: `scene_over_recording_gpu_emits_begin_draws_end`, `gpu_sink_binds_the_page_texture`.
- **AC-3** — `GlyphTextures` holds `Gpu` handles: `glyph_textures_sync_creates_one_texture_per_page`, `glyph_textures_sync_writes_pending_uploads`.
- **AC-4** — Guides are tessellated quads: `push_guides_emits_one_quad_per_guide` (selection already is).
- **AC-5** — `BeginPass { target: None }` and its `EndPass` leave the macroquad camera alone; a target pass owns it: `pass_without_target_keeps_the_camera`, `pass_with_target_owns_the_camera`.
- **AC-6** — `tests/perf.rs`'s `CountingSink` takes `Vertex`/`u32`; `scripts/check.sh` green; `e2e.sh --backend all` green with no pixel change (checked by the runner or owner if the tools are missing here).

## Out of scope

- Other slices of T155; input, the loop and `main` ([[T156 Platform migration of input and the loop]]); anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `src/shell/render.rs`, `src/shell/glyphs.rs`
- `src/shell/gpu_mq.rs` (`BeginPass { target: None }` keeps the current macroquad camera until T159 moves the frame cache to passes; T159 restores the reset)
- `tests/perf.rs` (`CountingSink` takes `Vertex`/`Command::Draw`)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T158): specify the slice`
- [ ] tests — `test(T158): add failing tests`
- [ ] models — `feat(T158): add the types`
- [ ] behaviour — `refactor(T158): migrate the slice`
- [ ] quality — `chore(T158): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T155 by the owner, following the batch's recommendation.
- 2026-11-13 — BLOCKED (batch): the slice cannot be done inside *Files owned* (`render.rs`, `glyphs.rs`) without a pixel change or a broken build. Found by reading the code (no code written, so the tree stays green):
  1. **Camera reset.** `MqGpu` runs `BeginPass { target: None }` as `set_default_camera()` and `EndPass` likewise (`gpu_mq.rs`, `execute`), and `submit` rejects a `Draw` outside a pass. But `app.rs` (~l. 1286) calls `draw_shapes` between `set_camera(render target)` and `set_default_camera()` for the frame cache. A `GlSink` that submits through `MqGpu` would draw the cached frame to the screen instead of the target. Fixing it needs either a change in `gpu_mq.rs` (T157) or the frame cache moved to passes first (T159, which depends on T158).
  2. **Texture handles.** `MqTexture` keeps only an id into the interpreter's private table, so `GlyphTextures` over `Gpu` handles forces `GlSink` to submit through one shared `MqGpu`; it cannot be done half-way without dead code.
  3. **Callers outside the owned files** use the API that would change: `tests/perf.rs` implements `MeshSink::draw(&mut Mesh, …)`; `minimap.rs`, `sheet.rs`, `command_palette.rs`, `toolbar.rs`, `app.rs` and `ui_shapes.rs` use `Batch`, `GlSink`, `hex_color`, `to_mq_color`.
  Recommended fix: owner decides one of (a) add to T158's files `src/shell/gpu_mq.rs` and make `BeginPass { target: None }` keep the current macroquad camera until T159 moves the frame cache to passes (then restore the reset in T159); or (b) swap the order: do the frame cache (T159) first. Also add `tests/perf.rs` to the files owned (its `CountingSink` must take `Vertex`/`Command::Draw`) and keep `Batch`'s `Color` API so the other shell files compile until T159–T161. The named tests (`scene_over_recording_gpu_emits_begin_draws_end`, `batch_over_recording_gpu_splits_chunks_at_the_limits`) can use `RecordingGpu` from `src/shell/recording.rs`.
- 2026-11-13 — UNBLOCKED by the owner: option (a) of the blocker. `gpu_mq.rs` and `tests/perf.rs` added to *Files owned*; keep `Batch`'s `Color` API so the other shell files compile until T159–T161. T159 must restore the camera reset in `BeginPass { target: None }`.
