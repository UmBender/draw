---
id: T146
title: Retained renderer in draw-gfx
status: todo
wave: 71
branch: task/T146-retained-renderer-in-draw-gfx
depends_on: [T145, T135, T136, T153, T161]
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
- **AC-2** — Pan and zoom change one uniform and re-tessellate nothing; the renderer has the primitives text, formulas, grids and the minimap need.
  Tests: draw-call and vertex counts on the recording `Gpu`. The app path and the e2e goldens are [[T172 Retained renderer in the app]].
- **AC-3** — Anti-aliasing is analytic in the fragment shader; MSAA stays as a fallback chosen by measurement.
  Pixel tests on lines, circles and thin strokes at several zooms.
- **AC-4** — The tessellation time is a benchmark in `tests/perf.rs`, recorded in the Log. The ADR-T24-5 budgets in the app are [[T172 Retained renderer in the app]].

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.
- Wiring the renderer into `src/shell`, the e2e goldens and the frame budgets in the app ([[T172 Retained renderer in the app]]).

## Files owned

- `crates/draw-gfx/src/**`, `tests/perf.rs`
- this note and `ADR-T146-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T146): specify the retained renderer`
- [ ] tests — `test(T146): add failing tests for the chunks`
- [ ] models — `feat(T146): add the chunk and geometry types`
- [ ] behaviour — `feat(T146): implement the retained renderer`
- [ ] quality — `chore(T146): pass clippy and rustfmt`
- [ ] docs — `docs(T146): record the tessellation benchmark`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-15 — scoped by the owner to `draw-gfx`: tested on the recording `Gpu` and headless GL; the app wiring, the e2e goldens and the frame budgets (AC-2 app part, AC-4 budgets) moved to [[T172 Retained renderer in the app]].
