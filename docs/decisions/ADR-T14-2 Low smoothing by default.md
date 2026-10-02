---
id: ADR-T14-2
title: Low smoothing by default
status: accepted
kind: decision
date: 2026-10-02
task: T14
builds_on: [ADR-0007]
amends: [ADR-0007]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T14-2 Low smoothing by default

## Context

[[ADR-0007 Anti-tremor pipeline]] made **Medium** the default strength and
left the values to be tuned in [[T14 Performance and release validation]].
The user tried every level on the target laptop: **Low** feels right, Off
lets the tremor through, and the stronger levels are not needed.

## Decision

- The default `SmoothingLevel` is **Low** (`min_dist` 1.5 px, `α` 0.6,
  `ε` 0.8 px).
- The values of every level stay as in ADR-0007; the `S` cycle order
  (off → low → medium → high) is unchanged.

## Alternatives considered

- **Keep Medium as default** — the user would press `S` three times at the
  start of every session to reach Low.
- **Give Medium the Low values** — the labels would lie and High would be
  a bigger jump.

## Consequences

- A new session draws with less lag; Medium and High stay one or two `S`
  presses away for a shakier day.
- Tests that pinned the Medium default now pin Low.

## Rollback plan

Move `#[default]` back to `SmoothingLevel::Medium` in `core::smoothing` and
revert the two test expectations, with a rollback ADR reverting this one.
