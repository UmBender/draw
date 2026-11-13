---
id: T159
title: Minimap and frame cache over Gpu
status: review
wave: 68
branch: task/T159-minimap-and-frame-cache-over-gpu
depends_on: [T158]
adrs: [ADR-T159-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T159 Minimap and frame cache over Gpu

## Goal

Slice S3 of [[T155 Gpu migration of the drawing]]: `minimap.rs` and the MSAA frame cache in `app.rs` (render target, blit) on `Gpu` handles.

## Known facts (2026-11-13)

- Split out of T155 after the batch blocked it (about 7,000 lines; every slice needs `e2e.sh --backend all`). The design and the slice list are in the T155 note; the traits are in [[ADR-T140-1 The Gpu and Platform traits]].
- No pixel change allowed.

## Spec

Draft: the spec step refines each criterion into named tests.

Design in [[ADR-T159-1 Texture filter and readback in the Gpu trait]].

- **AC-1** — `minimap.rs` and the frame cache (document layer, overview, selection image, blit) emit `Gpu` calls only: no `draw_rectangle`, `draw_texture_ex`, `render_target_ex`, `set_camera`, `get_texture_data` or `set_filter` on a macroquad texture remains in them. Checked by `minimap_and_frame_cache_use_no_macroquad_draw_calls` (source scan of the two files, outside `mod tests`).
- **AC-2** — The `Gpu` trait has `TargetDesc.filter`, `set_filter` and `read_texture`: `recording_gpu_records_set_filter_and_read_texture`, `recording_gpu_read_texture_returns_one_rgba_image`, `recording_gpu_set_filter_unknown_handle_fails`, `null_gpu_read_texture_returns_zeroed_pixels`.
- **AC-3** — The blit and the minimap panel are tessellated quads: `texture_quad_unflipped_maps_top_left_to_origin`, `texture_quad_flipped_swaps_the_rows`, `border_bars_cover_the_rect_edges`.
- **AC-4** — A scope of draws into a target becomes one pass: `scope_list_wraps_draws_in_one_pass_with_clear`.
- **AC-5** — `BeginPass { target: None }` owns the camera: `every_pass_owns_the_camera`.
- **AC-6** — `e2e.sh --backend all` green; no pixel change in the screenshot comparisons.

## Out of scope

- Other slices of T155; input, the loop and `main` ([[T156 Platform migration of input and the loop]]); anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `src/shell/minimap.rs`, the frame cache part of `src/shell/app.rs`
- `crates/draw-gfx/src/gpu.rs`: `Gpu::set_filter(&mut self, texture, Filter)`, a `filter` field in `TargetDesc`, and `Gpu::read_texture(&mut self, texture) -> Result<Vec<u8>, GpuError>` (readback for export and `read_view`), with a new ADR amending ADR-T140-1
- `src/shell/gpu_mq.rs`: implement the new calls; restore the camera reset in `BeginPass { target: None }` (removed in T158)
- `src/shell/render.rs`: a shared accessor for the one `MqGpu`, its pipeline and textures, so `minimap.rs` and `app.rs` reach the interpreter
- the ADR above and `crates/draw-gfx` tests for the new calls (`RecordingGpu` / test doubles included)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T159): specify the slice`
- [x] tests — `test(T159): add failing tests`
- [x] models — `feat(T159): add the types` (committed as "add the filter and readback calls to Gpu")
- [x] behaviour — `refactor(T159): migrate the slice`
- [x] quality — `chore(T159): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T155 by the owner, following the batch's recommendation.
- 2026-11-13 — BLOCKED (batch): the slice needs three things outside *Files owned*, so I changed no code. (1) The `Gpu` trait (`crates/draw-gfx/src/gpu.rs`) has no way to set a texture's filter after creation, and the frame cache switches the document layer between `Nearest` and `Linear` per frame (`blit_filter`, `document.texture.set_filter`) and sets the overview target to `Linear`; `TargetDesc` has no filter either. (2) The trait has no texture readback, and `read_view` / the export path use `get_texture_data` on the layer. (3) The one `MqGpu` and its pipeline live in private `thread_local`s of `render.rs` (`GPU`, `PIPELINE`, `TEXTURES`), and `gpu_mq.rs` still leaves the camera to the caller for `BeginPass { target: None }` (`pass_owns_camera`, "until T159"); `minimap.rs` and `app.rs` cannot reach the interpreter, make a target pass or blit a target texture without editing `render.rs` and `gpu_mq.rs`. Recommended fix: widen *Files owned* to `crates/draw-gfx/src/gpu.rs`, `src/shell/gpu_mq.rs` and `src/shell/render.rs` (an ADR amending ADR-T140-1 for `set_filter` / `TargetDesc.filter` and a readback call, plus a shared accessor for the interpreter), then re-run the batch for T159.
- 2026-11-13 — UNBLOCKED by the owner: widen *Files owned* as the blocker recommended. Filter: `set_filter` method plus `TargetDesc.filter` (not a per-`Draw` filter, so T158's draws stay as they are). Readback: `read_texture` in the `Gpu` trait (not an `MqGpu`-only method). Record both in an ADR amending ADR-T140-1.
- 2026-11-13 — done, in review. [[ADR-T159-1 Texture filter and readback in the Gpu trait]] added `TargetDesc.filter`, `Gpu::set_filter` and `Gpu::read_texture` (recording and null doubles included). Draws into a target are collected per `render::render_to` scope and submitted as one pass (one MSAA resolve, not one per batch chunk); `BeginPass { target: None }` sets the default camera again. `app.rs` and `minimap.rs` call no macroquad draw, camera, target or readback function (overlay marquee/caret still use `draw_rectangle`/`draw_line`: T160). `scripts/check.sh` green; `e2e.sh --backend all` green on both backends. Deviations: the models step is merged with the trait implementations (the trait change cannot compile half-done); `every_pass_owns_the_camera` tests a constant helper, the real check is e2e. The e2e cases have no screenshot of the minimap panel, so its pixels were not compared: look at it once by hand (pan a drawing with the minimap shown, and a zoomed-out view with the layer blit).
