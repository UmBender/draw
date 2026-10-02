---
id: ADR-T21-2
title: Eraser mode chosen on press
status: accepted
kind: decision
date: 2026-10-01
task: T21
builds_on: [ADR-T21-1]
amends: [ADR-T21-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T21-2 Eraser mode chosen on press

## Context

[[ADR-T21-1 Cell fills and a two-mode eraser]] decides per touched shape:
outline → remove, filled inside → clear. Writing the tests showed the flaw:
a drag across several filled cells always crosses grid lines, so it would
delete the grid instead of clearing the cells. The same happens when
rubbing out a filled circle and slipping over its outline.

## Decision

The eraser picks one **mode per gesture**, on `Down`:

- **Clear fills** — if the press point lies inside a filled `Rect`/
  `Ellipse` (and not on its outline) or inside a filled grid cell (and not
  on a grid line). The whole drag then clears every fill it passes over —
  cell fills and closed-shape fills — and **never removes** a shape.
- **Remove** — otherwise. The drag removes every shape it hits, with the
  existing `Shape::hit` (a filled inside still counts), exactly as before
  T21.

The preview hides affected shapes; in clear mode it draws them again
without the cleared fills. Release commits one transaction, one undo step.
This amends the "two effects per shape" rule of ADR-T21-1; everything else
there stands. `Shape::hit_outline` is still needed to tell a press on an
outline from a press inside.

## Alternatives considered

- **Only the outer border removes a grid** — rubbing out a filled circle
  that crosses its outline would still delete it.
- **A modifier for clear mode** — the press point already says what the
  user means; no extra key to learn.

## Consequences

- Behaviour before T21 is unchanged unless the press lands on a fill.
- To delete a filled shape, press on its outline or outside it.

## Rollback plan

Revert with ADR-T21-1's rollback (the T21 commits).
