---
title: Taming shaky input
step: 4
requires: ["[[01 2D vectors, AABBs and point-segment distance]]"]
feature: "[[Anti-tremor strokes]]"
code: ["src/core/smoothing.rs"]
tags: [tutorial]
---

# Taming shaky input

> **Learning path step 4.** Requires: [[01 2D vectors, AABBs and point-segment distance]] ·
> Next: *9 Building drawing tools* (the pen feeds a `Smoother`), *13 Property-based testing and fuzzing*

## Why this matters here

The person `draw` is built for has a hand tremor. A mouse or touchpad reports
the pointer dozens of times per second, and every tiny shake ends up in the
stroke: the line looks hairy, and it is stored as hundreds of nearly useless
points. Three small, classic techniques fix both problems, each attacking a
different kind of noise:

| Stage | Removes | When |
|-------|---------|------|
| Resampling | micro-jitter, duplicate events | live, per pointer event |
| Exponential moving average (EMA) | wobble across several points | live, per kept point |
| Ramer–Douglas–Peucker (RDP) | redundant points along straight-ish runs | once, on release |

## The concept

### 1. Resampling: ignore moves that are too small

Keep a point only if it is at least `min_dist` away from the last point we
kept. Measure against the last *kept raw* point, not the previous event —
otherwise a slow drag of many 1 px steps would never be kept at all.

```
min_dist = 2.5

raw x:   0     0.8    1.9    2.7    3.1    5.6
         keep  drop   drop   keep   drop   keep
               (0.8)  (1.9)  (2.7)  (0.4)  (2.9)   ← distance to last kept
```

A shake of a pixel or two around the same spot now produces no points at all.

### 2. EMA: average the wobble out

An exponential moving average replaces each point by a blend of where we were
and where the hand says we are:

```
s = s_prev + α · (raw − s_prev)        0 < α ≤ 1
```

`α = 1` is "no smoothing"; `α = 0.4` moves 40 % of the way each step. It is a
low-pass filter: slow, intentional motion passes through, fast back-and-forth
shaking cancels out. Take a tremor that alternates `y = +2, −2, +2, …` with
`α = 0.4`:

```
raw y:     +2     −2     +2     −2     +2    …
smooth y:  +2   +0.40  +1.04  −0.18  +0.69   …  → settles at ±0.5
```

The swing shrinks from ±2 to ±0.5 — a 16× drop in variance — with one
multiply-add per point and no memory beyond `s_prev`. The price is lag: the
smoothed line trails the cursor, more so for smaller `α`. That is why the
stroke's *raw* last point is appended at the end, so the line reaches where
the pen actually lifted.

### 3. Ramer–Douglas–Peucker: keep only the points that matter

After smoothing, a straight-ish run still has many points. RDP keeps the fewest
points such that no dropped point is farther than `ε` from the result:

1. Draw a segment from the first to the last point.
2. Find the point farthest from that segment.
3. If it is within `ε`, every interior point can go. Otherwise keep it and
   repeat on the two halves.

```
ε = 0.5          C(2,3)
                  /\
                 /  \
         B(1,1.3)    D(3,1.4)
               /      \
        A(0,0)          E(4,0)

Segment A–E: farthest is C (3.0)  > ε → keep C, split.
Segment A–C: B is 0.11 away      ≤ ε → drop B.
Segment C–E: D is 0.06 away      ≤ ε → drop D.
Result: A, C, E
```

Two properties make RDP safe: the endpoints are always kept, and every removed
point is within `ε` of the output (each was within `ε` of the segment that
replaced it). It is also *idempotent*: simplifying the result again changes
nothing, because the same farthest points are found in the same order.

The textbook version is recursive. A recursive split can nest as deep as the
number of points (a zig-zag where every split lands next to an endpoint), and a
long stroke could overflow the call stack. `draw` uses an explicit stack of
`(first, last)` index spans on the heap instead.

## How draw implements it

All of it is in `src/core/smoothing.rs`; distances use
`geom::distance_to_segment` from [[01 2D vectors, AABBs and point-segment distance]].

- `SmoothingLevel` — `Off`, `Low`, `Medium` (default), `High`. `next()` cycles
  them for the `S` key, `label()` names them for the UI, and `params()` returns
  a `SmoothingParams { min_dist_px, alpha, epsilon_px }` from the table in
  [[ADR-0007 Anti-tremor pipeline]].
- `Smoother::new(params, px_to_world)` converts the pixel tolerances to world
  units once (`px_to_world = 1 / zoom`), so smoothing feels identical at any
  zoom. It also sanitises input: a NaN or non-positive scale becomes `1.0`,
  `alpha` is clamped to `[MIN_ALPHA, 1]`, negative distances become `0`.
- `Smoother::push(raw)` is stages 1 and 2. It returns `false` for a non-finite
  point, an exact duplicate, or a point closer than `min_dist`. Otherwise the
  point is blended as `prev·(1−α) + raw·α` — the same formula rearranged into a
  convex combination, which cannot overflow for finite inputs — and stored.
  With `α = 1` the raw point is stored as-is, so `Off` is an exact passthrough.
- `Smoother::points()` is the live polyline the pen tool previews.
- `Smoother::finish(self)` appends the last raw point (unless it is already
  the last point) and calls `simplify_rdp` with `ε` in world units.
- `simplify_rdp(points, eps)` marks endpoints in a `keep: Vec<bool>`, pushes
  `(0, n−1)` on a stack, and loops: pop a span, find its farthest interior
  point (starting the running maximum at `eps`, so "nothing found" means
  "drop them all"), mark it and push both halves. Finally it collects the kept
  points in order. `eps` that is NaN or `≤ 0` returns the input unchanged.

The tests next to the code pin the behaviour: unit tests for the level table,
resampling, EMA jitter reduction and the end point; proptests for the RDP
contract (`rdp_keeps_endpoints`, `rdp_error_bounded`, `rdp_idempotent`,
`rdp_is_subsequence_and_never_grows`) and for `smoother_output_is_finite` with
NaN/∞ mixed into the input. `rdp_huge_input_does_not_overflow_stack` runs a
10 000-point sawtooth on a 256 KiB thread: a recursive RDP would crash there.

## Try it

1. In `ema_reduces_jitter`, print `raw_var` and `smooth_var`, then switch
   `medium()` to `SmoothingLevel::High.params()`. Predict the steady-state
   swing for `α = 0.25` before running (hint: solve `a = −a + α(2 + a)`).
2. Make `simplify_rdp` recursive (a helper that calls itself on both halves)
   and run `cargo test rdp_huge`. Watch the test binary abort with a stack
   overflow, then revert.
3. Change the resampling check to compare against the previous *event* instead
   of the last *kept* point. Which test fails, and why would slow strokes break?
4. Add a test: a circle of 360 points through `simplify_rdp` with `eps = 0.5`
   returns far fewer points, and every original point is within `0.5` of the
   result (reuse `distance_to_polyline`).

## Further reading

- [[ADR-0007 Anti-tremor pipeline]] — the decision, table and alternatives.
- U. Ramer, *An iterative procedure for the polygonal approximation of plane
  curves* (1972); D. Douglas & T. Peucker (1973).
- G. Casiez, N. Roussel, D. Vogel, *1 € Filter* (CHI 2012) — the adaptive
  alternative to a fixed EMA.
