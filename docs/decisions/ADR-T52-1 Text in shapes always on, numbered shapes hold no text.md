---
id: ADR-T52-1
title: Text in shapes always on, numbered shapes hold no text
status: accepted
kind: decision
date: 2026-11-04
task: T52
builds_on: [ADR-T16-1, ADR-T16-3, ADR-T19-1, ADR-T74-1]
amends: []
supersedes: [ADR-T31-1]
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T52-1 Text in shapes always on, numbered shapes hold no text

## Context

[[T31 Text in rectangles and grid cells]] put text in rectangles,
ellipses and grid cells behind the `DRAW_EXPERIMENTAL=shape-text` switch
([[ADR-T31-1 Shape text prototype behind a switch]]). The prototype
proved itself and the owner wants it on for everyone
([[T52 Text in shapes by default]]). Under T31 a numbered shape with text
drew the text *instead of* its number, so the number could vanish
without a trace. A numbered node's number is its content.

## Decision

- **Always on, switch removed.** `core::experimental` (`Experimental`,
  `SHAPE_TEXT`, `EXPERIMENTAL_ENV`), `Helpers::experimental` and
  `Editor::with_experimental` / `Editor::experimental` are deleted. The
  shell no longer reads `DRAW_EXPERIMENTAL`; setting it does nothing and
  prints nothing. Every rule of ADR-T31-1 (model, fit, opening, edit
  session, commit and undo, drawing, eraser, formulas) holds as if the
  switch were on, except the numbers rule below.
- **Numbered rect/ellipse holds no text** (replaces ADR-T31-1 *Numbers*).
  The invariant `label.is_some() ⇒ text.is_none()` for `Shape::Rect` and
  `Shape::Ellipse` is kept by the model:
  - `Shape::with_slot_text` leaves a labelled rect/ellipse unchanged;
  - `Shape::with_label(Some(_))` leaves a rect/ellipse that holds text
    unchanged — **text wins**: `numbering::label_new` cannot number it,
    so the label count does not grow and the counter does not advance
    ([[ADR-T19-1 Numbering counter and undo]]). No text is ever lost.
  - `Shape::text_host_at(p)` is `text_slot_at(p)` minus labelled
    rect/ellipse. Every way into a shape's text uses it: the Text tool
    press (a numbered shape gets a free block there, as on empty
    canvas), Select's second click, and the press that keeps the only
    selected shape. `text_slot_at` itself is unchanged, so the eraser's
    clear-fill drag on a numbered filled shape works as before.
  - Paste and duplicate copy shapes whole, so they keep the rule.
  - Grids, numbered or not, keep text in their cells.
- **Snapshots** ([[ADR-T74-1 Line-based editor snapshot format]]). The `helpers` record
  keeps its seventh token, now reserved: written `0`, read as a bool and
  ignored, so every fixture stays byte-identical. The editor always lets
  the snapshot reader accept shape text, and `read_snapshot` rejects a
  document rect/ellipse with both a label and text.
- **Fuzzing.** The harness no longer fuzzes two settings; it asserts the
  invariant after every event and drives sessions that number a shape
  before typing into it.

## Alternatives considered

- **Keep `core::experimental` as an empty mechanism** — 2.0.0 adds no
  features ([[T50 Version 2.0.0 refactor]]); dead code with a parser and
  tests. Re-adding a switch is a small task if one is ever needed.
- **One-line notice when `DRAW_EXPERIMENTAL` is set** — owner chose
  silence.
- **Drop the seventh `helpers` token** — changes the snapshot format and
  every fixture (files outside this task) for no behaviour gain.
- **Numbering a shape with text drops the text** — loses user text.
- **Refuse silently by not numbering *and* not advancing in the tool** —
  the same outcome as "text wins", but spread over call sites instead of
  one model rule.
- **Make `text_slot_at` skip numbered shapes** — the eraser uses it to
  clear fills; numbered filled shapes would lose that.

## Consequences

- One fewer environment variable; tutorial 31's switch is history.
- `core::snapshot::Reader::set_shape_text` and the `shape-text` checks in
  `core::snapshot` (outside this task's files) stay, always on through
  the editor; a later snapshot-format task may drop them with the
  reserved token.
- The render rule "text hides the label" becomes unreachable;
  `shown_label` keeps it as a guard.

## Rollback plan

A rollback ADR and `revert(T52)` commits bring back `core::experimental`,
the switch reads and ADR-T31-1's numbers rule; ADR-T31-1 goes back to
`accepted`.
