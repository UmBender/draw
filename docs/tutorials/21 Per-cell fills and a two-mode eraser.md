---
title: Per-cell fills and a two-mode eraser
step: 21
requires: ["[[10 Selection, clipboard and fill]]", "[[18 A grid tool with live parameters]]"]
feature: "[[Cell fill]]"
code: ["src/core/shape.rs", "src/core/tools/bucket.rs", "src/core/tools/eraser.rs"]
tags: [tutorial]
---

# Per-cell fills and a two-mode eraser

> **Learning path step 21.** Requires: steps 10 and 18 · Next: —

## Why this matters here

A DP table is more useful when you can shade cells: the base cases, the
visited states, the answer. And a fill you can't take back is a fill you
won't dare to use, so the eraser has to remove colour as well as shapes.

## The concept

**Sparse data inside a value.** A grid has up to 64 × 64 cells but usually a
handful are filled. Storing a sorted `Vec` of `(row, col, colour)` keeps
empty grids empty, makes lookups a binary search, and — because the fills
are part of the shape value — every existing operation (move, copy, undo)
handles them for free. Keeping the list *canonical* (sorted, unique, in
range) means two equal grids compare equal, which the bucket uses to detect
"nothing changed".

**Decide intent once, at the start of a gesture.** The first design
decided per shape: outline → delete, inside → clear. The tests exposed the
bug before any code existed: a drag across several cells must cross grid
lines, so it would delete the grid. The fix is a *mode* chosen on press:

```
Down inside a fill, away from outlines ──► Clear: clear every fill passed
Down anywhere else ─────────────────────► Remove: delete every shape hit
```

This is a common pattern in direct-manipulation UIs: the press point says
what the user means, and the rest of the drag follows that intent instead
of being reinterpreted at every pixel.

## How draw implements it

- `shape.rs`: `CellFill`, `grid_cell_at` (fraction along `a → b` times the
  cell count, clamped so the far edge belongs to the last cell),
  `grid_cell_rect` (signed cell size, same as the axis indices),
  `with_cell_fill` (binary search; insert, replace or remove), and
  `hit_outline`, which `hit` now builds on.
- `bucket.rs`: one more candidate in `topmost_where`, then
  `with_cell_fill` or `with_fill`; unchanged shape → no undo step.
- `eraser.rs`: `mode_at` on `Down`; `clear_at` returns `None` without
  cloning when there is nothing to clear (the drag samples every few pixels
  over every shape); `clear_into` keeps the latest cleared copy per shape;
  `Up` concatenates `Replace` and `Remove` edits into one `Transaction`.

## Try it

- In `eraser::tests`, start a drag on a filled cell and end it on empty
  canvas past the grid; check the grid survives.
- Make `with_cell_fill` push without sorting and watch
  `cell_fills_stay_valid` and the fuzz invariant fail.

## Further reading

- [[ADR-T21-1 Cell fills and a two-mode eraser]],
  [[ADR-T21-2 Eraser mode chosen on press]]
- Sparse vs dense matrix representations (CSR/COO) — the same trade-off at
  scale.
