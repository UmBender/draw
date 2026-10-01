---
title: Snapping and alignment guides
step: 17
requires: ["[[09 Building drawing tools]]", "[[16 Growing a data model without breaking it]]"]
feature: "[[Snapping]]"
code: ["src/core/snap.rs", "src/core/tools/shape_tool.rs", "src/shell/render.rs"]
tags: [tutorial]
---

# Snapping and alignment guides

> **Learning path step 17.** Requires: steps 9 and 16 · Next: step 18

## Why this matters here

A competitive-programming sketch is mostly boxes and arrows: a segment
tree, a DP table, a graph. Hand-drawn boxes of slightly different sizes and
edges a few pixels apart look messy and are harder to read. Snapping makes
the obvious intent — "same size as that one", "lined up with that one",
"a circle" — exact, without asking the user to aim.

## The concept

**Snapping is a pipeline of small, pure functions.** Each step takes a point
and returns a (maybe) moved point:

```
raw end ──grid──▶ ──size──▶ ──align──▶ ──round──▶ snapped end
```

Three ideas make it work:

1. **Tolerances live in screen space.** "Within 6 pixels" must feel the
   same at zoom 0.1 and zoom 10, so the tolerance is converted to world
   units with the camera at the moment of the event:
   `camera.world_len(ALIGN_TOLERANCE_PX)`.
2. **Order is a decision.** Steps can disagree. Running round last means a
   nearly-square box is always *exactly* square; running grid first means
   smart snaps can refine a grid-snapped point. The order is written down in
   an ADR, not left to whoever touches the code next.
3. **Explain the result, not the steps.** Guides are computed from the
   *final* shape: for each line of it (edges and centre of a box, or the two
   endpoints of a line) that lies on a target, draw one guide. If round
   snapping moved an edge off an alignment, no stale guide remains.

Example: a 50 × 30 box exists at the origin. You drag a new box from
`(200, 3)` to `(253, 28)`:

| Step | Start | End |
|------|-------|-----|
| raw | (200, 3) | (253, 28) |
| align start (y = 0 is 3 away) | (200, 0) | |
| size (53 → 50, 28 → 30) | | (250, 30) |
| align (already on y = 30) | | (250, 30) |
| round (50 vs 30: not round) | | (250, 30) |

Guides: horizontal lines at y = 0, 15 and 30 — the new box's top, centre and
bottom all match the neighbour.

## How draw implements it

- `snap::Targets::from_shapes` turns shapes into *anchors* (a line value plus
  the extent of the target along it, for drawing the guide) and *sizes*.
- `snap::snap_to_grid`, `snap_size`, `align_point`/`align_end` and
  `snap_round` are the steps; each returns its input when its result would
  not be finite, which keeps the pipeline total (AC-6 is a proptest).
- `snap::snap_point` (start) and `snap::snap_drag` (end) chain the steps;
  `snap::Snaps::new` turns the helper flags and modifiers into which steps
  run: `Alt` disables all, `Shift` keeps only the grid so its square/45°
  constraint can be applied exactly afterwards.
- `shape_tool::on_pointer` snaps the start on `Down` and the end on every
  `Move`/`Up`, keeping the guides in the drag; `shape_tool::preview` hands
  them to the overlay.
- `render::dot_grid_spacing` doubles the grid step until dots are at least
  `DOT_GRID_MIN_PX` apart, and `render::dot_grid_points` enumerates the
  visible multiples (capped), so zooming far out never floods the frame.

## Try it

1. Change `ROUND_TOLERANCE` to `0.3` and run
   `cargo test snap::tests::far_from_round_is_unchanged` — why does it fail?
2. Swap the align and round steps in `snap_drag`, then run
   `cargo test snap::tests` — which test notices the different order?
3. Add a test that a box drag's *centre* aligns with a line endpoint.

## Further reading

- [[ADR-T17-1 Snapping order and tolerances]]
- [[03 Cameras - world space vs screen space]] — `world_len`/`screen_len`
- [[13 Property-based testing and fuzzing]] — why `snap_is_finite_for_any_input`
  uses `f32::MAX`
