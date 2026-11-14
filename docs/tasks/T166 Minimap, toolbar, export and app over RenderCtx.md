---
id: T166
title: Minimap, toolbar, export and app over RenderCtx
status: todo
wave: 68
branch: task/T166-minimap-toolbar-export-and-app-over-rend
depends_on: [T165]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T166 Minimap, toolbar, export and app over RenderCtx

## Goal

Slice 2 of [[T164 Generic run over Platform and Gpu]]: the minimap, the UI shapes, the toolbar, export, `CachedFrame`/`blit*` and the `app.rs` resource calls take the `RenderCtx` the loop owns; the bridge `thread_local!` of [[T165 RenderCtx over Gpu for the drawing and UI text]] goes away; the drawing reads the window size and the DPI from `Platform::window()` instead of macroquad.

## Known facts (2026-11-13)

- Split out of [[T164 Generic run over Platform and Gpu]] after the batch blocked it (findings in the T164 Log).
- After T165 the callers in `minimap.rs` (7 uses), `ui_shapes.rs` (12), `toolbar.rs`, `export.rs`, `app.rs` (8 `with_gpu` calls) and `tests/perf.rs` reach the `RenderCtx<MqGpu>` through the bridge.
- `render.rs` calls macroquad `screen_width`/`screen_height`/`screen_dpi_scale` ~25 times, `app.rs` 9 times, `ui_text.rs` reads `miniquad::window::dpi_scale`. `Platform::window()` already returns `WindowMetrics` (ADR-T140-1), so the trait needs no change.
- `app::run_on<P: Platform>` (T163) owns the loop; it creates the `RenderCtx<MqGpu>` here (still a concrete `G` until T167).
- `vendor/miniquad` is not changed. No pixel change allowed.

## Spec

Draft: the spec step refines each criterion into named tests.

- **AC-1** — No `thread_local!` in the shell holds the `Gpu`, its resources or a `RenderCtx`; everything that draws takes it from the loop. Test: `shell_sources_have_no_gpu_thread_local` (source scan for `thread_local!` next to `Gpu`, `GPU`, `TEXTURES`, `SCOPE`, `PIPELINE`, `RenderCtx`, `with_gpu`, `render::submit`).
- **AC-2** — `render.rs`, `ui_text.rs` and `app.rs` call no macroquad window size or DPI function; the drawing gets them from `WindowMetrics`. Tests: `drawing_reads_the_window_size_from_the_platform` (source scan) and a frame over `RecordingPlatform` with chosen metrics that lays out at that size.
- **AC-3** — `e2e.sh --backend all` green; no pixel change; `tests/perf.rs` builds and runs.

## Out of scope

- `run<P, G>`, the `match` in `main` and the widened source scan ([[T167 Generic run and the match in main]]).

## Files owned

- `src/shell/minimap.rs`, `src/shell/ui_shapes.rs`, `src/shell/toolbar.rs`, `src/shell/export.rs`, `src/shell/app.rs`, `src/shell/render.rs`, `src/shell/ui_text.rs`, `src/shell/render_ctx.rs` (if T165 made it), `src/shell/sheet.rs`, `src/shell/command_palette.rs`, `src/shell/mod.rs`
- the other `src/shell/` call sites and `tests/` (including `tests/perf.rs`)
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T166): specify the slice`
- [ ] tests — `test(T166): add failing tests`
- [ ] models — `feat(T166): add the types`
- [ ] behaviour — `refactor(T166): migrate the slice`
- [ ] quality — `chore(T166): pass clippy and rustfmt`

## Log

- 2026-11-14 — split from T164 by the owner, following the batch's recommendation.
