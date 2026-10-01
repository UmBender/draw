---
title: Snapping
task: "[[T17 Snapping]]"
adrs: ["[[ADR-T17-1 Snapping order and tolerances]]", "[[ADR-T16-3 Helper settings and hooks]]", "[[ADR-T16-2 Helper key bindings]]"]
tutorial: "[[17 Snapping and alignment guides]]"
shortcuts: ["M", "Shift+G", "Alt (hold while dragging)"]
tags: [feature]
---

# Snapping

## What it does

Lines, arrows, rectangles and ellipses come out neat without effort. With
**smart snap** on, almost-round boxes become exact circles and squares, box
sides match the sizes of boxes already drawn (or whole multiples of them),
and corners, edges and centres line up with nearby shapes — a thin accent
guide shows every alignment. With **grid snap** on, points land on a
20-unit world grid shown as faint dots.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Toggle smart snap (round, sizes, alignment) | `M` or the toolbar |
| Toggle grid snap and the dot grid | `Shift+G` or the toolbar |
| Draw without snapping | hold `Alt` while dragging |
| Exact square / circle / 45° line | hold `Shift` (grid snap still applies) |

Both helpers are off at start.

## How it works

`core::snap` (`src/core/snap.rs`) is a set of pure functions. The shape tool
(`src/core/tools/shape_tool.rs`) snaps the start point on `Down`
(grid → align) and the moving end on every event
(grid → size → align → round); size and round only apply to boxes.
Targets are collected from the document: box edges, centres and sizes of
rectangles, ellipses and grids (plus grid cell sizes), and line/arrow
endpoints. Tolerances are 6 screen pixels, so snapping feels the same at
every zoom. Guides are computed from the final shape, so they never claim an
alignment that a later step undid. `shell::render` draws the dot grid under
the shapes and the guides above the preview. Details and trade-offs:
[[ADR-T17-1 Snapping order and tolerances]].

## Limits

- Only the line, arrow, rectangle and ellipse tools snap; moving a
  selection, freehand strokes and the grid tool do not.
- Every document shape is a target, also off-screen ones; freehand strokes
  never are.
- The snap grid step is fixed (20 world units); when zoomed out the dots are
  thinned to powers of two of the step, but points still snap to every step.
