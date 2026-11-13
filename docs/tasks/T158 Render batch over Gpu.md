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
