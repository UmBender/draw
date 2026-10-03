---
id: ADR-T26-2
title: Gesture frame budget
status: proposed
kind: decision
date: 2026-10-03
task: T26
builds_on: [ADR-T26-1]
amends: [ADR-T24-5]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T26-2 Gesture frame budget

## Context

[[ADR-T24-5 Redraw budgets]] left pan and zoom at medium zoom over budget
(40–60 ms per event). [[ADR-T26-1 Reuse the document layer during pan and zoom]]
replaces those re-renders with a blit of the cached layer and one
re-render when the gesture settles.

## Decision

- A pan or zoom frame during a gesture (`DRAW_FRAME_TIMES` line
  `draw: moved re-render …`) costs **< 2 ms** CPU on the ADR-T24-5 scene
  at any zoom.
- The settle re-render (`full` line after the gesture) keeps the ADR-T24-5
  full re-render budgets; it now includes the layer margin and is listed
  separately in task logs.
- Status stays `proposed` until the owner measures on the target; the
  numbers go in [[T26 Pan and zoom without re-rendering]]'s *Log*.

## Alternatives considered

- **Budgeting the settle re-render below 16 ms** — it happens once per
  gesture, and shrinking it is geometry work
  ([[T25 Linear-time selection with large documents]], option 4–5).

## Consequences

- Smooth pan and zoom no longer depend on document size; the remaining
  stall is one re-render ≈ 100 ms after the gesture ends.

## Rollback plan

Budgets are text. Changing one is a new ADR amending this one.
