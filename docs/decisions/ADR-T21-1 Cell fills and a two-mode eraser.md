---
id: ADR-T21-1
title: Cell fills and a two-mode eraser
status: accepted
kind: decision
date: 2026-10-01
task: T21
builds_on: [ADR-0004, ADR-T08-1, ADR-T16-1, ADR-T18-3]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T21-1 Cell fills and a two-mode eraser

## Context

Grids are open shapes with no fill (ADR-T16-1), so the bucket ignores
them. The owner wants to fill single cells, and to take fills away with
the eraser — on cells, circles and rectangles — without deleting the
shape. Until now the eraser removed any shape it touched, and a filled
shape counted as touched anywhere inside.

## Decision

- **Fills live in the grid**: `Shape::Grid { fills: Vec<CellFill> }`,
  sorted by `(row, col)`, one entry per cell, only valid cells. A sparse
  list keeps unfilled grids empty and makes move, copy and undo work with
  no extra code (the shape is cloned whole). Cells are indexed from `a`
  toward `b`, the same convention as the axis indices (ADR-T18-3). Grids
  are never resized, so indices stay valid.
- **Bucket**: the target is the topmost shape that is a closed shape
  containing the point *or* a grid whose box contains it; on a grid it
  sets one cell with `tx_replace`. The grid stays an open shape
  (`contains` = false), so selection and hit-testing are unchanged.
- **Eraser has two effects per shape**: outline hit (`hit_outline`, i.e.
  `hit` without the filled-interior rule) → remove; else inside a filled
  closed shape → clear its fill; else inside a filled grid cell → clear
  that cell. A shape both cleared and removed in one drag is removed. On
  release, fill changes (`Replace`) then removals go in one transaction.
  The preview hides every affected shape and draws the cleared copies.
- **Renderer** draws cell fills before the grid lines. Axis indices keep
  the outline colour.

## Alternatives considered

- **Bucket inserts a separate filled rectangle per cell** — fills would
  not follow the grid when it moves and would need z-order management.
- **Dense `Vec<Option<ColorId>>` of `cols × rows`** — up to 4096 entries
  for every grid, even unfilled ones.
- **Clearing with the bucket (same colour toggles)** — the owner chose the
  eraser.
- **Keep "touch a filled inside = delete"** — makes fills impossible to
  remove without undo.

## Consequences

- The eraser no longer deletes a filled shape by rubbing its inside: the
  first pass clears the fill, touching the outline deletes it.
- `Shape::Grid` is no longer field-for-field `Copy`-friendly; `match
  *shape` with `..` still works because the bound fields are `Copy`.

## Rollback plan

Revert the T21 commits: removes `fills`, restores the bucket and eraser.
