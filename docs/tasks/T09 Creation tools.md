---
id: T09
title: Creation tools
status: ready
wave: 5
branch: task/T09-creation-tools
depends_on: [T04, T08]
adrs: ["[[ADR-0007 Anti-tremor pipeline]]", "[[ADR-0013 World-space widths and zoom limits]]"]
feature: "[[Drawing tools]]"
tutorial: "[[09 Building drawing tools]]"
tags: [task]
---

# T09 Creation tools

## Goal

Pen (smoothed), line, arrow, rectangle and ellipse tools.

## Spec

- **AC-1** — Pen: down starts a `Smoother` (current level, `px_to_world` from camera);
  move pushes; `preview()` shows the live stroke; up commits one `Stroke` in one
  transaction. A click without movement commits a dot. *Tests:* `pen::tests::*`.
- **AC-2** — Stroke width = `style.width_px / zoom` at the time of drawing.
  *Test:* `pen_width_is_zoom_independent_on_screen`.
- **AC-3** — Shape tool (`Line`, `Arrow`, `Rect`, `Ellipse`): drag from down to up;
  preview while dragging; commit on up. Drag shorter than 2 px (screen) commits nothing.
  *Tests:* `shape_tool::tests::*`.
- **AC-4** — `Shift` constrains: rect → square, ellipse → circle, line/arrow → nearest 45°.
  *Tests:* `shift_constrains_*`.
- **AC-5** — `cancel()` (Esc / tool switch) discards the gesture with no document change.
  *Test:* `cancel_discards_gesture`.
- **AC-6** — Each completed gesture is exactly one undo step. *Test:* `one_gesture_one_undo`.

## Out of scope

Text, pressure, curve fitting.

## Files owned

`src/core/tools/pen.rs`, `src/core/tools/shape_tool.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 9 — requires steps 4 and 8.

## Log
