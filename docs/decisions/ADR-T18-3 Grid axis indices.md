---
id: ADR-T18-3
title: Grid axis indices
status: accepted
kind: decision
date: 2026-10-01
task: T18
builds_on: [ADR-T16-1, ADR-T16-2, ADR-T16-3, ADR-T18-1, ADR-T18-2]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T18-3 Grid axis indices

## Context

The owner wants grids (DP tables, matrices) numbered: 0-based column
indices along x and row indices along y, **outside** the grid, with index 0
where the drag started and indices growing in the direction the mouse
moved. Exactly one numbering scheme; anything else is drawn by hand.

## Decision

- **Model**: `Shape::Grid` gains `axes: bool`. `a` is the drag start and
  `b` the drag end; the grid tool already stores them that way and
  `translate` keeps the order, so the orientation needs no extra field.
- **Placement** (`shape::grid_axis_labels`, pure, world space): column `i`
  (`0..cols`) is the `i`-th column counting from `a.x` toward `b.x`; its
  label box is that column's width, one cell height tall, just outside the
  edge `y = a.y` (on the side away from `b`). Row `j` likewise: one cell
  wide, just outside `x = a.x`. A top-left → bottom-right drag gives
  indices above and left, like a matrix.
- **Bounds** of a grid with axes include the label bands, so culling and
  marquee selection see the labels. `hit` is unchanged (lines only).
- **Rendering**: every label uses one font size, fitted by `label_size` to
  the longest index (`max(cols, rows) − 1`) in one cell's screen box, with
  the same `LABEL_MIN_PX`/`LABEL_MAX_PX` rules as shape labels.
- **Setting**: `Helpers::grid_axes` (off by default), toggled by
  `Command::ToggleGridAxes`, bound to `I` ("indices") and to a third
  flyout row (`axes  [on|off]`). New grids copy it at release, like the
  dimensions; the preview shows it live.

## Alternatives considered

- **Labels inside the cells** — the owner asked for outside.
- **Store orientation separately / normalize `a`,`b`** — `a`/`b` already
  carry it; normalizing would lose it.
- **Labels on the far side too, or 1-based** — one scheme only, by request.
- **No key, flyout only** — the app is keyboard-first; `I` was free.

## Consequences

- Every exhaustive `Shape::Grid { .. }` literal gains `axes` (shape,
  snap and render tests, fuzz strategies).
- Clicking on a label does not select the grid; clicking a grid line does.

## Rollback plan

Revert the AC-7 commits of T18: removes the field, the command, the key,
the flyout row and the label drawing.
