---
id: ADR-T85-1
title: Arrows resize the one selected grid
status: accepted
kind: decision
date: 2026-11-09
task: T85
builds_on: [ADR-T16-1, ADR-T16-2, ADR-T18-3, ADR-T21-1, ADR-T31-1, ADR-0005]
amends: [ADR-T16-2]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T85-1 Arrows resize the one selected grid

## Context

`→ ← ↓ ↑` run `Command::GridCols` and `GridRows`, which only change the size
of the next grid (ADR-T16-2). A committed grid could not be resized at all;
the only way was to redraw it and lose its contents. T85 gives the arrows a
second meaning when one grid is selected, without a new key.

A `Shape::Grid` counts its cells from corner `a`, the point where the drag
started. In a grid drawn up or to the left, column 0 is the right-hand
column or row 0 is the bottom row (ADR-T18-3). The task asks for the box to
change at the right and bottom edges, and for cell contents to stay.

## Decision

- `Editor::execute` decides before the helper path: when the Select tool is
  active and the selection is exactly one grid, `GridCols` and `GridRows`
  go to the grid and leave `helpers` alone. In every other state they
  behave as before. The commands keep their names and the keymap does not
  change.
- `Shape::grid_stepped` is the pure step: it returns the resized grid, or
  `None` when the clamped size is the same or a coordinate would stop being
  finite. A `None` means no undo step, no helper change and no redraw.
- The step always acts on the right and bottom edges **on screen**, whatever
  the drag direction, and the cell size stays. When `a` is on the moving
  edge (a grid drawn leftwards or upwards), `a` moves and the cell indices
  shift by the size change, so every remaining cell keeps its place in the
  world. Cells that fall off are dropped; undo brings them back through the
  stored `Replace` edit.
- A step that changes the grid cancels the gesture in progress first, like
  every other command that edits the document (`Copy`, `Delete`, …). A press
  that changes nothing leaves the gesture alone.

## Alternatives considered

- **Always move `b`.** Simplest, and the indices of the cells never change,
  but a grid drawn leftwards would grow at its left edge, against the task's
  criterion and the user's view of the table.
- **A new `Command::ResizeGrid`, chosen by the keymap.** The keymap is a pure
  map from chords to commands and does not know the selection, and the plan
  places the decision in `Editor::apply`.
- **Ignore the arrows during a gesture.** It would leave one press with no
  effect and no feedback; cancelling matches the other editing commands.

## Consequences

- The grid size flyout buttons send the same commands but only show with
  the Grid tool, so they never reach the new path.
- On a grid with axis indices drawn leftwards or upwards, index 0 stays at
  corner `a`, so the indices of the existing cells change by the size step.
  The contents do not move.
- Selecting a grid and pressing an arrow no longer changes the size of the
  next grid. Pressing `Esc` or picking the Grid tool first still does.

## Rollback plan

Revert the T85 commits: the guarded match arms in `Editor::execute` and
`Shape::grid_stepped`. Grids already resized stay valid, because the shape
format does not change.
