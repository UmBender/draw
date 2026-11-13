---
id: T155
title: Gpu migration of the drawing
status: in-progress
wave: 68
branch: task/T155-gpu-migration-of-the-drawing
depends_on: [T140]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T155 Gpu migration of the drawing

## Goal

Everything the shell draws goes through the `Gpu` trait, with macroquad as one interpreter of it. Input and the loop are untouched.

## Known facts (2026-11-13)

- Split out of [[T140 Platform and Gpu traits with recording interpreters]] AC-2 after the batch blocked it. The traits, the recorder and [[ADR-T140-1 The Gpu and Platform traits]] already exist.
- What moves: `render.rs` `Batch`/`MeshSink` become `Command::Draw`; `glyphs.rs` and `minimap.rs` textures and the render target in `app.rs` become `Gpu` handles; `ui_text.rs` `draw_text_ex` becomes glyph quads; `toolbar`, `sheet`, `command_palette` and `ui_shapes` primitives become tessellated triangles.
- Pixels can be checked without a tablet (`e2e.sh`, screenshots), so this part can run unattended.

## Spec

Refined after reading the code (2026-11-13). Findings that shape the work:

- `render.rs` already has a seam: `Batch<S: MeshSink>` and `push_text_*` take a sink. But `Batch`, every `draw_*` helper and the sink tests use macroquad's `Mesh`, `Vertex` and `Color`, and glyph pages live in a `thread_local` `TEXTURES` of macroquad `Texture2D`.
- `draw_guides`, `draw_selection` and the toolbar, minimap, sheet and palette still call `draw_line`, `draw_rectangle`, `draw_texture_ex` directly.
- `app.rs` owns the MSAA frame cache (`render_target_ex`, `set_camera`, `blit`).
- `ui_text.rs` uses `draw_text_ex` and `measure_text` on a macroquad `Font`: glyph quads need the glyph atlas to rasterize the UI font (a metrics change would break pixel parity).

Slices (each ends with `e2e.sh --backend all` green; no pixel change):

1. **S1** — macroquad `Gpu` interpreter (`src/shell/gpu_mq.rs`: buffers, textures, targets, pipelines, `submit` through `draw_mesh` and `RenderPass`), tested for chunk limits and handle checks.
2. **S2** — `render.rs`: `Batch` emits `Vertex` and `Command::Draw`; `GlyphTextures` holds `Gpu` texture handles; `draw_guides` and `draw_selection*` become tessellated quads.
3. **S3** — `minimap.rs` and the frame cache in `app.rs` (render target, blit) on `Gpu` handles.
4. **S4** — `toolbar`, `sheet`, `command_palette`, `ui_shapes` primitives tessellated.
5. **S5** — `ui_text.rs`: UI text as glyph quads from the shared atlas.

- **AC-1** — The drawing path (render, glyphs, minimap, ui, frame cache) emits `Gpu` calls only; no macroquad drawing call remains outside the macroquad `Gpu` interpreter. Check: `grep` for `draw_mesh|draw_line|draw_rectangle|draw_texture_ex|draw_text_ex|render_target_ex|set_camera` finds only `gpu_mq.rs` (test `no_macroquad_draw_calls_outside_the_interpreter`, a source scan).
- **AC-2** — A test draws a scene over `RecordingGpu` and asserts the command list: `scene_over_recording_gpu_emits_begin_draws_end` (a rectangle, a stroke and a text block: one pass, draws with the white-block UVs, glyph draws bound to the atlas texture), `batch_over_recording_gpu_splits_chunks_at_the_limits`.
- **AC-3** — `e2e.sh --backend all` green; no pixel change in the screenshot comparisons.

## Out of scope

- Input, the loop and `main` (see [[T156 Platform migration of input and the loop]]); anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `src/shell/render.rs`, `glyphs.rs`, `minimap.rs`, `ui_text.rs`, `toolbar`, `sheet`, `command_palette`, `ui_shapes`, the drawing part of `app.rs`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T155): specify the Gpu migration of the drawing`
- [ ] tests — `test(T155): add failing tests for the drawing over the recorder`
- [ ] models — `feat(T155): add the macroquad Gpu interpreter`
- [ ] behaviour — `refactor(T155): draw through the Gpu trait`
- [ ] quality — `chore(T155): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T140 AC-2 by the owner, following the batch's recommendation.
- 2026-11-13 — spec committed (slices S1–S5 in *Spec*).
- 2026-11-13 — BLOCKED (batch): the migration touches about 7 000 lines (`render.rs` 2 800, `toolbar.rs` 1 200, `glyphs.rs` 1 100, `app.rs` frame cache, minimap, sheet, palette, UI text) and every `draw_*` helper and sink test is typed on macroquad's `Mesh`/`Color`. A rewrite that keeps pixel parity needs an `e2e.sh --backend all` run after each slice, which one unattended session cannot do without leaving the tree half-migrated. I stopped after the spec rather than commit untestable partial work. Recommended fix: split T155 into one task per slice S1–S5 (S1 first: the macroquad `Gpu` interpreter is independent and testable), then re-run the batch with one slice per task.
