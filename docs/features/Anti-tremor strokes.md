---
title: Anti-tremor strokes
task: "[[T04 Stroke smoothing]]"
adrs: ["[[ADR-0007 Anti-tremor pipeline]]"]
tutorial: "[[04 Taming shaky input]]"
shortcuts: ["S"]
tags: [feature]
---

# Anti-tremor strokes

## What it does

Freehand pen strokes are steadied while you draw, so a shaky hand still
produces clean lines. Small jitters are dropped, the remaining wobble is
averaged out live, and when you release the pen the stroke is simplified to the
few points that matter. The line always ends exactly where the cursor was.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Draw a smoothed stroke | Pen tool (`P`), drag |
| Cycle strength off → low → medium → high | `S` |

| Level | Feels like |
|-------|------------|
| Off | Raw input; only exact duplicate points are removed |
| Low (default) | Light steadying, almost no lag ([[ADR-T14-2 Low smoothing by default]]) |
| Medium | Good balance for everyday drawing |
| High | Strongest steadying for a very shaky hand; the line trails the cursor a little more |

Status: this note covers the smoothing engine ([[T04 Stroke smoothing]]). The
pen tool that feeds it is wired in [[T09 Creation tools]], the `S` binding in
[[T11 Keymap and macros]] (see [[Keymap]]).

## How it works

`src/core/smoothing.rs` implements the three stages of
[[ADR-0007 Anti-tremor pipeline]]:

1. **Resample** — `Smoother::push` ignores a point closer than `min_dist` to the
   last kept raw point (and any exact duplicate).
2. **EMA** — each kept point is pulled a fraction `α` of the way from the
   previous smoothed point towards the raw point. `Smoother::points` is the
   live preview.
3. **RDP** — `Smoother::finish` appends the last raw point, then
   `simplify_rdp` (Ramer–Douglas–Peucker, iterative) removes every point within
   `ε` of the simplified line.

`SmoothingLevel::params` holds the table (`min_dist` px / `α` / `ε` px):
Off 0 / 1.0 / 0, Low 1.5 / 0.6 / 0.8, Medium 2.5 / 0.4 / 1.5, High 4.0 / 0.25 / 2.5.
Distances are in screen pixels and multiplied by `px_to_world` (`1 / zoom`) when
the `Smoother` is created, so the feel is the same at every zoom level.

Robustness: non-finite points are ignored; a bad `px_to_world` falls back to
`1.0`; `α` is clamped to `[MIN_ALPHA, 1]` so a stroke can never freeze; RDP
uses an explicit stack, so even a pathological 10 000-point zig-zag runs on a
small thread stack.

## Limits

- No curve fitting (Catmull-Rom, Bézier): strokes are polylines. A render-time
  stage could be added later (see ADR-0007 alternatives).
- No pressure or velocity-adaptive smoothing (One-Euro filter was rejected for
  its extra tuning parameters).
- EMA adds lag that grows with the level; the true end point is restored on
  release, but the live preview trails the cursor slightly.
- The parameter values are starting points; [[T14 Performance and release validation]]
  tunes them with the user through an amending ADR.
