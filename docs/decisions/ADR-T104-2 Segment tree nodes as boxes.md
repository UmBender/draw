---
id: ADR-T104-2
title: Segment tree nodes as boxes
status: accepted
kind: decision
date: 2026-11-10
task: T104
builds_on: [ADR-T104-1, ADR-T87-1]
amends: [ADR-T104-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T104-2 Segment tree nodes as boxes

## Context

ADR-T104-1 drew a segment tree like any tree: one circle per node
holding `$[l, r]$`. Trying it, the owner preferred square nodes with an
empty box under each one, to write the node's value (the sum, minimum,
… being computed) by hand or with the Text tool.

## Decision

Amends the **Tree** drawing of ADR-T104-1; parsing, size limits and the
first number are unchanged.

- **Node.** Two squares of the same size, stacked and touching: the top
  one holds `$[l, r]$` (typeset, in the rectangle's text slot), the one
  under it is empty, for the value. Both are plain `Shape::Rect`s, so
  the value box takes text, fill and every edit like any rectangle.
- **Size.** One half side `h` for all squares: `NODE_RADIUS_EM × size`,
  or more when the widest label needs it to fit the square's text frame
  at full size.
- **Layout.** The layered tree layout of ADR-T87-1 with `h` as the
  radius; each layer is `2h` further down to make room for the value
  boxes.
- **Edges.** An undirected line from the middle of a parent's value box
  bottom edge to the middle of the child's range box top edge, both
  ends just off the strokes.
- `EdgeList` remembers that it came from `segtree:`; every other graph
  keeps circles.

## Alternatives considered

- **Circles** (ADR-T104-1) — no room for the value.
- **One 1 × 2 grid per node** — one shape, but grid cells are the place
  for arrays; a rectangle's own text slot is what the Text tool fills
  with one click.
- **Value box beside the range** — the tree gets twice as wide, and it
  is already the wide direction.

## Consequences

- A segment tree has `2(2n − 1)` boxes and `2n − 2` lines.
- `segtree: 100` is about as wide as before and taller.

## Rollback plan

A new ADR restoring circles, with `git revert` of the commits of this
ADR.
