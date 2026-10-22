---
id: ADR-T37-1
title: Remember the last text size
status: accepted
kind: decision
date: 2026-10-21
task: T37
builds_on: [ADR-0013, ADR-T29-2, ADR-T31-1]
amends: [ADR-T29-3]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T37-1 Remember the last text size

## Context

[[ADR-T29-2 Text edit mode]] starts every new block at
`text_size_px(stroke width)` on screen. [[ADR-T29-3 Resize text while typing]]
lets `Ctrl+[` / `Ctrl+]` resize the open block, but rejected touching the
stroke width, so "the next block still follows `[` / `]`". The owner asked
for the opposite: after growing one block, the next one should already be
that big.

## Decision

- The Text tool state keeps `size_px: Option<f32>`, the **on-screen** font
  size (world size × zoom when the edit ends) of the last edit that was
  resized with `Ctrl+[` / `Ctrl+]` and ended by finishing it (`Esc`,
  `Ctrl+Enter`, a click away, a grid-cell step) — even if the edit was
  blank. Throwing the edit away with `Ctrl+Z` does not change it.
- A new free block, and new text in an empty shape slot or grid cell
  (ADR-T31-1), start at `size_px` converted to world units at the current
  zoom; with `None` they start at `text_size_px(width)` as before.
  Re-opened blocks and existing shape text keep their own size; a grid
  cell stepped into with `Tab` / `Enter` still takes the size of the cell
  left.
- `[` / `]` outside edit mode (`WidthUp` / `WidthDown`) clear `size_px`,
  even at either end of the ladder, so the stroke width sets the size
  again whenever the user asks for it.
- Typing in a block without resizing it does not change `size_px`, so a
  block fitted into a small cell never shrinks the next free block.
- Undo and redo do not touch it. It lives for the session only.

## Alternatives considered

- **Remember the size of every committed block** — opening a small
  cell or an old block would silently change the next block's size.
- **Remember world units** — after a zoom the next block would be
  unreadably small or huge; widths already follow the on-screen rule
  (ADR-0013).
- **Change the stroke width with `Ctrl+[` / `Ctrl+]`** — rejected by
  ADR-T29-3 for coupling two settings; still true.
- **Persist it to disk** — out of scope; nothing else is persisted.

## Consequences

The size a new block gets is no longer a pure function of the stroke
width: the user resets it with `[` / `]`. Free text sizes stay on the
text ladder at the zoom they were resized at.

## Rollback plan

Remove `text::State::size_px`, `TextEdit::resized` and the three editor
uses with a `revert(T37)` commit and a rollback ADR; new blocks then
follow the width rule again.
