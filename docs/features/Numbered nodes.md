---
title: Numbered nodes
task: "[[T19 Auto-numbering]]"
adrs: ["[[ADR-T19-1 Numbering counter and undo]]", "[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T16-3 Helper settings and hooks]]", "[[ADR-T22-1 Numbering toggle drives grid indices]]", "[[ADR-T86-1 Zero-based numbering switch]]", "[[ADR-T87-1 Graph from an edge list key and layout rules]]", "[[ADR-T96-1 Keywords for graph input]]"]
tutorial: "[[19 Auto-numbering and undoable counters]]"
shortcuts: ["N", "Shift+N", "Ctrl+N", "Ctrl+L"]
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
| Restart numbering at the first number | `Shift+N` |
| Switch the first number between 1 and 0, restarting the counter | `Ctrl+N` |
| Draw a numbered node | ellipse (`C`) or rectangle (`R`) tool, drag |
| Give the last number back | undo (`Ctrl+Z`); redo takes it again |

Numbering is off at start and the counter starts at 1. After `Ctrl+N` it
starts at 0, to match 0-indexed code; shapes already drawn keep their
numbers, and undo never goes below the first number. Lines, arrows and
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

#### Pasting a problem's input

Put one keyword on the first line and paste the input under it as it is
([[T96 Keywords for graph input]]):

| First line | What follows |
|------------|--------------|
| `graph:` or `tree:` | Edges as above |
| `dgraph:` | Edges, with `u v` and `u v w` as arrows from `u` to `v` |
| `parents:` | Parents of nodes 2, 3, …; the root is 1 |
| `parents0:` | Parents of nodes 1, 2, …; the root is 0 |

```
parents:
5
1 1 2 2
```

After a keyword, a first line that only counts — `n`, or `n m` when `m`
lines follow and no node is above `n` — is skipped. A parent list may span
lines. If it holds one `-1`, it has one entry per node, counted from 1
(`parents:`) or 0 (`parents0:`), and the `-1` marks the root. A parent
list's tree is drawn from its root.

## How it works

`core::numbering` (`src/core/numbering.rs`) holds pure functions:
`label_new` puts `next_number` on a rectangle or ellipse, `label_count`
counts the shapes with a given label, and `advance` / `roll_back` turn a
count change into a new counter, stopping at `Helpers::first_number()`
([[ADR-T86-1 Zero-based numbering switch]]). The shape tool
(`src/core/tools/shape_tool.rs`) labels its preview and the committed shape;
it never writes the counter. The editor (`src/core/editor.rs`) counts the
shapes labelled with the counter before and after a gesture end or redo
(grew → counter + 1) and the shapes labelled `counter − 1` around an undo
(shrank → counter − 1). See [[ADR-T19-1 Numbering counter and undo]].

`core::tools::layout` (`src/core/tools/layout.rs`) reads an edge list
(`EdgeList::parse`), lays it out (`EdgeList::shapes`) and swaps it for the
block (`replace_block`). See
[[ADR-T87-1 Graph from an edge list key and layout rules]]. A keyword first
line picks an `InputFormat`; `skip_count_line` and `parent_edges` turn a
pasted input into the same edges
([[ADR-T96-1 Keywords for graph input]]).

## Limits

- Labels cannot be edited after creation; no letters, and the first
  number is 1 or 0.
- `N`, `Shift+N` and `Ctrl+N` are not undo steps.
- No toolbar button or on-screen sign shows the first number; the next
  node's preview does.
- Undoing a paste that contains a copy of the newest number also gives that
  number back, so the next node repeats it.
- Grid cells are not numbered inside; grids only get axis indices.
- A graph's numbers come from its text and do not move the counter. As with
  a paste, undoing a graph that holds the number just below the counter
  gives that number back, and redoing one that holds the counter's own
  number takes it.
- Node names in an edge list must be numbers. Without a keyword, a leading
  `n m` line is read as an edge. With one, a first edge `u v` is taken for
  a count line when `v` lines follow and no node after it is above `u`.
- A parent list cannot mark its root with `0`, and a list without `-1`
  always has root 1 (`parents:`) or 0 (`parents0:`).
- Parallel edges of different weights overlap.
