---
id: ADR-T135-1
title: Regions live in the machine module
status: accepted
kind: decision
date: 2026-11-12
task: T135
builds_on: [ADR-T134-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T135-1 Regions live in the machine module

## Context

T135 moves the modal overlays and the view regions into the machine. The
editor (10k lines, with its tests) still does the work behind the routing
table (ADR-T134-1), and its methods read these fields everywhere, so they
cannot be handed in from outside without rewriting every method.

## Decision

- `machine/regions.rs` defines `Modal` (`Closed | Palette | Sheet`: one
  value, so the two overlays are never open together by construction),
  `Pages` (current index and the slots of the other pages) and `View`
  (camera and selection of the current page).
- `Editor` holds one field of each type instead of the six loose fields.
  The machine module owns the types and their invariants; the editor reads
  and writes them through their methods.
- `Machine::interaction` reads the `Modal` region. While a modal is open the
  editor routes every event to it and none reaches a tool.
- UNATTENDED CHOICE: the types move, the storage stays inside `Editor`
  (the adapter of ADR-T134-1). Moving the storage to `Machine` waits for
  the tasks that move the tool code (T136).

## Alternatives considered

- **Store the regions in `Machine` and pass them to `Editor::handle`** —
  rewrites every editor method and test at once.

## Consequences

Snapshots are unchanged (same fields written). Overlay exclusivity is a type
property. A later task can lift the storage out without changing the types.

## Rollback plan

`git revert` the T135 behaviour commit.
