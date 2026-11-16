---
id: T180
title: Text and formulas on the retained path
status: in-progress
wave: 72
branch: task/T180-text-and-formulas-on-the-retained-path
depends_on: [T146, T173, T179]
adrs: [ADR-T180-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T180 Text and formulas on the retained path

## Goal

Text blocks, labels and formulas have a retained form (glyph quads in chunks) that [[T172 Retained renderer in the app]] can draw, with rasters that still follow the zoom steps.

## Known facts (2026-11-16)

- Split (b) of T172 (its Log, 2026-11-15): text and formulas rasterise per zoom step (`text_raster`, `label_raster`), so they cannot be zoom-independent chunks without a design decision.
- The app does not use this path yet: nothing the user sees changes in this task.

## Spec

New module `draw_gfx::text` (`TextCache`, `GlyphSource`, `GlyphEntry`), see [[ADR-T180-1 Retained text chunks keyed by raster step]].

- **AC-1** — ADR-T180-1 decides the rasters per step. Tests in `crates/draw-gfx/tests/text.rs`: `raster_ladders_match_the_shell` (root test in `tests/text_parity.rs`), `text_raster_below_min_is_none`, `text_raster_picks_smallest_at_or_above`.
- **AC-2** — Retained quads for blocks, labels and formulas; emission counter. Tests: `block_quads_are_world_space_and_zoom_independent`, `pan_re_emits_nothing`, `zoom_inside_step_re_emits_nothing`, `zoom_across_step_re_emits_that_block_only`, `edit_re_emits_that_block`, `atlas_epoch_change_re_emits`, `formula_has_quads_and_rules`, `label_is_centered_and_capped_label_re_emits_on_zoom`, `missing_glyph_is_skipped`, `non_finite_input_emits_nothing`.
- **AC-3** — Parity in `tests/text_parity.rs` (root crate): `retained_block_matches_push_text_block` and `retained_formula_matches_push_text_block` at zoom 0.25, 1 and 4, `retained_label_matches_push_text_centered`. Metric: the retained world quads mapped through the camera equal the quads the current path sends to a recording sink (positions within 1e-3 px, same uv, colour and page). This compares the geometry the GPU would draw, not pixels, so no GL is needed; pixel parity of the whole app is T172's.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-gfx/src/**`, `crates/draw-gfx/tests/**`, `src/shell/ui_text.rs`, `src/shell/render.rs` (text part, additive), a new parity test under `tests/`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T180): specify the retained text`
- [ ] tests — `test(T180): add failing tests`
- [ ] behaviour — `feat(T180): emit retained text and formula quads`
- [ ] quality — `chore(T180): pass clippy and rustfmt`
- [ ] docs — `docs(T180): record the results`

## Log

- 2026-11-16 — created by the owner from the split of [[T172 Retained renderer in the app]] proposed in its Log.
