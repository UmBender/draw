---
title: Grid tool
task: "[[T18 Grid tool]]"
adrs: ["[[ADR-T18-1 Grid drag reads live dims and snaps as a box]]", "[[ADR-T18-2 Grid size flyout in the toolbar]]", "[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T16-2 Helper key bindings]]", "[[ADR-T16-3 Helper settings and hooks]]"]
tutorial: "[[18 A grid tool with live parameters]]"
shortcuts: ["G", "←", "→", "↑", "↓", "Shift"]
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
| Square cells | hold `Shift` |
| No snapping for this drag | hold `Alt` |
| Abort the drag | `Esc` |

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

## Limits

- No per-cell fill, numbering or merged cells.
- A committed grid cannot be resized or have its dimensions changed —
  redraw it.
- The flyout covers about 130 × 60 px of canvas at the left edge while the
  grid tool is active; a drag cannot start there.
- With smart snap on, an almost-square box snaps to exactly square even
  when `cols ≠ rows`; hold `Alt` to avoid it.
