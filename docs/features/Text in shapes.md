---
title: Text in shapes
task: "[[T31 Text in rectangles and grid cells]]"
adrs: ["[[ADR-T31-1 Shape text prototype behind a switch]]", "[[ADR-T52-1 Text in shapes always on, numbered shapes hold no text]]"]
tutorial: "[[31 Prototyping an interaction behind a switch]]"
shortcuts: ["T", "Tab", "Shift+Tab", "Enter", "Shift+Enter"]
tags: [feature]
---

# Text in shapes

## What it does

Type into a rectangle, an ellipse or a grid cell: name a graph node, fill
a DP table cell by cell. The text sits centred in the shape, shrinks to
fit and travels with it. It is always on
([[T52 Text in shapes by default]]); it started as a prototype behind a
switch ([[T31 Text in rectangles and grid cells]]).

A **numbered** rectangle or ellipse ([[Numbered nodes]]) holds no text:
its number is its content. Clicking inside one with the Text tool starts
a free text block there, as on empty canvas. Grids keep text in their
cells, with or without indices.

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

- `Shape::Rect` / `Shape::Ellipse` carry `text: Option<ShapeText>` and
  `Shape::Grid` carries `texts: Vec<CellText>` (`src/core/shape.rs`).
  `Shape::text_frame` is the box the text fits into and
  `Shape::placed_texts` the fitted blocks (`text_layout::fit`).
- `Shape::text_host_at` is the slot under the pointer that may take
  text: `text_slot_at` minus numbered rectangles and ellipses. The Text
  tool, Select's second click and the press that keeps the only selected
  shape all go through it.
- The model keeps `label.is_some() ⇒ text.is_none()`: `with_slot_text`
  never fills a numbered shape and `with_label` never numbers a shape
  holding text (text wins; the numbering counter does not advance).
  Snapshots with both are rejected; the fuzz harness checks it after
  every event.
- `tools::text::TextEdit` has a `Host`; the edit is re-fitted into the
  host's frame as it is typed. `Editor::finish_text` commits it as one
  `Replace`; `Editor::step_cell` walks cells (`Shape::grid_cell_step`).
- The renderer draws `placed_texts`, in the background colour on fills,
  and the label of numbered shapes (`render::shown_label`).

Decisions: [[ADR-T31-1 Shape text prototype behind a switch]] (superseded),
[[ADR-T52-1 Text in shapes always on, numbered shapes hold no text]].

## Limits

- Text never grows the shape and is never clipped; long text gets small.
- An unfilled shape is still selected by its outline, not by its text.
- A numbered rectangle or ellipse cannot be named; draw it unnumbered
  (`N` off) to write in it.
- No text in lines, arrows or strokes; no spreadsheet features.
