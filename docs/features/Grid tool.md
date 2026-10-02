---
title: Grid tool
task: "[[T18 Grid tool]]"
adrs: ["[[ADR-T18-1 Grid drag reads live dims and snaps as a box]]", "[[ADR-T18-2 Grid size flyout in the toolbar]]", "[[ADR-T18-3 Grid axis indices]]", "[[ADR-T22-1 Numbering toggle drives grid indices]]", "[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T16-2 Helper key bindings]]", "[[ADR-T16-3 Helper settings and hooks]]"]
tutorial: "[[18 A grid tool with live parameters]]"
shortcuts: ["G", "←", "→", "↑", "↓", "N", "Shift"]
tags: [feature]
---

# Grid tool

## What it does

Draws an `n × m` table — a DP table, a board, a matrix — in one drag, like
a rectangle. While dragging, the arrow keys add or remove columns and rows
and the preview follows live. The grid is one shape: erase, select, move,
copy and undo it like any other.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Grid tool | `G` or the toolbar |
| Draw a grid | left drag |
| One column more / fewer | `→` / `←` (also before or between drags) |
| One row more / fewer | `↓` / `↑` |
| Change columns / rows with the mouse | `-` / `+` in the flyout next to `G` |
| Axis indices on / off | numbering: `N` or the toolbar `N` button |
| Square cells | hold `Shift` |
| No snapping for this drag | hold `Alt` |
| Abort the drag | `Esc` |

**Axis indices.** With numbering on (`N`, the same switch that numbers
circles and squares, [[ADR-T22-1 Numbering toggle drives grid indices]]),
new grids show 0-based column indices
along x and row indices along y, outside the grid. Index 0 is at the corner
where the drag started and the indices grow in the direction the mouse
moved: a top-left → bottom-right drag puts `0 1 2 …` above the grid and
`0 1 …` down its left side; a bottom-right → top-left drag puts them below
and to the right, counting leftwards and upwards. There is exactly one
scheme; anything else is drawn by hand.

While the grid tool is active, a small panel right of the `G` toolbar
button shows the current `cols` and `rows` with `-`/`+` buttons; it follows
the arrow keys too. Grids start at 4 × 4; columns and rows range over `1..=64`. The last values
stay for the next grid. With snapping on, corners snap like a rectangle's
(world grid, sizes of other boxes and grid cells, alignment guides).

## How it works

`core::tools::grid` (`src/core/tools/grid.rs`) keeps a `Drag` with the
snapped start and end, the guides, the `Shift` state and the style. It does
**not** store the dimensions: `preview` and the commit on `Up` read
`grid_cols`/`grid_rows` from `Helpers` in the tool's `DrawStyle`, so an
arrow key (an editor command that does not cancel the gesture) shows up on
the next redraw and the committed grid has the values at release. With
`Shift`, `square_cells` stretches the box to `side·cols × side·rows` where
`side` is the larger dragged cell side. Rendering, hit-testing and moving
are the T16 `Shape::Grid` code. See
[[ADR-T18-1 Grid drag reads live dims and snaps as a box]].

The flyout lives in `src/shell/toolbar.rs`: `flyout_panel` and
`flyout_layout` are pure layout functions, its buttons are
`ButtonKind::GridCols(±1)` / `GridRows(±1)` running the same editor commands
as the arrow keys, and `route` swallows presses on the panel when
`shell::app` reports the grid tool active
([[ADR-T18-2 Grid size flyout in the toolbar]]).

Axis indices: `Shape::Grid` has `axes: bool`; `a` is the drag start, so
the orientation is already in the shape and survives moves.
`shape::grid_axis_labels` gives each index's world box (one cell, just
outside the start edges) and the grid's `bounds` include those bands. The
renderer draws all indices at one size (`render::axis_label_size`, fitted
to the longest index), hidden when cells are too small on screen
([[ADR-T18-3 Grid axis indices]]).

## Limits

- No per-cell fill, numbering or merged cells.
- A committed grid cannot be resized or have its dimensions changed —
  redraw it.
- Clicking an index does not select the grid; click a grid line.
- Axes cannot be toggled on a committed grid — redraw it.
- The flyout covers about 130 × 60 px of canvas at the left edge while the
  grid tool is active; a drag cannot start there.
- With smart snap on, an almost-square box snaps to exactly square even
  when `cols ≠ rows`; hold `Alt` to avoid it.
