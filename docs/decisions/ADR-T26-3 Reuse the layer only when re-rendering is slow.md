---
id: ADR-T26-3
title: Reuse the layer only when re-rendering is slow
status: accepted
kind: decision
date: 2026-10-03
task: T26
builds_on: [ADR-T26-1]
amends: [ADR-T26-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T26-3 Reuse the layer only when re-rendering is slow

## Context

With [[ADR-T26-1 Reuse the document layer during pan and zoom]] every pan
or zoom blits the cached layer, whatever the scene. The owner tried it
on a page with few shapes: panning felt laggy, because content beyond the
margin only appears after the 100 ms settle, while a full re-render of
that page would have taken a millisecond or two. The blit only pays off
when a full re-render is slower than a frame.

How much a re-render costs depends on the number of visible shapes and on
the zoom: the same 5 120 strokes take ≈ 15 ms zoomed out and 40–60 ms at
medium zoom ([[ADR-T24-5 Redraw budgets]]).

## Decision

- The shell measures the CPU time of every `Full` re-render (always, not
  only with `DRAW_FRAME_TIMES`) and keeps the last one.
- `reuse_if_slow(redraw, last_full)` turns `Moved` into `Full` while the
  last full re-render took less than `REUSE_ABOVE` = 8 ms (half a 60 Hz
  frame), or when nothing has been measured yet. It is applied before
  `settle`. Every other plan is unchanged.
- So a cheap scene re-renders on every pan or zoom event as before T26.
  An expensive one blits; when it settles, the settle re-render is
  measured again, so zooming out of a heavy view returns to re-rendering
  for the next gesture.

## Alternatives considered

- **A visible-shape count threshold** (the owner's suggestion) — easy to
  test, but the cost per shape varies several times with zoom and stroke
  length, so one count is too low at one zoom and too high at another.
  The measured time is the quantity that matters.
- **Shorter `SETTLE` only** — still shows blur and edges on cheap scenes.

## Consequences

- Small documents behave exactly as before T26 (sharp, no settle delay).
- The measurement is CPU submission time only; GPU fill is not included,
  as for the frame timer.
- One slow frame (a spike, or the first frame's GL warm-up) switches the
  next gesture to blitting until its settle re-render is measured again.

## Rollback plan

`shell::app`: drop the `reuse_if_slow` call (or set `REUSE_ABOVE` to
zero) to blit on every camera change as in ADR-T26-1. Record as a new ADR.
