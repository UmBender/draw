---
id: ADR-T18-1
title: Grid drag reads live dims and snaps as a box
status: accepted
kind: decision
date: 2026-10-01
task: T18
builds_on: [ADR-T08-1, ADR-T16-1, ADR-T16-2, ADR-T16-3, ADR-T17-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T18-1 Grid drag reads live dims and snaps as a box

## Context

The grid tool drags a box like the rectangle tool, but its shape also has
`cols × rows`, which the arrow keys change *during* the drag
([[ADR-T16-2 Helper key bindings]]). The dims live in `Helpers` inside
`DrawStyle` ([[ADR-T16-3 Helper settings and hooks]]). Two questions: where
the tool gets the dims from, and how the drag snaps (T17 added snapping for
the shape tools, [[ADR-T17-1 Snapping order and tolerances]]).

## Decision

- **Dims are read, never stored.** The tool's `Drag` keeps only the start,
  end, guides, `Shift` state and style (sampled on `Down`, like the shape
  tool). `preview` reads `view.style.helpers` and `Up` reads
  `ctx.style.helpers`, so an arrow key changes the preview on the next
  redraw and the committed grid uses the values at release. The values
  persist for the next grid because they stay in the editor.
- **Shift = square cells**: with `w`, `h` the dragged box and `side =
  max(|w| / cols, |h| / rows)`, the end becomes
  `start + (side·cols·sign w, side·rows·sign h)`. It is computed from the
  dims at the moment the shape is built, so it stays exact when the dims
  change mid-drag.
- **Snapping as a box**: the start uses `snap::snap_start` and the end
  `snap::snap_end` with `DragKind::Box`, exactly like rectangles; the
  guides go to the overlay. `Shift` keeps only grid snap and then applies
  the square-cells constraint, as for the shape tool.
- **Stray clicks** reuse `shape_tool::MIN_DRAG_PX`; grids are never
  numbered.

## Alternatives considered

- **Copy dims into the drag on `Down` and update them via a command hook** —
  the tool API has no command hook; it would need a change outside the
  task's files and duplicates state the editor already owns.
- **Snap as `DragKind::Point`** — loses size matching with other grids and
  box guides (edges and centre). The cost of `Box` is that round snapping
  can make an almost-square box exactly square even when `cols ≠ rows`;
  it only acts within 10 % and `Alt` turns it off.

## Consequences

- No state to keep in sync: the preview can never show stale dims.
- The editor must redraw after `GridCols`/`GridRows` (it already returns
  `true` when the dims change, T16).

## Rollback plan

Revert the T18 behaviour commit in `src/core/tools/grid.rs`; the T16 stub
(no-op grid tool) is restored and nothing else depends on it.
