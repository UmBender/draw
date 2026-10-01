---
id: ADR-T17-1
title: Snapping order and tolerances
status: accepted
kind: decision
date: 2026-10-01
task: T17
builds_on: [ADR-T16-3, ADR-T16-1, ADR-0013]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T17-1 Snapping order and tolerances

## Context

[[T17 Snapping]] fills the `snap::snap_end` stub left by
[[ADR-T16-3 Helper settings and hooks]]. Four snaps (grid, matching sizes,
alignment, near-round) can each move the same point; they can disagree, and
they interact with the `Shift` constraint. Tolerances must feel the same at
every zoom ([[ADR-0013 World-space widths and zoom limits]]).

## Decision

- **Pipeline.** The moving end of a shape drag goes
  grid → size → align → round; the start point (on `Down`) goes
  grid → align. Grid snap is gated by `Helpers::grid_snap`, the others by
  `Helpers::smart_snap`. Size and round apply only to box drags
  (`Rect`/`Ellipse`); `Line`/`Arrow` get grid and align. Later steps win:
  round runs last, so an almost-square box is always exactly square, with
  the larger side (which a size snap may already have matched).
- **Tolerances** are screen pixels converted with the camera at the event:
  `SIZE_TOLERANCE_PX` = `ALIGN_TOLERANCE_PX` = 6. `ROUND_TOLERANCE` = 0.1 is
  a ratio. Sizes match integer multiples up to `MAX_SIZE_MULTIPLE` = 8.
- **Grid**: a fixed world step `GRID_STEP` = 20 units, independent of the
  T18 grid shapes. The renderer draws the dots on multiples of the step,
  doubling the spacing until dots are at least `DOT_GRID_MIN_PX` apart, so
  zooming out never floods the frame.
- **Targets** are all document shapes (the tool has no viewport): box edges
  and centres of `Rect`/`Ellipse`/`Grid`, endpoints of `Line`/`Arrow`; sizes
  from box widths/heights and grid cells. Freehand strokes are ignored —
  their boxes are noisy. Cost is O(shapes) per pointer event.
- **Guides** are derived from the final shape, not from the snap steps: one
  guide per x/y line of the result (box edges + centre, or endpoints) that
  lies on a target. A guide never lies about the result, even after round
  snapping moved an aligned edge.
- **Overrides** are sampled per event like `Shift`: `Alt` disables every
  snap; `Shift` keeps only grid snap and then applies its constraint, so the
  constraint is exact and no guides are shown.
- **Totality**: each step that would produce a non-finite value is skipped.

## Alternatives considered

- **Snap steps decide the guides** — guides would go stale when a later
  step moves the point.
- **Zoom-dependent grid step** — snapping positions would change with zoom;
  shapes drawn at different zooms would not line up.
- **Only on-screen shapes as targets** — needs the viewport in `ToolView`
  (every fixture) for a small gain at scratch-pad document sizes.
- **Alt latched at `Down`** — releasing `Alt` mid-drag could not bring
  snapping back; per-event matches `Shift`.

## Consequences

- Shapes drawn with helpers off are unchanged (all helpers default off).
- Very large documents pay a linear scan per move while snapping is on.
- Snapping while moving selections or for freehand strokes stays out of
  scope; adding it reuses the `snap` functions.

## Rollback plan

Revert T17's commits (`snap_end` back to identity, shape tool without snap
calls, empty render hooks) and add a rollback ADR; ADR-T16-3's stubs remain.
