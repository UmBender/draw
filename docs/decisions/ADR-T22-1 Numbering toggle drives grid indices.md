---
id: ADR-T22-1
title: Numbering toggle drives grid indices
status: accepted
kind: decision
date: 2026-10-01
task: T22
builds_on: [ADR-T18-3, ADR-T19-1, ADR-T16-2]
amends: [ADR-T18-3, ADR-T18-2]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T22-1 Numbering toggle drives grid indices

## Context

[[ADR-T18-3 Grid axis indices]] gave grid indices their own setting
(`Helpers::grid_axes`, key `I`, a flyout row). The owner wants one switch:
the numbering button that labels circles and squares should also number
grids, and a numbered grid must not consume a counter value.

## Decision

- The grid tool sets `axes` on new grids from `Helpers::numbering`.
  `Shape::Grid::axes` stays: each grid keeps its indices after the switch
  changes.
- `Helpers::grid_axes`, `Command::ToggleGridAxes`, the `I` binding and the
  flyout's `axes` row (`ButtonKind::GridAxes`) are removed. The flyout is
  back to the two rows of [[ADR-T18-2 Grid size flyout in the toolbar]].
- The counter is untouched: [[ADR-T19-1 Numbering counter and undo]]
  counts shapes whose `label` equals the counter; grids have no `label`,
  so drawing, undoing or redoing a grid never moves it.

## Alternatives considered

- **Keep both switches** — two controls for one idea; the owner asked for
  one.
- **Grids take a number too (e.g. a label)** — breaks the circle 5, grid,
  square 6 sequence the owner wants.

## Consequences

- `Helpers` is back to three bools; the `struct_excessive_bools` allow
  added in T18 is no longer needed and is removed.
- Turning numbering on to label nodes also numbers grids drawn meanwhile.

## Rollback plan

Revert the T22 commits; ADR-T18-3's separate toggle comes back.
