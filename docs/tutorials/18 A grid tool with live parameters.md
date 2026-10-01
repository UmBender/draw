---
title: A grid tool with live parameters
step: 18
requires: ["[[09 Building drawing tools]]", "[[16 Growing a data model without breaking it]]"]
feature: "[[Grid tool]]"
code: ["src/core/tools/grid.rs"]
tags: [tutorial]
---

# A grid tool with live parameters

> **Learning path step 18.** Requires: steps 9 and 16 · Next: step 19

## Why this matters here

DP tables and boards are drawn constantly in competitive programming, and a
grid made of loose lines is painful to move or fix. The grid tool makes it
one drag — but the number of cells must be adjustable *while* dragging,
which means a tool parameter that changes in the middle of a gesture.

## The concept

A gesture has state (where it started, where the pointer is) and
**parameters** (style, cell counts). There are two ways to handle a
parameter that can change mid-gesture:

1. **Copy it into the gesture** on `Down` and update the copy when it
   changes. Needs a notification path from wherever the change happens
   into the tool, and two copies of the value can disagree.
2. **Read it where it is used.** Keep it in one place (the editor) and let
   the preview and the commit look it up every time.

draw uses option 2: a single source of truth means the preview can never
show stale dimensions, and the tool API from step 9 (`on_pointer`,
`preview`, `cancel`) needs no new hook.

```
Down ─ Move ─ (→) ─ Move ─ (↓) ─ Up
         │      │      │     │    └─ commit with helpers at release
         │      │      │     └─ editor: grid_rows += 1, redraw
         │      │      └─ preview reads helpers: 5 × 4
         │      └─ editor: grid_cols += 1, redraw
         └─ preview reads helpers: 4 × 4
```

The constraint for square cells is a small piece of geometry: with the
dragged box `w × h` and `cols × rows` cells, a square cell has side
`s = max(|w|/cols, |h|/rows)`, and the box becomes `s·cols × s·rows` in the
direction of the drag (keep the signs of `w` and `h`).

## How draw implements it

In `src/core/tools/grid.rs`:

- `State { drag: Option<Drag> }` — `Drag` holds `start`, `end`, `guides`,
  `shift` and the style sampled on `Down`. No cell counts.
- `Drag::shape(&Helpers)` builds the `Shape::Grid`, clamping the counts to
  `1..=GRID_MAX_CELLS` and calling `square_cells` when `shift` is held.
- `on_pointer` snaps the start with `snap::snap_start` and the end with
  `snap::snap_end(.., DragKind::Box, ..)` — the same pipeline as rectangles
  (step 17). On `Up` it skips drags shorter than `MIN_DRAG_PX` and commits
  one transaction.
- `preview` calls `drag.shape(&view.style.helpers)`; that is the whole
  "live parameter" mechanism.

The arrow keys are plain editor commands (`GridCols(±1)`, `GridRows(±1)`,
from step 16) that change `Helpers` and do not cancel the gesture.

## Try it

- In `grid::tests`, write a test that drags, sets `f.dims(2, 8)` with
  `Shift` held on every event, and checks the committed box is four times
  taller than wide.
- Change `Drag::shape` to clamp to `1..=8` instead of `GRID_MAX_CELLS`:
  the range property `committed_grid_is_finite_and_in_range` still passes
  (8 is in range), but `dims_persist_for_next_grid` (9 rows) fails. A
  property checks an invariant; an example pins the exact behaviour.

## Further reading

- [[ADR-T18-1 Grid drag reads live dims and snaps as a box]]
- [[09 Building drawing tools]] — the tool API this tool plugs into.
- "Single source of truth" in UI state management (e.g. the Elm
  architecture).
