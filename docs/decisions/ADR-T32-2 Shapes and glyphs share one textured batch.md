---
id: ADR-T32-2
title: Shapes and glyphs share one textured batch
status: accepted
kind: decision
date: 2026-10-14
task: T32
builds_on: [ADR-T24-3, ADR-T32-1]
amends: [ADR-T24-3]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T32-2 Shapes and glyphs share one textured batch

## Context

[[ADR-T24-3 Batched meshes and append-only document updates]] submits
shapes as untextured chunks and flushes the batch before every text block
or label, so text lands on top of the shape before it. With glyphs in an
atlas ([[ADR-T32-1 Own glyph atlas rasterized with fontdue]]), a separate
text batch would still need a flush at every switch between a shape and a
text block, and a page of mixed shapes and text would cost two draw calls
per switch.

macroquad's default shader multiplies the texture sample by the vertex
colour, and an untextured mesh samples a 1×1 white texture.

## Decision

- Every atlas page keeps a 4×4 opaque white block at texel
  `(GAP, GAP)`; shape vertices use the UV of its centre, `WHITE_UV`. A
  shape thus looks the same drawn with any page bound or with none.
- `Batch` remembers the page its current chunk is bound to (`None` until
  the first glyph). Adding a glyph on another page flushes and rebinds;
  shapes never flush. Chunks of shapes and glyphs on one page are the same
  draw call, so z-order is the order things were added in, with no flush
  between a shape and its label or a text block.
- `MeshSink::draw` receives the chunk's page; the GL sink binds that
  page's texture (`None` → untextured) and calls `draw_mesh`.
- Text blocks, number labels and grid indices go through this path;
  `BATCH_MAX_VERTICES` / `BATCH_MAX_INDICES` still bound every chunk.

## Alternatives considered

- **A separate text batch** flushed on every switch — more draw calls on
  mixed pages, and two batches to keep in order.
- **A second shader without texturing for shapes** — a pipeline switch per
  switch, the same cost as the flush.

## Consequences

- One draw call per chunk and per page; a page of text is a handful of
  draw calls instead of one per character.
- The white block must never be overwritten: the packer reserves it first
  on every page and after every clear.
- Recorded chunks in tests carry their page, so batching is unit-tested
  without a GL context.

## Rollback plan

Revert with ADR-T32-1 (its rollback plan): shapes return to untextured
chunks and text to `draw_text_ex` after a flush.
