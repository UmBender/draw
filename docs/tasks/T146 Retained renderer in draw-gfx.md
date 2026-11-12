---
id: T146
title: Retained renderer in draw-gfx
status: todo
wave: 71
branch: task/T146-retained-renderer-in-draw-gfx
depends_on: [T145, T135, T136, T153]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T146 Retained renderer in draw-gfx

## Goal

Shapes are tessellated once, kept in GPU buffers per chunk and drawn with the camera in the vertex shader.

## Known facts (2026-11-12)

- [[2.0 Architecture proposal]] §6. Removes the document layer re-render, the settle re-render and the blit of [[T26 Pan and zoom without re-rendering]].
- The risk is cache invalidation, so every cached geometry is compared with a recomputed one.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Chunks of shapes keyed by shape id and revision; an edit rebuilds only the chunks it touches.
  Tests: `edit_rebuilds_only_its_chunk`, proptest `cached_geometry_equals_recomputed` over edit sequences.
- **AC-2** — Pan and zoom change one uniform and re-tessellate nothing; text, formulas, grids and the minimap draw through the same path.
  Tests: draw-call and vertex counts on the recorder; the formula and e2e goldens stay equal within the pixel tolerance.
- **AC-3** — Anti-aliasing is analytic in the fragment shader; MSAA stays as a fallback chosen by measurement.
  Pixel tests on lines, circles and thin strokes at several zooms.
- **AC-4** — The ADR-T24-5 budgets are met on the target (overlay under 2 ms, full frame under 16 ms at any zoom) and recorded in the log.
  Manual measurement with the frame timer; the tessellation time is also a benchmark in `tests/perf.rs`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-gfx/src/**`, `tests/perf.rs`
- this note and `ADR-T146-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T146): specify the retained renderer`
- [ ] tests — `test(T146): add failing tests for the chunks`
- [ ] models — `feat(T146): add the chunk and geometry types`
- [ ] behaviour — `feat(T146): implement the retained renderer`
- [ ] quality — `chore(T146): pass clippy and rustfmt`
- [ ] docs — `docs(T146): record the frame times`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
