---
id: T165
title: RenderCtx over Gpu for the drawing and UI text
status: todo
wave: 68
branch: task/T165-renderctx-over-gpu-for-the-drawing-and-u
depends_on: [T163]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T165 RenderCtx over Gpu for the drawing and UI text

## Goal

Slice 1 of [[T164 Generic run over Platform and Gpu]]: a `RenderCtx<G: Gpu>` owns what the `render.rs` and `ui_text.rs` thread-locals hold today (the `Gpu`, the glyph page textures, the pipeline, the open `render_to` scope, the menu glyph textures), and `render.rs` and `ui_text.rs` draw through `&mut RenderCtx<G>`.

## Known facts (2026-11-13)

- Split out of [[T164 Generic run over Platform and Gpu]] after the batch blocked it (too large for one session; findings in the T164 Log). The traits are in [[ADR-T140-1 The Gpu and Platform traits]]; the filter and readback calls in [[ADR-T159-1 Texture filter and readback in the Gpu trait]].
- `render.rs` holds `TEXTURES`, `GPU`, `PIPELINE` and `SCOPE` in a `thread_local!` (next to `ATLAS` and `STATS`, which hold no GPU state); `ui_text.rs` holds `GLYPHS` (with `GlyphTextures<MqGpu>`) and `FONT`. The free functions `draw_inline`, `GlSink`, `render_to`, `submit` and ~90 uses of `Batch`/`MeshSink` reach them.
- How the slice compiles (owner decision, batch plan): the callers outside `render.rs` and `ui_text.rs` (`minimap.rs`, `ui_shapes.rs`, `toolbar.rs`, `export.rs`, `app.rs`, `tests/perf.rs`) keep working through **one** temporary bridge, a single `thread_local!` holding a `RenderCtx<MqGpu>` and one accessor, documented as removed by [[T166 Minimap, toolbar, export and app over RenderCtx]]. Those callers change only as much as the bridge needs.
- `RecordingGpu` (in `src/shell/recording.rs`) is the test interpreter.
- `vendor/miniquad` is not changed. No pixel change allowed. The window size and DPI reads stay as they are (T166).

## Spec

Draft: the spec step refines each criterion into named tests.

- **AC-1** — `RenderCtx<G: Gpu>` owns the `Gpu`, the glyph page textures, the pipeline, the scope and the menu glyph textures; the drawing in `render.rs` and `ui_text.rs` takes `&mut RenderCtx<G>` (or is its methods). Test: a stroke, a text block and a UI label drawn through `RenderCtx<RecordingGpu>` give the expected `Gpu` calls.
- **AC-2** — `render.rs` and `ui_text.rs` have no `thread_local!` holding the `Gpu` or its resources; the only one left in the shell is the bridge. Test: `render_and_ui_text_hold_no_gpu_thread_local` (source scan).
- **AC-3** — `e2e.sh --backend all` green; no pixel change.

## Out of scope

- Removing the bridge, the window size through `Platform` ([[T166 Minimap, toolbar, export and app over RenderCtx]]); `run<P, G>` and the `match` in `main` ([[T167 Generic run and the match in main]]).

## Files owned

- `src/shell/render.rs`, `src/shell/ui_text.rs`, `src/shell/glyphs.rs`, `src/shell/gpu_mq.rs`, `src/shell/recording.rs`, `src/shell/mod.rs`, a new `src/shell/render_ctx.rs` if the session prefers it
- the call sites in other `src/shell/` files and in `tests/` (including `tests/perf.rs`), only to go through the bridge
- the `Gpu` trait only through an ADR amending ADR-T140-1 if a call is missing
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T165): specify the slice`
- [ ] tests — `test(T165): add failing tests`
- [ ] models — `feat(T165): add the types`
- [ ] behaviour — `refactor(T165): migrate the slice`
- [ ] quality — `chore(T165): pass clippy and rustfmt`

## Log

- 2026-11-14 — split from T164 by the owner, following the batch's recommendation.
