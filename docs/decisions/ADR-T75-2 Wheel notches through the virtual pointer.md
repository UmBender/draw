---
id: ADR-T75-2
title: Wheel notches through the virtual pointer
status: accepted
kind: decision
date: 2026-11-08
task: T75
builds_on: []
amends: ["[[ADR-T75-1 Headless sway session for end-to-end tests]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T75-2 Wheel notches through the virtual pointer

## Context

[[ADR-T75-1 Headless sway session for end-to-end tests]] has the harness's
virtual pointer send no events, with `swaymsg seat - cursor` driving them
all. `swaymsg seat - cursor press button4` reports success, but the app
never receives a wheel event: no `wheel` record appeared in a trace of the
pan and zoom case. The most likely cause is that sway sends a discrete
value of 1, which wlroots accumulates up to a full notch of 120 before
telling a client that binds the low-resolution axis. That reading comes
from the protocol and memory of the source, not from a test of it.

## Decision

A `wheel` step sends one notch on the virtual pointer the harness already
holds: `axis_source` wheel, a vertical `axis_discrete` of 15 units and one
notch, then `frame`. It reaches the app as a real wheel would. Moves,
presses and releases stay with `swaymsg`.

## Alternatives considered

- **120 `swaymsg` commands per notch** — depends on an accumulator detail
  and is not what a wheel sends.
- **Leave zoom out of the case** — zoom is part of the listed gesture.

## Consequences

- The harness writes three more protocol messages, covered by
  `wheel_message_is_source_discrete_and_frame`.

## Rollback plan

Remove the `Wheel` step and `wheel_message` from `tests/e2e.rs` in a
`revert(T75)` commit, under a new ADR with `kind: rollback`.
