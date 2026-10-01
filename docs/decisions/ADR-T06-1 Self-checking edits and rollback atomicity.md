---
id: ADR-T06-1
title: Self-checking edits and rollback atomicity
status: accepted
kind: decision
date: 2026-10-01
task: T06
builds_on: [ADR-0005]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T06-1 Self-checking edits and rollback atomicity

## Context

[[ADR-0005 Undo via transaction log]] makes every mutation an `Edit` inside a
`Transaction` and requires `Document::apply` to be atomic. It does not say how
atomicity is achieved, nor how much an edit is checked before it runs. A
silently wrong edit (removing the wrong shape at an index, replacing a shape
that has changed) would corrupt the undo history without any visible error.

## Decision

- `Remove` and `Replace` carry the full shape they expect to find; `apply`
  verifies the stored `(id, shape)` (resp. `before`) matches exactly and fails
  with `ApplyError::Mismatch` otherwise. `Insert` fails on a duplicate id.
  Inserted and replacing shapes must be finite.
- Atomicity is by **rollback**: edits are applied one by one; on the first
  failure the already applied edits are undone with their inverses (which are
  valid by construction) in reverse order, and the id counter is restored.
- The id counter only moves forward; applying an `Insert` with an id at or past
  the counter bumps it, so ids are never reused even for foreign transactions.

## Alternatives considered

- **Validate on a clone of the document** — simple, but copies every shape on
  every action; the same cost ADR-0005 rejected for snapshots.
- **Dry-run simulation of indices** — avoids the copy, but duplicates the apply
  logic and is easy to get subtly wrong.
- **Trust edits (index only)** — cheapest, but a stale transaction corrupts
  the document silently.

## Consequences

- Applying costs one extra shape comparison per `Remove`/`Replace`
  (proportional to the shape size), never a document copy.
- `History::undo`/`redo` can detect a document mutated behind their back and
  refuse (return `false`) instead of corrupting it.
- Builders must snapshot the current shapes into the edits they create.

## Rollback plan

Drop the equality checks in `Document::apply` (`src/core/document.rs`); the
public API and the transaction format stay the same.
