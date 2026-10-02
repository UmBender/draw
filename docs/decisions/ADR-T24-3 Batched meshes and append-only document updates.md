---
id: ADR-T24-3
title: Batched meshes and append-only document updates
status: accepted
kind: decision
date: 2026-10-02
task: T24
builds_on: [ADR-T07-1, ADR-T24-1, ADR-0014]
amends: [ADR-T24-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T24-3 Batched meshes and append-only document updates

## Context

On the target, a full re-render of ≈ 5 600 shapes takes 34 ms of CPU
(budget: 16 ms). `perf` shows where it goes:

- ≈ 60 % is per-triangle submission. Each `draw_triangle`, `draw_line` or
  `draw_rectangle` call goes through `QuadGl::geometry`, which copies 3–4
  vertices with a `memmove` and re-checks the batching state. An ellipse
  fill alone is up to 256 calls.
- Tessellation math is a small share.

Every commit (a new stroke or shape) re-renders the whole document layer,
so letting go of the pointer hitches, even though the only change is one
shape on top.

## Decision

- **Batching.** `shell::render` writes triangles into a `Batch`:
  - a vertex and an index list; quads share 4 vertices and fans share
    their centre;
  - submitted with one `draw_mesh` per chunk of at most
    `BATCH_MAX_VERTICES` / `BATCH_MAX_INDICES`, both below macroquad's
    per-draw-call capacity (10 000 / 5 000);
  - flushed before any text is drawn, and at the end of a pass, so z-order
    is unchanged.

  The batch hands finished chunks to a sink: the GL sink calls
  `draw_mesh`, and a recording sink makes the chunking unit-testable.
  Screen-space tessellation ([[ADR-T07-1 Screen-space tessellation in the renderer]])
  is unchanged.
- **Append-only updates.** `Editor::document_base_revision()` is the
  revision of the latest document change that was *not* a pure append of
  shapes on top. Pure appends are commits by pen, line, arrow, rect,
  ellipse and grid, plus paste and duplicate, and only when the shape
  count grew. `shell::app::plan_redraw` returns `Redraw::Append { from }`
  when all of these hold:
  - the cached layer's revision is at least the base revision;
  - the shape count grew;
  - camera, size, hidden set and underlay are unchanged.

  Then only the shapes from index `from` on are drawn onto the cached
  document layer, without clearing it. macroquad begins render passes
  with `PassAction::Nothing`, so the multisampled target keeps its
  contents.

## Alternatives considered

- **Raise macroquad's draw-call capacity (`Conf::draw_call_*_capacity`)**
  — fewer draw calls, but the per-triangle overhead was the cost, not the
  draw-call count. It can be added later, with the batch limits raised
  to match.
- **Per-shape mesh cache** ([[Rendering performance options]] option 5) —
  pan would not re-tessellate, but it needs world-space meshes and
  amending ADR-T07-1. Bigger than this task.
- **Detecting appends in the shell** (compare shape count and last id) —
  a move and a new stroke coalesced into one frame would pass the check
  and leave a stale picture. Core knows which commands only append.

## Consequences

- Full re-renders do one `memmove` per few thousand indices instead of one
  per triangle.
- Committing a stroke or shape draws one shape. Undo, redo, erase, move,
  fill and delete still re-render the whole layer.
- A new tool that commits must be classified: it is a rewrite unless it is
  added to the append list in `Editor`. The safe default is "rewrite".

## Rollback plan

- Batching: make `Batch` flush after every primitive, or go back to the
  direct `draw_*` calls in `shell::render`.
- Appends: make `plan_redraw` never return `Append`.

Either is recorded as a rollback ADR reverting this one.
