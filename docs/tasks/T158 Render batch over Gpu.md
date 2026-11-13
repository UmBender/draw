---
id: T158
title: Render batch over Gpu
status: todo
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

Draft: the spec step refines each criterion into named tests.

- **AC-1** — The slice's code emits `Gpu` calls only, as described in the goal.
- **AC-2** — `e2e.sh --backend all` green; no pixel change in the screenshot comparisons.

## Out of scope

- Other slices of T155; input, the loop and `main` ([[T156 Platform migration of input and the loop]]); anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `src/shell/render.rs`, `src/shell/glyphs.rs`
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
