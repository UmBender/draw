---
id: T157
title: macroquad Gpu interpreter
status: todo
wave: 68
branch: task/T157-macroquad-gpu-interpreter
depends_on: [T140]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T157 macroquad Gpu interpreter

## Goal

Slice S1 of [[T155 Gpu migration of the drawing]]: `src/shell/gpu_mq.rs`: buffers, textures, targets, pipelines and `submit` through `draw_mesh` and `RenderPass`, tested for chunk limits and handle checks. Nothing else in the shell changes.

## Known facts (2026-11-13)

- Split out of T155 after the batch blocked it (about 7,000 lines; every slice needs `e2e.sh --backend all`). The design and the slice list are in the T155 note; the traits are in [[ADR-T140-1 The Gpu and Platform traits]].
- No pixel change allowed.

## Spec

`MqGpu` implements `Gpu` over macroquad. It needs a GL context, so the logic
that can be tested without a window is separate and pure: a handle `Table`,
the geometry chunker and the pass checker. Nothing else in the shell uses
`MqGpu` yet (S2 onwards), so there is no pixel change.

- **AC-1** — Geometry is split into chunks that fit macroquad's `u16` indices and
  `render::BATCH_MAX_VERTICES` / `BATCH_MAX_INDICES`, keeping triangles whole
  and the vertices in order. Tests: `chunks_of_small_geometry_is_one_chunk`,
  `chunks_split_at_the_index_limit`, `chunks_split_at_the_vertex_limit`,
  `chunks_remap_indices_to_local_vertices`, `chunks_of_empty_geometry_is_empty`,
  `chunks_reject_index_out_of_range`, `chunks_reject_partial_triangle`,
  plus proptest `chunks_cover_every_triangle_within_limits`.
- **AC-2** — Handles are checked: `table_get_unknown_id_is_unknown_handle`,
  `table_get_after_insert_returns_value`, `table_ids_are_unique`.
- **AC-3** — Pass structure is checked: `passes_draw_outside_pass_is_error`,
  `passes_nested_begin_is_error`, `passes_end_without_begin_is_error`,
  `passes_unfinished_list_is_error`, `passes_balanced_list_is_ok`.
- **AC-4** — `MqGpu` implements `Gpu` (buffers, textures, targets, pipelines,
  `submit` through `draw_mesh` and `RenderPass`); it compiles and passes
  clippy. `Blend::Opaque` is rejected with `GpuError::Backend` (no caller
  uses it yet). Check: `cargo build`.
- **AC-5** — `scripts/check.sh` green and no other shell file changes, so no
  pixel change (the `e2e.sh --backend all` run is for S2 onwards, where
  drawing changes).

## Out of scope

- Other slices of T155; input, the loop and `main` ([[T156 Platform migration of input and the loop]]); anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `src/shell/gpu_mq.rs`, `src/shell/mod.rs` (one line)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T157): specify the slice`
- [ ] tests — `test(T157): add failing tests`
- [ ] models — `feat(T157): add the types`
- [ ] behaviour — `refactor(T157): migrate the slice`
- [ ] quality — `chore(T157): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T155 by the owner, following the batch's recommendation.
