---
id: ADR-T24-1
title: Document layer keyed by a document revision
status: accepted
kind: decision
date: 2026-10-02
task: T24
builds_on: [ADR-0006, ADR-T12-1, ADR-T15-1, ADR-T08-1]
amends: [ADR-T12-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T24-1 Document layer keyed by a document revision

## Context

[[ADR-T12-1 Blocking event loop with cached frame]] re-renders the whole
scene into one cached frame on every state change. With ≈ 5 000 shapes a
pointer move while drawing a circle re-tessellates all of them although only
the preview changed ([[T24 Smooth redraws with large documents]],
[[Rendering performance options]] option 1).

`Editor::handle` and `Editor::apply` return one `bool` ("needs a redraw"),
which cannot tell a document change from an overlay change. Every document
mutation goes through `History` (commit, undo, redo), and a tool only
commits on pointer down or up, never on a move.

## Decision

- **Two cached targets** in `shell::app`, both multisampled
  ([[ADR-T15-1 MSAA on the cached frame]]):
  - the *document layer*: background, underlay dot grid and every visible
    shape not in `Overlay::hidden`;
  - the *frame*: the document layer blitted 1:1, then preview shapes, guides,
    selection outline, marquee and toolbar. Every woken frame blits the frame
    as before.
- The document layer is described by a `LayerKey`: document revision,
  camera, framebuffer size, hidden ids and whether grid snap (the underlay)
  is on. A pure `plan_redraw(cached, next, dirty)` returns
  `Full` (key changed or no cache: both targets), `Overlay` (same key but
  dirty: frame only) or `None` (blit only).
- **Core reports document changes additively**: `Editor::document_revision()`
  is a counter that grows whenever the document may have changed. The editor
  compares the history's undo/redo lengths before and after each call that
  can mutate the document (tool pointer down/up, editing commands, undo,
  redo, clear). A commit on a full undo stack leaves both lengths unchanged,
  so there the editor falls back to the call's `bool`. `handle` and `apply`
  keep their signatures.
- Camera and viewport are compared by value in the shell; no view revision
  is needed.
- `DRAW_FRAME_TIMES` turns on a stderr line per re-render (layers, CPU ms,
  shape count) for measuring on the target.

## Alternatives considered

- **`Change::{Document, View, Overlay}` instead of `bool`** — every caller
  and test of `handle`/`apply` would change; the revision is additive.
- **A revision counter inside `Document` or `History`** — exact, but those
  modules are not part of this task; the history-length fingerprint is exact
  except on a full stack, where it is conservative.
- **Bump the revision whenever a mutating call returns `true`** — a select
  click, `Ctrl+A` or copy would re-render the document layer for nothing.
- **Draw the overlay straight to the window every frame** — saves the second
  target, but the overlay would lose MSAA and look different from committed
  shapes.

## Consequences

- An overlay-only change (preview, marquee, guides, selection, toolbar hover
  or toggle) costs one blit plus the overlay, independent of document size.
- Pan, zoom, resize, commits, undo/redo and a change in the hidden set
  (move drag start/end, eraser marking a shape) still re-render everything.
- Two MSAA targets: about twice the GPU memory of ADR-T12-1/ADR-T15-1
  (≈ 80 MB at 2560 × 1600 with 4×).
- A future document mutation that bypasses `History`, or a tool committing on
  a pointer move, would need the revision bumped explicitly; the tests in
  `core::editor` cover today's paths.

## Rollback plan

`shell::app`: render the frame as before (one target, `draw_scene` in full)
and ignore the key; `Editor::document_revision` can stay. Record as a
rollback ADR reverting this one.
