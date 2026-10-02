---
id: ADR-0016
title: Arrow head proportional to width
status: accepted
kind: decision
date: 2026-10-02
task:
builds_on: [ADR-0004, ADR-0013]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0016 Arrow head proportional to width

## Context

[[T05 Shape model]] sized the arrow head as
`max(ARROW_HEAD_MIN_LENGTH, ARROW_HEAD_LENGTH_PER_WIDTH * width)` with an
8 world-unit minimum. Under [[ADR-0013 World-space widths and zoom limits]]
a new shape gets `width = chosen_px / zoom`, so an arrow drawn zoomed in
has a thin world width while its head stays at the 8-unit floor: the shaft
shrinks with the zoom, the head does not, and the head looks oversized.

## Decision

- The head length is `ARROW_HEAD_LENGTH_PER_WIDTH * width` (negative or NaN
  widths count as 0), with no absolute minimum. `ARROW_HEAD_MIN_LENGTH` is
  removed.
- An arrow looks the same on screen whatever zoom it was drawn at: scaling
  an arrow and its width by `k` scales its head by `k`.

## Alternatives considered

- **Keep the minimum, in screen pixels** — the shape model has no zoom, and
  a shape must look the same after zooming (ADR-0013).
- **Store the head size on the shape** — extra state for no visible gain.

## Consequences

- Thin arrows have smaller heads than before: a 1 px arrow gets a 4 px head
  instead of 8 px. If that proves too small, raise
  `ARROW_HEAD_LENGTH_PER_WIDTH`.

## Rollback plan

Revert the commit that removed `ARROW_HEAD_MIN_LENGTH`.
