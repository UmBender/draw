---
id: ADR-T180-1
title: Retained text chunks keyed by raster step
status: accepted
kind: decision
date: 2026-11-16
task: T180
builds_on: [ADR-T179-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T180-1 Retained text chunks keyed by raster step

## Context

Text blocks, formulas and shape labels are drawn from glyph bitmaps
rasterised per zoom step (`text_raster`, `label_raster`, ADR-T29-1). A
retained chunk must not change on pan or zoom, yet which bitmap a glyph
samples depends on the step.

## Decision

- Text gets its own chunks, one per text block or label (not mixed with
  the shape chunks of `ChunkCache`), in `draw_gfx::text::TextCache`.
- Quads are in **world units**. For a glyph drawn from a raster of `rpx`
  physical pixels in a block of world size `s`, a texel is `s / rpx` world
  units, so the geometry depends on the raster step and not on the zoom.
- A chunk is keyed by (shape id, revision, raster steps, atlas epoch). The
  raster steps are the `rpx` of every distinct glyph scale of the block
  (one for plain text, a few for formulas); zero when the block is below
  the minimum size and draws nothing. A pan, or a zoom that keeps every
  `rpx`, re-emits nothing.
- Labels have a size in screen pixels that is zoom-dependent only while
  capped (`LABEL_MAX_PX`). The caller passes the label's world size,
  quantised to 10 fewer mantissa bits so float noise in a fit-limited
  label does not change the key. A capped label is re-emitted on zoom
  (exception to the rule above; pan still re-emits nothing).
- The cache does not know the atlas: a `GlyphSource` trait gives entries
  and the page side; the shell implements it for `GlyphAtlas`. The atlas
  epoch (bumped when the atlas clears) is part of the key.
- The raster ladders (`TEXT_RASTER_SIZES`, `LABEL_RASTER_SIZES`,
  `MAX_RASTER_PX`) are copied into draw-gfx, which cannot depend on the
  root crate; a parity test pins them to the shell's.

## Alternatives considered

- **Re-emit text every frame** — what the current path does; defeats the
  retained renderer.
- **One chunk of all text** — any edit or step change re-emits everything.
- **Screen-space quads** — would need re-emission on every pan or zoom.

## Consequences

Pan and in-step zoom cost no text work. Crossing a step re-emits only the
blocks that cross. The shell must keep the glyphs prepared in the atlas
before calling `update`, as it does today.

## Rollback plan

Delete `draw_gfx::text`; nothing else uses it (the app does not use this
path yet).
