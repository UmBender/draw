---
title: Drawing tools
task: "[[T09 Creation tools]]"
adrs: ["[[ADR-0007 Anti-tremor pipeline]]", "[[ADR-0013 World-space widths and zoom limits]]", "[[ADR-T08-1 Tool context and gesture overlay]]", "[[ADR-T109-1 Shape recognition on Z]]", "[[ADR-T117-1 Scored shape recognition with looser limits]]"]
tutorial: "[[09 Building drawing tools]]"
shortcuts: ["D", "W", "A", "R", "C", "Shift", "Esc", "Z"]
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
| Pen (freehand, smoothed) | `D`, then left drag; a click makes a dot |
| Line / Arrow | `W` / `A`, then drag from start to end (arrow head at the release point) |
| Rectangle / Ellipse | `R` / `C`, then drag corner to corner |
| Square, circle, 45° line or arrow | hold `Shift` while dragging (checked on every move) |
| Abandon the shape being drawn | `Esc` or switch tool |
| Undo the last shape | `Ctrl+Z` (one shape per gesture) |
| Clean up the last pen stroke | `Z` right after drawing it: a shaky line, box or circle becomes a line, rectangle or ellipse ([[T109 Shape recognition]]) |

### Shape recognition

`Z` acts on the last pen stroke only while nothing else was done since
(another stroke, an edit, an undo); view, tool, colour and selection
changes don't count. The stroke becomes:

- a **line** from its first to its last point when it is roughly straight
  (a wobble or a gentle curve is fine);
- a **rectangle** when it is a closed loop that passes near at least three
  corners of its box and follows its sides (rounded corners, an overshoot,
  a missing last corner are fine);
- an **ellipse** when it is a closed round loop (a loop that does not quite
  close, overshoots, or ends in a short tail is fine).

When a stroke fits a rectangle and an ellipse about equally well it is left
alone ([[ADR-T117-1 Scored shape recognition with looser limits]]).

The new shape keeps the stroke's colour and width, takes the stroke's
place in the stacking order and is one undo step (`Ctrl+Z` gives the
stroke back). A stroke that fits nothing well — a scribble, a triangle,
an open arc — is left alone and no undo step is added. A tilted box or
ellipse becomes the axis-aligned shape filling the stroke's box, since
draw has no rotation.

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
- `src/core/recognize.rs` — `fit(points)` resamples the stroke to 128
  points along its length (in `f64`), tries a line (chord ≥ 0.75 of the
  path, every sample within 0.1 of the chord), then a closed loop (ends
  within 0.3 of the path) measured in its minimum-area box over 90
  rotations. A rectangle candidate (3 corners near, a score from the
  residuals) and an ellipse candidate (at most 1 corner near) are both
  scored; the better wins unless it is a near tie, and no passing score
  means nothing ([[ADR-T109-1 Shape recognition on Z]], loosened by
  [[ADR-T117-1 Scored shape recognition with looser limits]]). `Editor::recognize_last_stroke`
  checks that the newest undo step inserts exactly one stroke and that
  the redo stack is empty, then commits a `tx_replace`.

## Limits

- No pressure sensitivity, curve fitting or text (out of scope for
  [[T09 Creation tools]]).
- New rectangles and ellipses are unfilled; use the bucket to fill them.
- Shape recognition knows lines, rectangles and ellipses only (no
  triangles, arrows or polygons) and works on demand, not while drawing.
  A redo that brings the last stroke back, or a paste / duplicate of a
  single stroke, makes `Z` available again.
- Zooming with the wheel in the middle of a gesture keeps the width and
  smoothing scale sampled on `Down`.
