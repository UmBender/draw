---
id: ADR-T24-5
title: Redraw budgets
status: accepted
kind: decision
date: 2026-10-03
task: T24
builds_on: [ADR-T24-1, ADR-T24-2, ADR-T24-3, ADR-T24-4]
amends: [ADR-T14-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T24-5 Redraw budgets

## Context

[[ADR-T14-1 Performance budgets]] covers the headless hot paths only.
Drawing was where a large session got clunky
([[T24 Smooth redraws with large documents]]). The redraw cost cannot be
timed in `tests/perf.rs`: it needs a GL context and the target's GPU
driver. `DRAW_FRAME_TIMES=1` prints the CPU time of each re-render instead.

The owner's numbers on the target (i5-8250U, UHD 620, Wayland). The scene
is 10 pen strokes, then `Ctrl+A` and `Ctrl+D` ×9, giving 5 120 strokes.
The time is for a pan/zoom `full` re-render unless noted.

| Re-render | Before T24 | Batching (ADR-T24-3) | Strips (ADR-T24-4) |
|-----------|------------|----------------------|--------------------|
| Overlay only (circle drag) | whole frame | 0.12–0.17 ms | — |
| Full, zoomed out | 33.8 ms | 19–46 ms (≈ 26 typical) | ≈ 15 ms, max 20 |
| Full, medium zoom | — | ≈ 210 ms | 40–60 ms |

Medium zoom costs more because strokes are wider on screen and keep more
of their points. Some of every frame also goes to the O(selection ×
document) lookups that [[T25 Linear-time selection with large documents]]
removes.

## Decision

- Redraw budgets are **manual checks** on the target with
  `DRAW_FRAME_TIMES=1`, using the scene above, before a release:
  - overlay-only re-render: **< 2 ms**. Met.
  - full re-render, zoomed out to see the whole scene: **< 16 ms**. Met
    on typical frames; single frames reach 20 ms.
  - full re-render at medium zoom: **< 16 ms**. **Not met** (40–60 ms).
    It is carried by T25 and by a gesture task that shows the cached
    layer moved/scaled during pan and zoom.
- A task that changes `shell::render` or `shell::app` records these
  numbers in its log.

## Alternatives considered

- **An automated redraw benchmark** — needs a window and GL in the test
  run, so it does not run headless or in `scripts/check.sh`.
- **Keep shrinking geometry inside T24** — each remaining cut is smaller.
  The gesture approach removes the per-movement re-render altogether.

## Consequences

- Pan and zoom at medium zoom on a 5 000-stroke scene still drop frames
  until T25 and the gesture task land.
- The budgets are for the target laptop; other machines only compare
  against their own earlier numbers.

## Rollback plan

Budgets are text. Changing one is a new ADR amending this one.
