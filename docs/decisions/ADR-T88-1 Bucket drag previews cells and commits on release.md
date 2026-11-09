---
id: ADR-T88-1
title: Bucket drag previews cells and commits on release
status: accepted
kind: decision
date: 2026-11-09
task: T88
builds_on: ["[[ADR-T21-1 Cell fills and a two-mode eraser]]", "[[ADR-T21-2 Eraser mode chosen on press]]", "[[ADR-T74-1 Line-based editor snapshot format]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T88-1 Bucket drag previews cells and commits on release

## Context

The bucket acted on the press only. Dragging it across a grid must fill
every cell passed over, show the path at once, be one undo step and be
undone by `Esc`. Tools may commit only on `Down` or `Up`: the document
revision is only tracked there.

## Decision

- A press on a grid cell starts a drag; the grid is the only target until
  release. The bucket's `State` holds the grid's id, the last pointer
  position and the grid as painted so far.
- `preview` shows the painted grid and hides the original, as the
  eraser's clear drag does. The release commits one `Replace`, or nothing
  if no cell changed. `cancel` drops the state, so the document never saw
  the drag.
- The segment between two pointer events is walked exactly: it is
  clipped to the grid's box in cell coordinates and visited with a
  grid traversal that steps one column or one row at a time. The cost is
  at most `cols + rows` cells per event.
- A press on a rectangle or ellipse still fills on the press, and a press
  anywhere else does nothing; neither starts a drag.
- The snapshot gets a `bucket` record only while a drag runs, in the
  `[tools]` section. Snapshots of an idle bucket are unchanged, so
  `VERSION` stays 1 and no fixture is regenerated.

## Alternatives considered

- **Commit each cell as it is passed** — would need one undo step per
  cell or a history that merges steps, and `Esc` could not restore.
- **Fill on release with no preview** — the path would be invisible while
  drawing it, and the path is the point of the feature.
- **Sample the segment at a fixed spacing** like the eraser — may skip a
  cell cut only at a corner, and the spacing would depend on the cell size.
- **Always write a `bucket` record** — changes every fixture for no gain.

## Consequences

- A plain click on a grid cell now commits on release, not on press. A
  pen barrel takeover during the press therefore cancels the fill.
- While dragging, the grid is drawn in the overlay, so shapes above it in
  the document appear under it until release.
- The first painted cell re-renders the document layer once, because the
  grid becomes hidden. Later cells only redraw the overlay.

## Rollback plan

Revert the T88 commits: `bucket::State` becomes a unit struct again, the
`bucket` snapshot record goes away, and the bucket acts on the press only.
Snapshots taken during a drag would no longer load; no others are affected.
