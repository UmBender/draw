---
id: ADR-T76-1
title: Remembered text size stays on the ladder
status: accepted
kind: decision
date: 2026-11-08
task: T76
builds_on: [ADR-T29-3, ADR-T74-1]
amends: [ADR-T37-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T76-1 Remembered text size stays on the ladder

## Context

[[ADR-T37-1 Remember the last text size]] remembers the on-screen size of
a resized block, world size × zoom when the edit ends. `Ctrl+[` / `Ctrl+]`
only put the block on a rung of the text ladder at the zoom of the key
press ([[ADR-T29-3 Resize text while typing]]), so zooming before the edit
ends multiplies the size. `Ctrl+[` to 16 px at zoom 1 and a zoom to
`ZOOM_MAX` remembers 320 px; the same at `ZOOM_MIN` remembers under 1 px.

The next block then starts unreadably huge or small, which is what
ADR-T37-1 set out to avoid, and a snapshot taken after that cannot be read
back: the reader refuses a remembered size above `MAX_TEXT_SIZE_PX`
([[ADR-T74-1 Line-based editor snapshot format]]). `--fuzz` fails at random
on it.

## Decision

- The remembered size is clamped to the ladder's range when the edit ends:
  `text_size_px(MIN_WIDTH_PX)`..=`text_size_px(MAX_WIDTH_PX)`, 14 to 76 px.
  Both ends are clamped.
- It stays the on-screen size when the edit ends. A size inside the range
  is kept as it is, even between rungs.
- Only the memory is clamped. The block keeps its own world size.
- The reader keeps its limit: every size the editor remembers is at most
  the top rung, which is below `MAX_TEXT_SIZE_PX`.

## Alternatives considered

- **Raise the reader's limit to `MAX_TEXT_SIZE × ZOOM_MAX`** — the snapshot
  would load, but draw would keep starting new blocks at a huge size.
- **Remember the rung chosen by the key press, ignoring later zoom** — it
  changes what the memory means ("the size when the edit ends") for sizes
  that are fine today.
- **Snap the memory to the nearest rung** — a zoomed block would remember a
  size it never showed; only the out-of-range cases are wrong.

## Consequences

A new block always starts between 14 and 76 px on screen, whatever the zoom
was when the last edit ended. A block resized far from zoom 1 can end at a
size the memory does not reach; the next block then starts at the end of
the ladder instead. Snapshots already saved with a larger size stay
rejected, with no migration.

## Rollback plan

Remove the clamp in `Editor::finish_text` with a `revert(T76)` commit and a
rollback ADR. The `--fuzz` failure comes back with it unless the reader's
limit is raised.
