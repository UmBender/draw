---
id: ADR-0005
title: Undo via transaction log
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0004]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0005 Undo via transaction log

## Context

Undo/redo is essential even in a minimal tool. Some actions touch many shapes
at once (paste, delete selection, clear canvas, move).

## Decision

Every document change is an `Edit` (`Insert { index, id, shape }`,
`Remove { index, id, shape }`, `Replace { id, before, after }`). One user
action = one `Transaction` (`Vec<Edit>`). `History` keeps an undo stack and a
redo stack of transactions; undo applies the inverse edits in reverse order.
A new transaction clears the redo stack. The history is capped (default 500
transactions) to bound memory.

## Alternatives considered

- **Snapshotting the whole document** — simplest, but copies everything on
  each action; bad for large scratch sessions on a weak laptop.
- **Persistent data structures** — needs a dependency or lots of code.

## Consequences

- Every mutation must go through `Document::apply(Transaction)`; tools never
  mutate the shape list directly.
- The invariant "undo all → empty, redo all → same" is fuzzable.

## Rollback plan

Falling back to snapshots would keep the `History` API and change only its
internals.
