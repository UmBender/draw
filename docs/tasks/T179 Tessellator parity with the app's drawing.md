---
id: T179
title: Tessellator parity with the app's drawing
status: todo
wave: 72
branch: task/T179-tessellator-parity-with-the-app-s-draw
depends_on: [T146, T173]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T179 Tessellator parity with the app's drawing

## Goal

`draw_gfx::tessellate` produces, for every shape kind, geometry whose pixels match the app's screen-space `draw_shape` (`src/shell/render.rs`) within the e2e tolerance, so that [[T172 Retained renderer in the app]] can swap the path without moving the goldens.

## Known facts (2026-11-16)

- Split (a) of T172 (its Log, 2026-11-15): the tessellator is world space with no feather geometry (T173 left `uv` zero), no minimum screen width, no fills, no arrow heads and no dot grid, while `draw_shape` works in screen space (minimum widths, joint and circle segment counts by pixel radius).
- The app does not use the tessellator yet: nothing the user sees changes in this task.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Feather strips: strokes and outlines carry the feather geometry and `uv` that `Blend::AlphaFeathered` of T173 reads.
- **AC-2** — Minimum screen width, joint and circle segment counts follow the zoom through the camera uniform or a per-chunk rule (an ADR chooses), with no re-tessellation on pan.
- **AC-3** — Rect and cell fills, arrow heads and the dot grid are tessellated.
- **AC-4** — Parity test: for each shape kind at zoom 0.25, 1 and 4, the tessellated geometry rendered on headless EGL equals `draw_shape` rendered through the same `Gpu` within the e2e pixel tolerance. The proptest `cached_geometry_equals_recomputed` of T146 stays green.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-gfx/src/**`, `crates/draw-gfx/tests/**`, `tests/perf.rs`, a new parity test under `tests/`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T179): specify the parity`
- [ ] tests — `test(T179): add failing parity tests`
- [ ] behaviour — `feat(T179): tessellate feather, fills, heads and the grid`
- [ ] quality — `chore(T179): pass clippy and rustfmt`
- [ ] docs — `docs(T179): record the parity results`

## Log

- 2026-11-16 — created by the owner from the split of [[T172 Retained renderer in the app]] proposed in its Log.
