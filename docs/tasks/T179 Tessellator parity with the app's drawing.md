---
id: T179
title: Tessellator parity with the app's drawing
status: in-progress
wave: 72
branch: task/T179-tessellator-parity-with-the-app-s-draw
depends_on: [T146, T173]
adrs: [ADR-T179-1, ADR-T179-2]
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

ADRs: [[ADR-T179-1 Zoom bucket per chunk for screen-space rules]] (the per-chunk rule), [[ADR-T179-2 Parity metric against the screen-space drawing]] (the tolerance).

- **AC-1** — Feather strips: strokes, lines and outlines carry the feather geometry and `uv` that `Blend::AlphaFeathered` reads. Tests (`crates/draw-gfx/src/tessellate.rs`): `line_edge_vertices_carry_feather_distance`, `feathered_edge_reaches_half_a_pixel_outside`.
- **AC-2** — Minimum screen width, joint and circle segment counts follow the zoom through a per-chunk bucket, with no re-tessellation on pan. Tests: `thin_line_is_one_pixel_wide_at_any_zoom`, `ellipse_segments_grow_with_zoom`, `zoom_bucket_rounds_to_powers_of_two`, and in `crates/draw-gfx/tests/chunks.rs`: `pan_keeps_chunks_and_bucket_change_rebuilds_them`.
- **AC-3** — Rect and cell fills and arrow heads are tessellated (heads at the segment end like `draw_shape`); the dot grid is `dot_grid_mesh`. Tests: `arrow_shaft_stops_at_the_head_base`, `dot_grid_mesh_has_one_quad_per_dot`.
- **AC-4** — Parity (`tests/tessellator_parity.rs`, one test per kind: `parity_stroke`, `parity_pressure_stroke`, `parity_line`, `parity_arrow`, `parity_rect`, `parity_ellipse`, `parity_grid`, each at zoom 0.25, 1 and 4): both paths on headless EGL, the metric of ADR-T179-2. The proptest `cached_geometry_equals_recomputed` of T146 stays green.

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
