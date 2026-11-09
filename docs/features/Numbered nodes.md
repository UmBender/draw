---
title: Numbered nodes
task: "[[T19 Auto-numbering]]"
adrs: ["[[ADR-T19-1 Numbering counter and undo]]", "[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T16-3 Helper settings and hooks]]", "[[ADR-T22-1 Numbering toggle drives grid indices]]", "[[ADR-T87-1 Graph from an edge list key and layout rules]]"]
tutorial: "[[19 Auto-numbering and undoable counters]]"
shortcuts: ["N", "Shift+N", "Ctrl+L"]
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

### A graph or tree from an edge list

Type the edges in a text block, select it and press `Ctrl+L`. The block
becomes numbered circles and edges, one undo step, already selected so it
can be moved. They are ordinary shapes, so every tool can edit them.

| You write | You get |
|-----------|---------|
| `1-2 2-3` | Edges, any number per line |
| `1>2` | An arrow from 1 to 2 |
| `1 2` on a line | An edge |
| `1 2 7` on a line | An edge with weight 7 written beside its middle |

If the edges form a tree it is drawn in layers from the first node named;
anything else goes on a circle. Sizes follow the block's text size, and the
spacing grows with the node count so that circles never overlap. A self-loop
is skipped, though its node is drawn; an edge repeated with the same
direction and weight is drawn once. With more than 200 nodes, or a line that
is not an edge, nothing changes and the reason, with the line number, goes
to the diagnostics log.

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

`core::tools::layout` (`src/core/tools/layout.rs`) reads an edge list
(`EdgeList::parse`), lays it out (`EdgeList::shapes`) and swaps it for the
block (`replace_block`). See
[[ADR-T87-1 Graph from an edge list key and layout rules]].

## Limits

- Labels cannot be edited after creation; no letters, no custom start.
- `N` and `Shift+N` are not undo steps.
- Undoing a paste that contains a copy of the newest number also gives that
  number back, so the next node repeats it.
- Grid cells are not numbered inside; grids only get axis indices.
- A graph's numbers come from its text and do not move the counter. As with
  a paste, undoing a graph that holds the number just below the counter
  gives that number back, and redoing one that holds the counter's own
  number takes it.
- Node names in an edge list must be numbers. A leading `n m` line is read
  as an edge.
- Parallel edges of different weights overlap.
