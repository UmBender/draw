---
title: Shapes
task: "[[T05 Shape model]]"
adrs: ["[[ADR-0004 Vector object model]]", "[[ADR-0013 World-space widths and zoom limits]]", "[[ADR-T07-1 Screen-space tessellation in the renderer]]", "[[ADR-T93-1 Coordinate plane key and plotting rules]]", "[[ADR-T95-1 Plotted point labels]]"]
tutorial: "[[05 Modelling shapes and hit-testing]]"
shortcuts: ["Ctrl+P"]
tags: [feature]
---

# Shapes

## What it does

Everything on the canvas is a vector shape: a freehand stroke, a line, an
arrow, a rectangle or an ellipse. Each has a palette colour and a width;
rectangles and ellipses can also be filled. Because shapes are vectors, they
stay sharp at any zoom and can be moved, copied, erased and filled as whole
objects.

## How to use

This note covers the **model** ([[T05 Shape model]]). Drawing shapes is added
by [[T09 Creation tools]] and rendering by [[T07 Renderer]], which extend this
note with their shortcuts.

| Action | Shortcut / gesture |
|--------|--------------------|
| Draw a line, arrow, rectangle or ellipse | `W` / `A` / `R` / `C`, then drag — see [[Drawing tools]] |
| Turn a selected text block of `x y` lines, each with an optional label, into a coordinate plane | `Ctrl+P` |

### Coordinate plane from points

Write one point per line in a text block, for example:

```
0 0 O
3.5 2
-1 4 $p_1$
```

Select the block and press `Ctrl+P`. It becomes two axes, numbered ticks and
a small filled circle per point, all ordinary shapes, as one undo step.
Anything after a line's two numbers is that point's label, written just up
and to the right of it; it may contain spaces or a `$…$` formula, and it
never changes the axes. A line that does not start with two numbers, a
value that is not finite, or more than
1 000 points changes nothing, and the reason goes to the diagnostics log.
One point, or equal coordinates, still gives a plot.

## How it works

`src/core/shape.rs` ([[ADR-0004 Vector object model]]):

- `Style { color: ColorId, width: f32 }` — width in world units
  ([[ADR-0013 World-space widths and zoom limits]]).
- `Shape` — `Stroke { points }`, `Line { a, b }`, `Arrow { a, b }` (head at
  `b`), `Rect { a, b, fill }`, `Ellipse { a, b, fill }`; for rect and ellipse
  `a`, `b` are opposite corners of the bounding box in any order.
- `bounds()` — box grown by half the width, plus the arrow head; used for
  AABB rejection and the selection outline.
- `hit(p, tol)` — "is the cursor on this shape?" for the eraser and the
  selection tool: within `tol + width/2` of the outline, or inside a filled
  rect/ellipse. Rejects by bounds first, then measures to segments, polygon
  edges, the arrow head triangle or the ellipse outline.
- `contains(p)` — interior test of rect and ellipse, used by the bucket
  (which only fills closed shapes, ADR-0004).
- `translate(delta)`, `with_fill(fill)`, `is_finite()` — for move/paste,
  bucket, and the "all stored coordinates are finite" invariant.
- `arrow_head(a, b, width)` — the head triangle, shared by renderer and
  `bounds` so what you see is what you hit. Head length is `4 × width`
  world units with no minimum, so it keeps its proportion at any zoom
  ([[ADR-0016 Arrow head proportional to width]]); base half-width half of that.

### Rendering

`src/shell/render.rs` ([[T07 Renderer]],
[[ADR-T07-1 Screen-space tessellation in the renderer]]) draws in screen
pixels through the camera:

- Outline width on screen is `width × zoom`, never less than 1 px
  (`screen_width`), so zoomed-out drawings stay visible.
- Shapes whose `bounds()` miss the visible area (plus 2 px) are skipped
  (`cull_rect`, `is_visible`).
- Strokes are segments, with round joints and caps only above 2 px; a
  single-point stroke is a dot. Arrows draw a shaft to the head base and the
  filled `arrow_head` triangle. Fills are drawn under the outline.
- Circles and ellipses are tessellated with at most ¼ px error
  (`circle_segments`, 8–256 segments) using `draw_triangle`, so drawing does
  not allocate.
- The shape being drawn is shown with `draw_preview`; the selection is a
  1 px accent outline padded 4 px on screen (`draw_selection`).

Tutorial: [[07 Rendering with macroquad and culling]].

### Coordinate plane

`src/core/tools/layout.rs` ([[T93 Coordinate plane from points]],
[[ADR-T93-1 Coordinate plane key and plotting rules]]):

- `PointList::parse` reads the text; `PointList::shapes` draws it, and
  `replace_block` swaps the result for the block.
- Both axes share one step, 1, 2 or 5 × 10ⁿ, at most about eight intervals
  on the longer axis, and one length per unit.
- Each axis runs between whole multiples of the step. It passes through the
  origin when that is in range, otherwise along the edge nearest it.
- Tick numbers are built from integers, so they show no float noise.
- `PointList::labels` holds each labelled point's index and label
  ([[T95 Labels on plotted points]], [[ADR-T95-1 Plotted point labels]]).
  Labels are drawn last and take no part in the frame.

## Limits

- Very large ellipses (radius above ~3300 px on screen) hit the 256-segment
  cap and may show facets.
- No anti-aliasing beyond what the window's MSAA setting provides.

- Ellipse outline distance is an estimate (`k0 (k0 − 1) / k1`): exact on the
  axes and on the outline, slightly off elsewhere for very flat ellipses.
  Good enough for a click tolerance of a few pixels.
- Strokes are open polylines: a stroke that loops back on itself has no
  interior and cannot be filled (see ADR-0004 rollback plan).
- Hit-testing is a linear scan with AABB rejection; no spatial index.
- In a plot, points can cover tick numbers when the axes run through the
  data, and coordinates above `1e300` in size are refused.
