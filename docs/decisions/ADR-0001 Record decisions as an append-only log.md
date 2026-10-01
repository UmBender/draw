---
id: ADR-0001
title: Record decisions as an append-only log
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: []
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0001 Record decisions as an append-only log

## Context

The project is built task by task, sometimes by parallel agents. Decisions
must be traceable in the order they were made, and undoing one must not erase
the reasoning behind it.

## Decision

Architecture decisions are ADRs in `docs/decisions/`, handled transactionally
as described in [[Decision Protocol]]: append only, explicit chain links
(`builds_on`, `amends`, `supersedes`, `reverts`), rollbacks as new ADRs, order
fixed by [[Decision Log]].

## Alternatives considered

- **Editing a single design doc in place** — loses history and the reason a
  choice was reversed.
- **Relying on git history only** — reasons get buried in diffs and are not
  linkable from Obsidian.

## Consequences

- Slightly more notes; every rollback costs one ADR.
- [[Architecture]] stays a readable "current state" view and links its ADRs.

## Rollback plan

Not expected. If dropped, a rollback ADR would freeze the log as-is.
