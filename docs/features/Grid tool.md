---
title: Grid tool
task: "[[T18 Grid tool]]"
adrs: ["[[ADR-T18-1 Grid drag reads live dims and snaps as a box]]", "[[ADR-T18-2 Grid size flyout in the toolbar]]", "[[ADR-T18-3 Grid axis indices]]", "[[ADR-T22-1 Numbering toggle drives grid indices]]", "[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T16-2 Helper key bindings]]", "[[ADR-T16-3 Helper settings and hooks]]", "[[ADR-T84-1 Grid from text keys and table rules]]", "[[ADR-T85-1 Arrows resize the one selected grid]]"]
tutorial: "[[18 A grid tool with live parameters]]"
shortcuts: ["G", "←", "→", "↑", "↓", "N", "Shift", "Ctrl+T", "Ctrl+Shift+V"]
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
| Resize a finished grid | Select tool, select only that grid, then `→` `←` `↓` `↑` |
| Change columns / rows with the mouse | `-` / `+` in the flyout next to `G` |
| Axis indices on / off | numbering: `N` or the toolbar `N` button |
| Square cells | hold `Shift` |
| No snapping for this drag | hold `Alt` |
| Abort the drag | `Esc` |
| Turn the selected text block into a grid | `Ctrl+T` |
| Build a grid from the system clipboard, at the cursor | `Ctrl+Shift+V` |

**Grid from text.** Lines become rows and runs of whitespace separate the
cells, so a DP table or a sample matrix appears at once. Blank lines are
ignored, short rows leave empty cells, one line gives a one-row strip.
`Ctrl+T` needs exactly one selected text block: the grid replaces it at its
top-left, in its colour and text size, as one undo step, and stays
selected. `Ctrl+Shift+V` puts the grid's corner at the cursor, in the
current colour and the size of a new text block, and selects it. Cells are
as wide as the longest token needs (square for digits). Axis indices follow
numbering. A text over 64 columns, 64 rows or 64 KiB changes nothing: it is
never cut, and the reason goes to the diagnostics log
(`~/.local/state/draw/draw.log` on most systems).

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

**Resizing a finished grid.** With the Select tool and exactly one grid
selected, the arrow keys change that grid instead of the size of the next
one. The cell size stays and the box changes at its right and bottom edges,
whichever way it was drawn. Cells that remain keep their text and fill;
those of a removed column or row go, and `Ctrl+Z` brings them back. Each
press is one undo step, and a press at 1 or 64 does nothing.

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

Resizing: `Editor::execute` sends `GridCols` and `GridRows` to
`Shape::grid_stepped` when `resizable_grid` finds a single selected grid
under the Select tool, and commits the result as one `Replace` edit, so the
history brings removed cells back. When corner `a` is on the moving edge (a
grid drawn leftwards or upwards), `a` moves and the cell indices shift, so
the remaining cells stay where they are
([[ADR-T85-1 Arrows resize the one selected grid]]).

Axis indices: `Shape::Grid` has `axes: bool`; `a` is the drag start, so
the orientation is already in the shape and survives moves.
`shape::grid_axis_labels` gives each index's world box (one cell, just
outside the start edges) and the grid's `bounds` include those bands. The
renderer draws all indices at one size (`render::axis_label_size`, fitted
to the longest index), hidden when cells are too small on screen
([[ADR-T18-3 Grid axis indices]]).

Grid from text: `tools::grid::TextTable::parse` is the pure text-to-cells
step and `TextTable::shape` builds the `Shape::Grid`. `Editor` runs it for
`Command::GridFromText` and for `InputEvent::PasteGrid`, the event the shell
makes of the clipboard for `Ctrl+Shift+V`. A refusal leaves its reason in
`Editor::take_notice`, which `shell::app::dispatch` writes to the log
([[ADR-T84-1 Grid from text keys and table rules]]).

## Limits

- No per-cell fill, numbering or merged cells.
- Rows and columns are added or removed only at the right and bottom
  edges; to insert one in the middle, redraw the grid.
- Clicking an index does not select the grid; click a grid line.
- Axes cannot be toggled on a committed grid — redraw it.
- Grid from text splits formulas that contain spaces like any text, and a
  grid cannot be turned back into text.
- The flyout covers about 130 × 60 px of canvas at the left edge while the
  grid tool is active; a drag cannot start there.
- With smart snap on, an almost-square box snaps to exactly square even
  when `cols ≠ rows`; hold `Alt` to avoid it.
