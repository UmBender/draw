---
id: ADR-T101-2
title: Rows and columns without overlap
status: accepted
kind: decision
date: 2026-11-10
task: T101
builds_on: [ADR-T101-1]
amends: [ADR-T101-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T101-2 Rows and columns without overlap

## Context

ADR-T101-1 made `Alt+H` / `Alt+V` a classic "distribute spacing": the
outer shapes fixed, equal gaps, negative when the shapes overlap, the
other axis untouched. Trying it, the owner expected the keys to lay the
selection out as a clean row (column): nodes drawn roughly in a line
stayed overlapped and off the line, and needed `Alt+↑` first. The owner
chose a row / column layout that never overlaps.

## Decision

Amends the **Distribute** rule of ADR-T101-1; keys, boxes, transaction
and guards are unchanged.

- **Needs two or more** selected shapes (was three): a pair can overlap
  too.
- **Order.** As before: near edge, then far edge, then stacking order.
  The first shape of that order never moves.
- **Cross axis.** Every shape is moved so its centre on the other axis
  meets the first shape's (`Alt+H`: same centre `y`; `Alt+V`: same
  centre `x`).
- **Gap.** `natural = (last.min − first.max − Σ middle sizes) / (n − 1)`
  as before, and `min_gap = ¼ ×` the largest selected size along the
  axis. `gap = max(natural, min_gap)`. Each shape after the first is
  placed `gap` after the previous one's far edge. So with room enough
  the last shape stays put and the gaps are equal as before; when the
  shapes would overlap or crowd, the row grows away from the first
  shape and nothing overlaps.

## Alternatives considered

- **Keep classic distribute** (ADR-T101-1) — standard in vector
  editors, but overlap survives and a row needs two keys.
- **Fixed minimum gap in world units** — wrong at every zoom level and
  shape size; a quarter of the largest shape reads as "a little air"
  for nodes, cells and text alike.
- **Centre on the selection box** instead of the first shape — every
  shape moves; anchoring on the first keeps one shape still, as the
  order already does.
- **Equal gaps without the cross-axis alignment** — the owner wanted a
  line, not only spacing.

## Consequences

- A second press is a no-op (the layout is a fixed point), as before.
- The last shape can now move, so the row can grow past the old
  selection box.

## Rollback plan

A new ADR restoring the ADR-T101-1 rule, with `git revert` of the
commits of this ADR.
