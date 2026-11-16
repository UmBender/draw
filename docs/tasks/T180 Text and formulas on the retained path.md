---
id: T180
title: Text and formulas on the retained path
status: todo
wave: 72
branch: task/T180-text-and-formulas-on-the-retained-path
depends_on: [T146, T173, T179]
adrs: []
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

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — An ADR decides the glyph rasters per zoom step (recommended: text quads live in their own chunks and are re-emitted only when the raster step changes; pan and zoom inside a step re-emit nothing).
- **AC-2** — Text blocks, shape labels and formulas produce retained quads; counts on the recorder show no re-emission on pan or on zoom inside a raster step.
- **AC-3** — Parity test: text and formula scenes rendered through the retained quads equal the current path within the formula and e2e tolerances at zoom 0.25, 1 and 4.

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
