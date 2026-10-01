---
title: Cell fill
task: "[[T21 Cell fill and fill eraser]]"
adrs: ["[[ADR-T21-1 Cell fills and a two-mode eraser]]", "[[ADR-T21-2 Eraser mode chosen on press]]", "[[ADR-T18-3 Grid axis indices]]"]
tutorial: "[[21 Per-cell fills and a two-mode eraser]]"
shortcuts: ["B", "E", "right drag"]
tags: [feature]
---

# Cell fill

## What it does

The bucket colours a single cell of a grid — mark visited DP states, the
answer cell or a path through a board. The eraser takes fills away without
deleting the shape: rub inside a filled cell, circle or rectangle and the
colour goes, the shape stays.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Fill one grid cell | bucket (`B`), click the cell |
| Recolour a cell | bucket with another colour |
| Clear fills | eraser (`E`) or right drag, **pressed inside a fill**, then drag over every fill to clear |
| Delete a shape | eraser pressed on its outline or on empty canvas, as before |

The press decides what a whole eraser stroke does. Started on a fill, it
only clears fills — cells and filled circles/rectangles — and can cross grid
lines and outlines without deleting anything. Started anywhere else, it
deletes what it touches, as it always did. Either way, one undo step.

Axis indices keep their colour: they sit outside the cells.

## How it works

`Shape::Grid` stores `fills: Vec<CellFill>` (`src/core/shape.rs`), sorted
by `(row, col)` with one entry per cell, counted from the drag-start corner
like the axis indices. `grid_cell_at`, `grid_cell_rect`, `cell_fill` and
`with_cell_fill` are pure helpers. Because the fills live in the shape,
moving, copying and undo work unchanged.

The bucket (`src/core/tools/bucket.rs`) targets the topmost shape that is a
closed shape containing the point or a grid whose box contains it. The
eraser (`src/core/tools/eraser.rs`) chooses `Mode::Remove` or `Mode::Clear`
on `Down`; in clear mode it collects the cleared versions of the shapes it
passes over, the preview shows them, and the release commits `Replace`
edits (and in remove mode `Remove` edits) as one transaction. The renderer
paints cell fills before the grid lines. See
[[ADR-T21-1 Cell fills and a two-mode eraser]] and
[[ADR-T21-2 Eraser mode chosen on press]].

## Limits

- No flood fill across several cells; click each cell.
- A shape drawn on top of a grid cell takes the bucket click.
- A clear stroke never deletes; to delete a filled shape, press on its
  outline (or outside it) and drag into it.
- Fills cannot be cleared with the bucket.
