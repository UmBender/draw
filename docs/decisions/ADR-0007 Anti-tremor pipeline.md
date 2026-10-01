---
id: ADR-0007
title: Anti-tremor pipeline
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

# ADR-0007 Anti-tremor pipeline

## Context

The user's hand shakes, so raw pointer input gives jagged strokes. Smoothing
has to work live (no lag that feels wrong) and the stored stroke should be
compact.

## Decision

Freehand strokes go through three stages (`core::smoothing`):

1. **Resample (live)** — drop points closer than `min_dist` *screen* pixels to
   the last kept point. Removes micro-jitter and duplicate events.
2. **Exponential moving average (live)** — `p' = p'_prev + α (p − p'_prev)`.
   Smaller `α` = smoother but more lag. The raw last point is kept as the
   stroke's end so the line reaches where the cursor actually is.
3. **Ramer–Douglas–Peucker (on release)** — simplify with tolerance `ε`
   *screen* pixels (converted to world units by the current zoom), keeping
   endpoints.

Strength levels (cycled with `S`, see [[Keymap]]):

| Level | `min_dist` px | `α` | `ε` px |
|-------|---------------|-----|--------|
| Off | 0 | 1.0 | 0 |
| Low | 1.5 | 0.6 | 0.8 |
| Medium (default) | 2.5 | 0.4 | 1.5 |
| High | 4.0 | 0.25 | 2.5 |

Values are starting points, tuned in [[T14 Performance and release validation]]
via an amending ADR if needed.

## Alternatives considered

- **Catmull-Rom / Bézier fitting** — nicer curves, more code and more
  ways to overshoot; can be added later as a render-time stage.
- **Only RDP** — compact but keeps the tremor's spikes during drawing.
- **One-Euro filter** — adaptive and good, but two more tuning parameters;
  EMA plus resampling is simpler to reason about.

## Consequences

- Smoothing is pure and deterministic, so it is easy to test and fuzz.
- Tolerances are in screen pixels, so smoothing feels the same at any zoom.

## Rollback plan

Each stage is a separate function; dropping or replacing one is an amending
ADR.
