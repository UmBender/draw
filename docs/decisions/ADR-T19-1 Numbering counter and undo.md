---
id: ADR-T19-1
title: Numbering counter and undo
status: accepted
kind: decision
date: 2026-10-01
task: T19
builds_on: [ADR-T16-1, ADR-T16-3, ADR-T08-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T19-1 Numbering counter and undo

## Context

With numbering on, every new rectangle or ellipse takes the label
`next_number` and the counter moves on ([[ADR-T16-1 Grid shape and shape labels]]).
Undoing a numbered shape must give its number back so the next node reuses
it, and redo must take it again. The counter lives in the editor's `Helpers`
([[ADR-T16-3 Helper settings and hooks]]), but shapes are committed by the
shape tool straight into `History` through `ToolCtx::commit`
([[ADR-T08-1 Tool context and gesture overlay]]): the editor never sees the
transaction, and `History` (not owned by T19) stores no side data and can
drop old entries at its limit, so a stack of counter snapshots kept beside it
would drift.

## Decision

The counter is **derived from label counts around each step**, in the editor:

- The shape tool labels the shape it commits (and its preview) with
  `numbering::label_new`, reading `next_number` from `ToolCtx::style`; tools
  never write the counter.
- **Forward rule** — after a tool gesture ends (with numbering on) and after
  a redo: if the number of shapes labelled `next_number` grew, the counter
  becomes `next_number + 1` (saturating).
- **Backward rule** — after an undo: if `next_number > 1` and the number of
  shapes labelled `next_number − 1` shrank, the counter becomes
  `next_number − 1`.
- Counts are compared, not presence, so a shape numbered after a reset (`1`
  again while an old `1` exists) still advances, and its undo still rolls
  back. Each step moves the counter by at most one.
- `Shift+N` (reset) and `N` (toggle) are not undoable; clipboard commands
  never touch the counter or labels.

The rules are pure functions in `numbering` (`label_count`, `advance`,
`roll_back`); the editor only takes the counts before and after.

## Alternatives considered

- **Counter snapshot stored per transaction** — needs `History` to carry
  side data or expose commits; that file belongs to T06 and would change for
  one helper.
- **Counter = max label + 1** — makes `Shift+N` meaningless while labelled
  shapes exist.
- **Renumbering on undo** — rewrites unrelated shapes; surprising.

## Consequences

- O(n) label scans on undo, redo and gesture end — negligible next to
  rendering.
- Undoing a paste that contains a copy of the newest number (`next − 1`)
  also rolls the counter back; the next node then repeats that number. Rare
  and harmless (labels are free-form).
- A redo after a reset can advance the counter only when it re-adds the
  current `next_number`.

## Rollback plan

Revert T19's editor hook (counter then never moves on undo); a later ADR can
move the counter into `History` side data if this proves confusing.
