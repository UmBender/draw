---
id: ADR-T16-1
title: Grid shape and shape labels
status: accepted
kind: decision
date: 2026-10-01
task: T16
builds_on: [ADR-0013, ADR-T07-1]
amends: [ADR-0004]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T16-1 Grid shape and shape labels

## Context

Competitive-programming sketches are full of DP tables, boards and numbered
tree/graph nodes. [[ADR-0004 Vector object model]] has five shape kinds; a
table drawn from rectangles and lines is slow to make and impossible to move
as one piece, and node numbers would need a text tool the app deliberately
lacks. [[T18 Grid tool]] and [[T19 Auto-numbering]] need a data model for both
before they can run in parallel.

## Decision

- New variant `Shape::Grid { a, b, cols, rows, style }`: `cols × rows`
  uniform cells in the axis-aligned box spanned by `a` and `b` (any corner
  order). `cols`, `rows` are `u32` in `1..=GRID_MAX_CELLS` (64); every query
  clamps them into that range, so an out-of-range value is never a panic.
- A grid is an **open** shape: no interior, no fill (`is_closed` false,
  `contains` false, `with_fill` a no-op, the bucket ignores it). It is hit
  within reach of any of its lines, like a stroke. Its bounds are the box
  plus half the outline width. The pure iterator `shape::grid_lines` is the
  single source of the line positions for hit-testing and rendering.
- `Rect` and `Ellipse` gain `label: Option<u32>`, a number drawn centred
  inside the shape. Labels exist on no other kind (enforced by the type).
  `translate` and `with_fill` keep the label; `Shape::label` /
  `Shape::with_label` read and set it.
- Rendering (screen space, [[ADR-T07-1 Screen-space tessellation in the renderer]]):
  the font size is fitted to the shape's on-screen box (ellipses: the
  inscribed square), hidden below `LABEL_MIN_PX` and capped at
  `LABEL_MAX_PX`. Glyphs are rasterized at a few fixed sizes and scaled, so
  zooming does not fill macroquad's glyph cache with one entry per size.

## Alternatives considered

- **Grid as a group of `Line`s** — no grouping exists; moving, erasing or
  copying a table would need a selection of 2·(n+1) shapes.
- **Per-column widths / per-cell fill** — not needed for scratch tables;
  can be an amending ADR later.
- **A general text shape** — much larger feature (editing, fonts, caret);
  numbers inside nodes cover the actual need.
- **`u8` dims** — fits 64, but `u32` matches the label type and avoids casts
  in the renderer and arithmetic.

## Consequences

- Every exhaustive `match` on `Shape` grows an arm; struct literals of
  `Rect`/`Ellipse` need `label`.
- Hit-testing a grid scans up to 130 segments, after AABB rejection.
- Labels are part of the document, so undo/redo, copy/paste and the fuzzer
  cover them for free.

## Rollback plan

Remove the variant and the field (a `revert(T16)` commit plus T18/T19
reverts) and add a rollback ADR; ADR-0004 stays as it was.
