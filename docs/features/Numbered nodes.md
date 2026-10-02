---
title: Numbered nodes
task: "[[T19 Auto-numbering]]"
adrs: ["[[ADR-T19-1 Numbering counter and undo]]", "[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T16-3 Helper settings and hooks]]", "[[ADR-T22-1 Numbering toggle drives grid indices]]"]
tutorial: "[[19 Auto-numbering and undoable counters]]"
shortcuts: ["N", "Shift+N"]
tags: [feature]
---

# Numbered nodes

## What it does

Trees and graphs are quick to draw: with numbering on, every new circle or
square gets the next number written inside it — 1, 2, 3, … The preview
already shows the number the shape will get. Undo gives the number back, so
fixing a misdrawn node does not leave a gap.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Numbering on / off | `N` or the toolbar |
| Restart numbering at 1 | `Shift+N` |
| Draw a numbered node | ellipse (`C`) or rectangle (`R`) tool, drag |
| Give the last number back | undo (`Ctrl+Z`); redo takes it again |

Numbering is off at start and the counter starts at 1. Lines, arrows and
strokes are never numbered. A grid drawn with numbering on gets 0-based
axis indices instead (see [[Grid tool]]) and does **not** use up a number:
circle 5, a grid, then square 6. Copy, paste and duplicate keep the
numbers of the copied shapes and do not move the counter.

## How it works

`core::numbering` (`src/core/numbering.rs`) holds pure functions:
`label_new` puts `next_number` on a rectangle or ellipse, `label_count`
counts the shapes with a given label, and `advance` / `roll_back` turn a
count change into a new counter. The shape tool
(`src/core/tools/shape_tool.rs`) labels its preview and the committed shape;
it never writes the counter. The editor (`src/core/editor.rs`) counts the
shapes labelled with the counter before and after a gesture end or redo
(grew → counter + 1) and the shapes labelled `counter − 1` around an undo
(shrank → counter − 1). See [[ADR-T19-1 Numbering counter and undo]].

## Limits

- Labels cannot be edited after creation; no letters, no custom start.
- `N` and `Shift+N` are not undo steps.
- Undoing a paste that contains a copy of the newest number also gives that
  number back, so the next node repeats it.
- Grid cells are not numbered inside; grids only get axis indices.
