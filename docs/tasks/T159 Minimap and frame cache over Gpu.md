---
id: T159
title: Minimap and frame cache over Gpu
status: todo
wave: 68
branch: task/T159-minimap-and-frame-cache-over-gpu
depends_on: [T158]
adrs: []
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

- **AC-1** — The slice's code emits `Gpu` calls only, as described in the goal.
- **AC-2** — `e2e.sh --backend all` green; no pixel change in the screenshot comparisons.

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

- [ ] spec — `docs(T159): specify the slice`
- [ ] tests — `test(T159): add failing tests`
- [ ] models — `feat(T159): add the types`
- [ ] behaviour — `refactor(T159): migrate the slice`
- [ ] quality — `chore(T159): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T155 by the owner, following the batch's recommendation.
- 2026-11-13 — BLOCKED (batch): the slice needs three things outside *Files owned*, so I changed no code. (1) The `Gpu` trait (`crates/draw-gfx/src/gpu.rs`) has no way to set a texture's filter after creation, and the frame cache switches the document layer between `Nearest` and `Linear` per frame (`blit_filter`, `document.texture.set_filter`) and sets the overview target to `Linear`; `TargetDesc` has no filter either. (2) The trait has no texture readback, and `read_view` / the export path use `get_texture_data` on the layer. (3) The one `MqGpu` and its pipeline live in private `thread_local`s of `render.rs` (`GPU`, `PIPELINE`, `TEXTURES`), and `gpu_mq.rs` still leaves the camera to the caller for `BeginPass { target: None }` (`pass_owns_camera`, "until T159"); `minimap.rs` and `app.rs` cannot reach the interpreter, make a target pass or blit a target texture without editing `render.rs` and `gpu_mq.rs`. Recommended fix: widen *Files owned* to `crates/draw-gfx/src/gpu.rs`, `src/shell/gpu_mq.rs` and `src/shell/render.rs` (an ADR amending ADR-T140-1 for `set_filter` / `TargetDesc.filter` and a readback call, plus a shared accessor for the interpreter), then re-run the batch for T159.
- 2026-11-13 — UNBLOCKED by the owner: widen *Files owned* as the blocker recommended. Filter: `set_filter` method plus `TargetDesc.filter` (not a per-`Draw` filter, so T158's draws stay as they are). Readback: `read_texture` in the `Gpu` trait (not an `MqGpu`-only method). Record both in an ADR amending ADR-T140-1.
