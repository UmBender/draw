---
id: ADR-T101-3
title: Compact rows and columns
status: accepted
kind: decision
date: 2026-11-10
task: T101
builds_on: [ADR-T101-2]
amends: [ADR-T101-2]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T101-3 Compact rows and columns

## Context

With ADR-T101-2, spread-out shapes kept their wide equal gaps (the last
shape fixed) and only crowded ones were pushed apart. Trying it, the
owner wanted every press to give a tight row whatever the starting
spacing.

## Decision

Amends the **Gap** rule of ADR-T101-2: `gap = ¼ ×` the largest selected
size along the axis, always. The first shape stays put, every other one
(the last included) is placed `gap` after the previous one's far edge
and centred on the first one's line. Order, two-shape minimum, cross
axis, transaction and guards are unchanged.

## Alternatives considered

- **Keep the larger of the natural and the minimum gap** (ADR-T101-2) —
  keeps a spread-out layout, which the owner did not want.
- **Gap from the two neighbours' sizes** — uneven gaps next to a big
  shape; one gap per row reads cleaner.

## Consequences

- The row can shrink as well as grow; a second press is still a no-op.

## Rollback plan

A new ADR restoring the ADR-T101-2 gap, with `git revert` of the
commits of this ADR.
