---
title: One toggle, two meanings
step: 22
requires: ["[[18 A grid tool with live parameters]]", "[[19 Auto-numbering and undoable counters]]"]
feature: "[[Numbered nodes]]"
code: ["src/core/tools/grid.rs", "src/core/editor.rs"]
tags: [tutorial]
---

# One toggle, two meanings

> **Learning path step 22.** Requires: steps 18 and 19 · Next: —

## Why this matters here

T18 gave grid indices their own switch (`I`). In use, "numbering on" meant
the same thing for nodes and for tables, and two switches for one idea is
one too many. T22 removes the second switch and lets numbering decide.

## The concept

**Merge settings that users think of as one.** Each extra toggle is a
state the user must remember. When two toggles are always flipped together,
fold them: the remaining one gets a slightly broader meaning ("number what
I draw") and the UI gets simpler.

**Derived state needs no bookkeeping.** The numbering counter (step 19) is
moved by counting shapes whose `label` equals the counter before and after
each change. A grid has no `label` — its indices are a flag, `axes` — so it
is invisible to that rule: drawing, undoing or redoing a grid never moves
the counter. The behaviour the owner asked for ("circle 5, grid, square 6")
needed tests, not code.

## How draw implements it

- `tools/grid.rs`, `Drag::shape`: `axes: helpers.numbering`. That single
  line replaces a `Helpers` field, a command, a key binding and a toolbar
  button removed in the same task.
- `editor.rs` tests `numbered_grid_keeps_counter` and
  `undo_numbered_grid_keeps_counter` pin the counter behaviour.
- The removal is recorded as [[ADR-T22-1 Numbering toggle drives grid indices]],
  which *amends* ADR-T18-3 instead of editing it: the history shows both the
  first design and why it changed.

## Try it

- Give grids a `label` (e.g. `Some(next_number)`) and watch
  `numbered_grid_keeps_counter` fail — the counter rule now sees grids.

## Further reading

- [[ADR-T18-3 Grid axis indices]], [[ADR-T19-1 Numbering counter and undo]]
