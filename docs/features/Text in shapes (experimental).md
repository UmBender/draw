---
title: Text in shapes (experimental)
task: "[[T31 Text in rectangles and grid cells]]"
adrs: ["[[ADR-T31-1 Shape text prototype behind a switch]]"]
tutorial: "[[31 Prototyping an interaction behind a switch]]"
shortcuts: ["T", "Tab", "Shift+Tab", "Enter", "Shift+Enter"]
tags: [feature, experimental]
---

# Text in shapes (experimental)

## What it does

Type into a rectangle, an ellipse or a grid cell: name a graph node, fill
a DP table cell by cell. The text sits centred in the shape, shrinks to
fit and travels with it. It is **off by default**; start the app with
`DRAW_EXPERIMENTAL=shape-text` to try it.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Type into a shape or cell | Text tool (`T`), click inside it |
| … with Select | click the shape to select it, click again inside it |
| Next / previous grid cell | `Tab` / `Shift+Tab` (reading order, wraps) |
| Cell below / above | `Enter` / `Shift+Enter` |
| Bigger / smaller text | `Ctrl+]` / `Ctrl+[` while typing |
| Finish | `Esc`, `Ctrl+Enter` or a click elsewhere |
| Cancel the edit | `Ctrl+Z` while typing |
| Clear text (and fill) | Eraser pressed inside the shape or cell (`E`, or right drag) |

Every other key works as in [[Text blocks]]. Each finished edit, and
each cell left with `Tab` / `Enter`, is one undo step. Formulas
(`$…$`, [[TeX formulas]]) work as in text blocks.

## How it works

- `core::experimental` parses `DRAW_EXPERIMENTAL`; the flags travel in
  `Helpers::experimental`, set once by `Editor::with_experimental` from
  `shell::app`.
- `Shape::Rect` / `Shape::Ellipse` carry `text: Option<ShapeText>` and
  `Shape::Grid` carries `texts: Vec<CellText>` (`src/core/shape.rs`).
  `Shape::text_frame` is the box the text fits into and
  `Shape::placed_texts` the fitted blocks (`text_layout::fit`).
- `tools::text::TextEdit` gains a `Host`; the edit is re-fitted into the
  host's frame as it is typed. `Editor::finish_text` commits it as one
  `Replace`; `Editor::step_cell` walks cells (`Shape::grid_cell_step`).
- The renderer draws `placed_texts` instead of the label
  (`render::shown_label`), in the background colour on fills.

Decisions: [[ADR-T31-1 Shape text prototype behind a switch]].

## Limits

- Experimental: the verdict (promote, rework, remove) is ADR-T31-2,
  after the owner's trial.
- Text never grows the shape and is never clipped; long text gets small.
- An unfilled shape is still selected by its outline, not by its text.
- No text in lines, arrows or strokes; no spreadsheet features.
