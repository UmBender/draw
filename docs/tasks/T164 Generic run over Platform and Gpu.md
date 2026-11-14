---
id: T164
title: Generic run over Platform and Gpu
status: done
wave: 68
branch: task/T164-generic-run-over-platform-and-gpu
depends_on: [T165, T166, T167]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T164 Generic run over Platform and Gpu

## Goal

Slice 3 of [[T156 Platform migration of input and the loop]]: the `MqGpu` thread-local goes away; `CachedFrame`, the toolbar, the minimap, export and `ui_text` take a `G: Gpu`; `fn run<P: Platform, G: Gpu>()` and `main` picks the interpreters with a `match`.

## Known facts (2026-11-13)

- Split out of [[T156 Platform migration of input and the loop]] after the batch blocked it (too large for one session; findings in the T156 Log). The traits are in [[ADR-T140-1 The Gpu and Platform traits]]; the filter and readback calls in [[ADR-T159-1 Texture filter and readback in the Gpu trait]].
- `vendor/miniquad` is not changed. No pixel change allowed.
- T155/T157–T161 wrapped the drawing in `wrap_pass::<MqGpu>` but kept the thread-local (`render::with_gpu`, `render::submit`, `ui_text`, `CachedFrame`, `blit*`).
- AC-4 of T156 (the tablet by hand) is checked at the end of this task, by the owner.

## Spec

Draft: the spec step refines each criterion into named tests.

- **AC-1** — No thread-local holds the `Gpu`; everything that draws takes `&mut G` (or a context that owns it). Test: `shell_sources_have_no_gpu_thread_local` (source scan for `thread_local!` next to `Gpu`, `GPU`, `TEXTURES`, `SCOPE`, `with_gpu`, `render::submit`).
- **AC-2** — `main` monomorphises over `P` and `G` with a `match`; a source scan finds no miniquad or macroquad type outside `platform_mq.rs` and `gpu_mq.rs` (the T161 scan widened). Test: `shell_sources_outside_the_mq_interpreters_name_no_macroquad` and `main_runs_generic_run_through_a_match`.
- **AC-3** — `e2e.sh --backend all` green; no pixel change.
- **AC-4** — Hand check with a tablet (pen pressure, tilt, eraser, seat changes), recorded in the Log by the owner (T156 AC-4).

## Out of scope

- New backends ([[T143 Wayland backend with window, keyboard, pointer, tablet and clipboard]], [[T144 X11 backend with window, keyboard, pointer, XInput2 pressure and selections]]); the retained renderer ([[T146 Retained renderer in draw-gfx]]).

## Files owned

- `src/main.rs`, `src/shell/app.rs`, `src/shell/render.rs`, `src/shell/gpu_mq.rs`, `src/shell/glyphs.rs`, `src/shell/minimap.rs`, `src/shell/toolbar.rs`, `src/shell/ui_text.rs`, `src/shell/export.rs`, `src/shell/mod.rs`
- the other `src/shell/` call sites and `tests/` (including `tests/perf.rs`)
- the `Gpu` trait only through an ADR amending ADR-T140-1 if a call is missing
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T164): specify the slice`
- [x] tests — moved to the slices
- [x] models — moved to the slices
- [x] behaviour — moved to the slices
- [x] quality — moved to the slices

## Log

- 2026-11-13 — split from T156 by the owner, following the batch's recommendation.
- 2026-11-13 — BLOCKED (batch): too large for one unattended session. The `Gpu` thread-locals (`GPU`, `TEXTURES`, `SCOPE`, `PIPELINE` in `render.rs`; `ui_text.rs`; 8 `with_gpu` calls in `app.rs`) are reached through free functions (`draw_inline`, `GlSink`, `render_to`, ~90 uses of `Batch`/`MeshSink` in `render.rs`, 12 in `ui_shapes.rs`, 7 in `minimap.rs`), and the drawing still calls macroquad `window_width`/`screen_height`. Making all of it take `&mut G` is a signature change through the whole shell with no compiling intermediate stub. Only the spec step is committed. Recommended fix: split into T164a (a `RenderCtx<G>` that owns the `Gpu`, textures, pipeline and scope; migrate `render.rs` and `ui_text.rs`), T164b (`minimap`, `toolbar`, `export`, `app.rs` resource calls and the window size via `Platform`), T164c (generic `run<P, G>`, `main` `match`, widened source scan, AC-4 by hand), each in its own session.
- 2026-11-14 — closed by the owner: split into [[T165 RenderCtx over Gpu for the drawing and UI text]] (the `RenderCtx`, `render.rs`, `ui_text.rs`), [[T166 Minimap, toolbar, export and app over RenderCtx]] (AC-1, the window size through `Platform`) and [[T167 Generic run and the match in main]] (AC-2 with an allowlist that T149 empties, AC-3, and AC-4 by hand at its end).
