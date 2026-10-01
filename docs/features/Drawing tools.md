---
title: Drawing tools
task: "[[T09 Creation tools]]"
adrs: ["[[ADR-0007 Anti-tremor pipeline]]", "[[ADR-0013 World-space widths and zoom limits]]", "[[ADR-T08-1 Tool context and gesture overlay]]"]
tutorial: "[[09 Building drawing tools]]"
shortcuts: ["P", "L", "A", "R", "C", "Shift", "Esc"]
tags: [feature]
---

# Drawing tools

## What it does

The creation tools put new shapes on the canvas: a smoothed freehand pen and
drag-to-draw line, arrow, rectangle and ellipse. While the button is held you
see the shape being drawn; releasing commits it as a single undo step.
`Shift` snaps rectangles to squares, ellipses to circles and lines/arrows to
45° steps.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Pen (freehand, smoothed) | `P`, then left drag; a click makes a dot |
| Line / Arrow | `L` / `A`, then drag from start to end (arrow head at the release point) |
| Rectangle / Ellipse | `R` / `C`, then drag corner to corner |
| Square, circle, 45° line or arrow | hold `Shift` while dragging (checked on every move) |
| Abandon the shape being drawn | `Esc` or switch tool |
| Undo the last shape | `Ctrl+Z` (one shape per gesture) |

## How it works

Both tools implement the tool API of
[[ADR-T08-1 Tool context and gesture overlay]] (`on_pointer`, `preview`,
`cancel`) and keep their gesture in their own `State`.

- `src/core/tools/pen.rs` — on `Down` a `Smoother` is created with the current
  `SmoothingLevel` and `px_to_world = 1 / zoom`
  ([[ADR-0007 Anti-tremor pipeline]]); every event pushes its world position;
  `preview` shows `Smoother::points()` as a `Shape::Stroke`; on `Up`,
  `Smoother::finish()` is inserted with `tx_insert` and `ToolCtx::commit`.
- `src/core/tools/shape_tool.rs` — a `Drag` stores the shape kind (from
  `ctx.tool`), the world start and end, and whether `Shift` was held on the
  last event. `Drag::shape` builds the `Shape`, applying
  `constrain_square` (side = `max(|dx|, |dy|)`, signs kept) or `constrain_45`
  (projection onto the nearest multiple of 45°). Drags shorter than
  `MIN_DRAG_PX` (2 px on screen) commit nothing.
- Width: both sample `style.width_px / zoom` on `Down`, so a shape looks the
  chosen pixel width at the zoom it was drawn and scales with the drawing
  afterwards ([[ADR-0013 World-space widths and zoom limits]]).
- Shapes with non-finite coordinates are never committed.

## Limits

- No pressure sensitivity, curve fitting or text (out of scope for
  [[T09 Creation tools]]).
- New rectangles and ellipses are unfilled; use the bucket to fill them.
- Zooming with the wheel in the middle of a gesture keeps the width and
  smoothing scale sampled on `Down`.
