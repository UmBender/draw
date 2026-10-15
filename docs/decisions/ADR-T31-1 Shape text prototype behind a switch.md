---
id: ADR-T31-1
title: Shape text prototype behind a switch
status: accepted
kind: decision
date: 2026-10-15
task: T31
builds_on: [ADR-T16-1, ADR-T16-3, ADR-T19-1, ADR-T21-1, ADR-T21-2, ADR-T29-1, ADR-T29-2, ADR-T29-3, ADR-T30-1]
amends: [ADR-T16-1, ADR-T21-2, ADR-T29-2]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T31-1 Shape text prototype behind a switch

## Context

[[T31 Text in rectangles and grid cells]] is an experiment: can typing
into rectangles, ellipses and grid cells (a DP table filled cell by
cell, named graph nodes) be fast without getting in the way? It must be
off by default, cheap to remove, and reuse the text edit mode of
[[ADR-T29-2 Text edit mode]] rather than invent a second one. The owner
answered the task's open questions in the spec step; this ADR records
the answers and how they are built.

## Decision

- **Switch.** `DRAW_EXPERIMENTAL` holds a comma- or space-separated list
  of feature names, case-insensitive; unknown names are ignored.
  `core::experimental::Experimental::from_env_value(Option<&str>)` is a
  pure function over that value; `shape-text` turns on
  `Experimental::shape_text`. The shell reads the variable once at start
  and passes it to `Editor::with_experimental`. The flags travel in
  `Helpers::experimental` ([[ADR-T16-3 Helper settings and hooks]]), so
  every tool sees them; no key or command changes them. Off, nothing new
  is reachable: no input can create shape text.
- **Model** ([[ADR-T16-1 Grid shape and shape labels]] amended).
  `Shape::Rect` and `Shape::Ellipse` gain `text: Option<ShapeText>`;
  `Shape::Grid` gains `texts: Vec<CellText>`, sorted by `(row, col)`, at
  most one per cell, all inside the grid, like `fills`
  ([[ADR-T21-1 Cell fills and a two-mode eraser]]). `ShapeText { text,
  size }` holds non-blank text and its **largest** font size in world
  units; `CellText` adds `col`, `row`. Text is part of the shape, so
  move, copy, paste, duplicate, undo and the fuzzer cover it for free;
  bounds and hit-testing do not change.
- **Fit (Q4).** Text is laid out like a text block
  ([[ADR-T29-1 Text shape with monospaced layout and an embedded font]],
  formulas typeset — [[ADR-T30-1 TeX subset, box layout and metrics]])
  and centred in the shape's *text frame*: the box of a rectangle, the
  inscribed rectangle of an ellipse (box / √2) or the grid cell, scaled
  by `SHAPE_TEXT_FILL` (0.8) about its centre. Its font size is the
  stored size, shrunk until the layout fits the frame
  (`text_layout::fit`). Shapes never grow; text is never clipped.
- **Numbers (Q3).** Text and label are separate. A shape with text draws
  its text instead of its label; blanking the text shows the number
  again. The numbering counter ([[ADR-T19-1 Numbering counter and undo]])
  and grid indices (outside the grid) are untouched.
- **Opening (Q1).** With the switch on:
  - a Text tool press re-opens a text block under it, as before;
    otherwise, if the topmost shape whose interior or cell holds the
    press is a rectangle, an ellipse or a grid, it opens an edit of that
    shape's text or of that cell (caret at the end); otherwise it starts a
    free block;
  - with Select, a click on a rectangle, ellipse or grid that was
    **already the only selected shape** opens its text (the cell under
    the click for a grid), exactly like re-editing a text block. So that
    the second click reaches it, a press inside the interior (or grid
    box) of the only selected rectangle, ellipse or grid counts as a
    press on it: it keeps the selection, and a drag from there moves it.
  A new text takes its size from the stroke width, like a new block
  (`text_size_px(width) / zoom`); `Ctrl+[` / `Ctrl+]` step it
  ([[ADR-T29-3 Resize text while typing]]).
- **Edit session.** The `TextEdit` of ADR-T29-2 gains a host
  `(shape id, cell, frame)`; its origin and display size are re-fitted
  to the frame as the source is typed. All T29 keys keep their meaning,
  except in a grid **cell** (Q2):

  | Key | In a cell |
  |-----|-----------|
  | `Tab` / `Shift+Tab` | commit, open the next / previous cell in reading order (left to right, top to bottom on screen), wrapping around the grid |
  | `Enter` / `Shift+Enter` | commit, open the cell below / above; at the bottom / top row the edit stays where it is |

  Cells hold one line. Outside edit mode the arrows still change the grid
  size for new grids (T18).
- **Commit and undo.** Ending a hosted edit replaces the host with its
  new text (`None` / no cell text when blank): one `Replace` per changed
  edit, nothing when unchanged. So **one history entry per finished
  edit**, also per cell when moving with `Tab` / `Enter`. `Ctrl+Z` while
  editing cancels the edit, as in T29.
- **Drawing.** While editing, the overlay hides the host, draws it
  without the edited text and draws the edit as an ordinary text block
  (as typed, formulas as source); the caret and the edit outline come
  from that block.
- **Eraser (Q5)** ([[ADR-T21-2 Eraser mode chosen on press]] amended).
  Text counts as interior content like a fill: a press inside a shape or
  cell with a fill **or text**, away from every outline, starts a clear
  drag, and clearing removes both the fill and the text there, in one
  undo step.
- **Formulas (Q6)** work in shape text as in text blocks.
- **Measured** (AC-5): time to fill a 5 × 5 DP table and to label a
  6-node graph, with and without the switch, and every misfire. The
  verdict is ADR-T31-2.

## Alternatives considered

- **Double-click with any tool** — needs a clock in the shell or turns
  every second click into an edit (ADR-T29-2 rejected the clock).
- **`Enter` on a selected shape** — a new binding for an experiment, and
  the cell would depend on an earlier click.
- **Arrows crossing cells at the text ends** — arrows mean two things
  inside one cell; `Tab` / `Enter` are the spreadsheet keys.
- **Typed number becomes the label** — couples text to the counter's
  undo rules.
- **Text as a separate `Shape::Text` linked to its host** — moving,
  copying and erasing would each need linking rules.
- **Clip or grow the shape** — clipping hides what was typed; growing a
  grid changes every cell.

## Consequences

- Every struct literal of `Rect`, `Ellipse` and `Grid` grows a field
  (`text: None`, `texts: Vec::new()`), including in `shape_tool.rs`,
  `clipboard.rs`, `snap.rs` and `tests/perf.rs` (files-owned amendment in
  the task note).
- An unfilled shape with text has no hit interior: the text is not a
  handle for selecting it (a click on the outline is).
- The fuzzer runs sessions with the switch on and off.

## Rollback plan

A rollback ADR plus `revert(T31)` commits remove the fields, the switch
module and the edit routing; ADR-T16-1, ADR-T21-2 and ADR-T29-2 stay as
they were.
