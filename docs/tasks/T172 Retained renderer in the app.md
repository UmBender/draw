---
id: T172
title: Retained renderer in the app
status: review
wave: 72
branch: task/T172-retained-renderer-in-the-app
depends_on: [T146, T173, T179, T180]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T172 Retained renderer in the app

## Goal

The app draws through the retained renderer of [[T146 Retained renderer in draw-gfx]]: shapes, text, formulas, grids and the minimap, with the camera as a uniform, and the frame budgets are met on the target.

## Known facts (2026-11-15)

- Created by the owner when T146 was scoped to `draw-gfx` (2026-11-15): T146 AC-2's app part (text, formulas, grids and the minimap through the same path; e2e goldens) and T146 AC-4 (budgets on the target) moved here.
- Removes the document layer re-render, the settle re-render and the blit of [[T26 Pan and zoom without re-rendering]].

## Spec

Refined by the unattended night-3 run (UNATTENDED CHOICE: the slice below, see the Log). Default stays `layers`; `retained` ships behind `--renderer retained`.

- **AC-1a** — `shell::retained::RetainedScene` draws the visible chunks of the document's shapes (hidden ones skipped) in one pass with `SetCamera(origin, zoom)` and the feathered pipeline; pan re-tessellates nothing and zoom inside a bucket neither. Tests: `retained_draw_sets_camera_and_feathered_pipeline`, `retained_pan_tessellates_nothing`, `retained_zoom_in_bucket_tessellates_nothing`, `retained_edit_rebuilds_only_the_changed_chunk`, `retained_hidden_shape_is_not_drawn`.
- **AC-1b** — `--renderer layers|retained` is parsed into `Options::renderer` (default `layers`), a bad value is refused. Tests: `renderer_flag_*` in `src/shell/options.rs`.
- **AC-1c** — With `--renderer retained` the app frame draws the document through `RetainedScene` (no layer target, no settle, no blit). Not unit-testable without a window; covered by AC-2.
- **AC-1d (LEFTOVER)** — text, formulas, labels, the minimap and the grid tool on the retained path: not in this slice.

- **AC-1** — Shapes, text, formulas, grids and the minimap draw through the retained renderer; pan and zoom re-tessellate nothing. Tests: draw-call and vertex counts on the recorder.
- **AC-2** — The formula and e2e goldens stay equal within the pixel tolerance. `scripts/e2e.sh --backend all`.
- **AC-3** — The [[ADR-T24-5 Redraw budgets]] budgets are met on the target (overlay under 2 ms, full frame under 16 ms at any zoom), measured with `--frame-times` on the perf scenes and recorded in the Log. The measurement is the owner's, on the desktop GPU (headless sway renders with pixman); an unattended run stops at `review` with AC-1 and AC-2 done.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `src/shell/**` (the drawing path), `tests/perf.rs`, `tests/e2e*`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T172): specify the wiring slice`
- [x] tests — `test(T172): add failing tests for the retained scene and --renderer`
- [x] behaviour — `feat(T172): draw the document through retained chunks`, `refactor(T172): draw the app frame through the retained scene on request`
- [x] quality — fmt and clippy fixes are in the behaviour commit
- [ ] docs — AC-3 frame times: the owner's, on the desktop GPU

## Log

- 2026-11-15 — created by the owner from T146 AC-2 (app part) and AC-4.
- 2026-11-15 — BLOCKED (batch): the slice cannot reach a green tree with
  unchanged e2e pixels without more groundwork. Findings: (1) the app's
  `draw_shape` in `src/shell/render.rs` works in screen space (minimum
  screen widths, joint and circle segment counts by pixel radius, fills,
  arrow heads, dot grid), while `draw_gfx::tessellate` is world space with
  no feather geometry (T173 left uv zero), no minimum width and no text,
  so its pixels differ from the goldens; (2) text and formulas rasterise
  per zoom step (`text_raster`, `label_raster`), so they cannot be
  zoom-independent chunks without a design decision; (3) the minimap and
  the T26 layer cache are tied to the current layer path. Tried: read the
  render path and the tessellator; wrote no code. Recommended split, each
  slice green on its own: (a) tessellator parity (feather strips, minimum
  screen width through the camera uniform or a per-chunk rule, rect fills,
  arrow heads) with a golden-diff test against `draw_shape` on the
  recorder; (b) text and formulas on the retained path (an ADR on glyph
  rasters per zoom step: re-emit text quads only on a raster-step change);
  (c) wire `ChunkCache` into the app frame, minimap and grids, and remove
  the T26 layer re-render, settle and blit; AC-3 stays the owner's.
- 2026-11-16 — split by the owner after the batch blocker (recorded on the task branch): tessellator parity went to [[T179 Tessellator parity with the app's drawing]], retained text and formulas to [[T180 Text and formulas on the retained path]]; T172 keeps the wiring (`ChunkCache` in the app frame, minimap and grids, removal of the T26 layer re-render, settle and blit) and AC-3.
- 2026-11-16 — UNATTENDED CHOICE (batch): shipped a smaller slice at `review`. `--renderer layers|retained` (default `layers`); `shell::retained::RetainedScene` draws the document's chunks under the camera uniform with the feathered pipeline (AC-1a, AC-1b green on the recorder); `--renderer retained` frame = clear, dot grid, chunks, overlay, laser, minimap panel, with no layer, settle or blit (AC-1c). The layers path and the T26 code stay in place; the default is not switched.
- 2026-11-16 — LEFTOVER: (1) text, formulas, labels (T180's `TextCache` is not wired: it needs a `GlyphSource` over the shell atlas and an epoch counter) and grid-tool text are not drawn with `--renderer retained`; (2) the minimap overview still renders through the old target path; (3) `scripts/e2e.sh --backend all` was not run with `retained` (it lacks text, so the goldens would differ), so the default stays `layers` and the T26 removal waits; (4) `--renderer` is not in `--help` (`src/main.rs` is outside the files owned); (5) AC-3 frame times are the owner's; (6) T148's `tessellate_many` is not wired, as the plan says.