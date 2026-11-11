---
id: ADR-T109-1
title: Shape recognition on Z
status: accepted
kind: decision
date: 2026-11-10
task: T109
builds_on: [ADR-0005, ADR-T11-1, ADR-T29-2, ADR-T74-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T109-1 Shape recognition on Z

## Context

A shaky pen circle or square is common when sketching fast; switching to
the ellipse or rectangle tool breaks the flow. The owner chose bare `Z`
(left hand; `Ctrl+Z` stays undo, `Q` is the laser pointer) to replace the
last pen stroke by the straight line, rectangle or ellipse it looks like,
and nothing when it looks like none of them: a wrong guess costs more than
redrawing. Draw has no rotated shapes.

## Decision

- **Key.** Bare `Z` → `Command::RecognizeShape` ("Last stroke to line /
  rectangle / ellipse"), exact modifiers (ADR-T11-1). In text edit mode
  keys never reach the keymap (ADR-T29-2), so `Z` types `z` there.
- **"Last pen stroke, nothing else since."** Read from the history, not a
  new editor field: the newest undo entry must be a transaction of exactly
  one `Insert` of a `Shape::Stroke`, the redo stack must be empty, the
  stroke must still be in the document and no gesture may be in progress.
  So a new stroke moves the target to that stroke; any other edit, a
  recognition, or an undo (redo stack not empty) disables `Z`. The state
  needs no new snapshot field (ADR-T74-1: a field outside the snapshot
  would break the "snapshot, load, continue equal" property), so the
  snapshot format is unchanged.
  UNATTENDED CHOICE: two consequences follow and are accepted: a redo
  that brings the stroke back makes it the last action again, and a
  paste or duplicate of exactly one stroke counts as a stroke (both put
  a single stroke insert on top of the history). View changes, tool,
  colour and selection changes are not edits and keep `Z` available.
- **Result.** One `tx_replace` of the stroke by the shape, same id and
  stacking position: one undo step that gives the stroke back. No fit →
  no transaction, no undo step, the command returns `false`. The new
  shape keeps the stroke's colour and width (`style`); per-point pressure
  factors are dropped. Rectangles and ellipses are unfilled, unlabelled
  (numbering is for new nodes drawn with the tools) and have no text.
- **Fitter** (`core::recognize`, pure, f64 inside). The stroke is
  resampled to 128 points evenly spaced along its length (pen strokes are
  simplified, so raw vertices are uneven). With `L` the path length:
  1. *Degenerate*: fewer than two points, a non-finite point or `L = 0` →
     no fit.
  2. *Line*: chord `c = |last − first|` with `c ≥ 0.8 L` and every sample
     within `0.05 c` of the chord → `Line` from the first to the last
     point of the stroke.
  3. *Closed*: `|last − first| ≤ 0.15 L`; otherwise no fit (an open curve
     is never closed into a shape).
  4. *Frame*: the minimum-area bounding box of the samples over 90
     rotations (1° steps, ties to the smallest angle), so rotated shapes
     are measured in their own frame. Its short side must be at least
     `0.1` of its long side. `D` is its diagonal.
  5. The samples must wind around the box centre at least `0.8` of a full
     turn (measured on the box normalised to a square).
  6. *Corners*: a box corner counts when a sample lies within `0.08 D`.
  7. *Rect*: 4 corners, mean distance of the samples to the box outline
     `≤ 0.035 D` and largest `≤ 0.1 D`.
     *Ellipse*: 0 corners, the same limits for the radial distance to the
     inscribed ellipse.
     Anything else (1–3 corners, larger residuals) → no fit.
  8. The rectangle or ellipse fills the **axis-aligned** bounding box of
     the stroke's points (AC-4: draw has no rotation), built from the
     original `f32` points, so the result is always finite.
- **Thresholds** are `pub const`s in `core::recognize` and are tested on
  deterministic generated samples (shaky lines, squares, rectangles,
  circles, ellipses, rotated ones) and on scribbles that must stay
  strokes (zigzag, spiral, triangle, figure eight, open arcs, an `L`).

## Alternatives considered

- **An editor field `recognizable: Option<ShapeId>`** — exact "nothing
  else since", including after redo, but it must be saved in snapshots
  (format bump, fixtures) or the snapshot-continue property fails.
- **Corner detection by turning angle** — sensitive to anti-tremor
  smoothing and sample spacing; the box-corner test and the two residuals
  are simpler and scale with the stroke.
- **Least-squares conic fit** — handles tilted ellipses directly, but the
  result must be axis-aligned anyway and the min-area box already gives
  the frame; more numerics for no visible gain.
- **Recognise while drawing** — out of scope (task note).

## Consequences

- Scribbles, triangles and open curves stay strokes; ambiguous rounded
  squares fall to the ellipse or to nothing, never to a wrong line.
- A rotated rectangle becomes the axis-aligned rectangle around it, which
  is larger; that is the owner's AC-4.
- The fitter costs `O(90 · 128)` per key press, independent of the
  stroke's length beyond the resampling.

## Rollback plan

Remove the binding, `Command::RecognizeShape`, `core::recognize` and the
editor's `recognize_last_stroke`, in a new ADR with `git revert` of the
T109 commits.
