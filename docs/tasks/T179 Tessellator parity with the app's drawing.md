---
id: T179
title: Tessellator parity with the app's drawing
status: review
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

- [x] spec — `docs(T179): specify the parity`
- [x] tests — `test(T179): add failing parity tests`
- [x] behaviour — `feat(T179): tessellate feather, fills, heads and the grid`
- [x] quality — `chore(T179): pass clippy and rustfmt`
- [x] docs — `docs(T179): record the parity results`

## Log

- 2026-11-16 — done (batch). The tests step did not compile before the models step (new `tessellate_at`, `zoom_bucket`, `dot_grid_mesh`, `ChunkCache::set_zoom` signatures), as allowed. `scripts/check.sh` green; the e2e suite was not run: nothing in `src/` changed and the app does not call the tessellator.
- Results: `tessellate_at(shape, zoom_bucket)` feathers strokes, lines, arrows, rect and ellipse outlines and fills, grid lines and cell fills (`uv[0]` is the edge distance, half a pixel out and in); the stroke follows `draw_shape`'s mitred strip, minimum step, mitre limit and joint rule. `parity_*` pass at zoom 0.25, 1 and 4 for stroke, pressure stroke, line, arrow, rect and ellipse (open and filled) and grid with fills, with the metric of [[ADR-T179-2 Parity metric against the screen-space drawing]] (reference on a 4× multisampled target).
- Deviations: the dot grid is `dot_grid_mesh` (plain quads for the `Alpha` pipeline, not feathered, as `draw_underlay` draws it); flat stroke ends and thin-line corners are hard, not feathered; the parity metric tolerates differences only within one pixel of a reference edge plus 4 % of ink (ADR-T179-2), not the 64-pixel e2e count, which a feathered edge would exceed by design.
- LEFTOVER: parity is tested at bucket zooms only; between buckets the minimum width is off by up to a factor of √2 for thin lines (ADR-T179-1). Text, labels and grid axis labels are T180. T172 may tighten the metric once the app draws through this path.

- 2026-11-16 — created by the owner from the split of [[T172 Retained renderer in the app]] proposed in its Log.
