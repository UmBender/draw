---
id: ADR-T49-1
title: Undo history capped by bytes
status: accepted
kind: decision
date: 2026-11-03
task: T49
builds_on: ["[[ADR-0005 Undo via transaction log]]", "[[ADR-T24-1 Document layer keyed by a document revision]]"]
amends: ["[[ADR-0005 Undo via transaction log]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T49-1 Undo history capped by bytes

## Context

[[ADR-0005 Undo via transaction log]] caps the undo stack at 500
transactions. A transaction holds whole shapes: a move stores each moved
shape twice (`Replace { before, after }`), a duplicate or delete stores
every shape once. Moving a large selection again and again therefore keeps
up to 1 000 copies of it. A soak probe (T49) grew a 1 000-stroke session
from 4 MiB to 140 MiB; a 5 000-stroke scene reaches gigabytes, where the
OOM killer ends draw with `SIGKILL` and leaves no trace in the T47 log.

The editor also detected document changes by comparing the stack lengths
([[ADR-T24-1 Document layer keyed by a document revision]]), which only
works while a commit drops at most one entry.

## Decision

- `Shape::heap_bytes` and `Transaction::heap_bytes` estimate the heap a
  shape and a transaction hold (by capacity, plus the edit buffer).
- `History` keeps the total over both stacks and, after each push onto the
  undo stack, drops the oldest undo entries while the total is over
  `HISTORY_BYTES` (64 MiB) or the count over `HISTORY_LIMIT` (500). The
  newest entry is always kept, so the last action can be undone even when
  it alone is over budget.
- `History::changes` counts every commit, undo and redo that changed the
  document. `Editor::track_document` bumps the document revision when that
  count moves, instead of comparing stack lengths.

## Alternatives considered

- **Share shapes between the document and the history (`Rc<Shape>`)** —
  removes the copies at the root, but changes `Edit`, `Document` and every
  tool: a design change for [[T50 Version 2.0.0 refactor]], not a fix.
- **Store moves as offsets** — fixes moves only; duplicate, delete and
  clear still copy shapes.
- **Lower the entry limit** — punishes small documents for what large
  selections do.

## Consequences

- Memory held by undo is bounded at about 64 MiB plus the newest entry.
- With very large selections the undo depth drops below 500 (about eight
  full moves of a 4 MiB document). Small documents keep 500 steps.
- The estimate ignores allocator overhead, so the real figure is somewhat
  higher; it is a budget, not an exact measure.

## Rollback plan

Remove the byte check from `History::push_undo` (the counter and the
estimates are harmless) with a `revert(T49)` commit and a rollback ADR
that reverts this one.
