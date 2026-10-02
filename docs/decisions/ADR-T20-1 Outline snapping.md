---
id: ADR-T20-1
title: Outline snapping
status: accepted
kind: decision
date: 2026-10-01
task: T20
builds_on: [ADR-T17-1, ADR-0013]
amends: [ADR-T17-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T20-1 Outline snapping

## Context

[[T20 Outline snapping]] lets a dragged point land anywhere on the outline
of a nearby rectangle, ellipse or grid, so arrows stop at graph nodes and
boxes touch. [[ADR-T17-1 Snapping order and tolerances]] fixed the snap
pipeline and its targets (edges, centres, sizes). A new step has to fit in
that order without fighting alignment or round snapping, and the result
must look right: two strokes that "touch" must not overlap by their
widths ([[ADR-0013 World-space widths and zoom limits]]). `shape_tool.rs`
belongs to a parallel task, so the change must stay behind
`snap_start`/`snap_end`.

## Decision

- **Targets.** `Targets` gains `outlines`: one `Outline` (rect, ellipse
  or grid box with its stroke width) per `Rect`, `Ellipse` and `Grid`.
  Lines, arrows and strokes have no outline target.
- **Pipeline** (amends ADR-T17-1). Start point: grid → align → outline.
  Moving end: grid → size → align → outline → round (size and round only
  for boxes). Outline runs only if alignment matched no anchor on either
  axis; if outline fires, round is skipped. Priority is therefore
  alignment > outline > round. All of it is smart snap: `Alt` and `Shift`
  overrides are unchanged.
- **Geometry.** Nearest point: rect = nearest of four sides; grid =
  nearest of `shape::grid_lines`; circle = exact radial projection;
  other ellipses = Eberly's bracketed root search (bisection, `f64`),
  chosen over Newton because it always converges, also for points inside,
  on an axis or at the centre. Flat ellipses are segments.
- **No overlap.** The point moves to `q + n·(w_target/2 + w_drag/2)`,
  `n` pointing from the outline towards `p` (the outward normal if `p` is
  exactly on it). The dragged width comes from `ToolView::style.width_px`
  through the camera. Snap fires when `|p − q| ≤ tolerance + offset`
  (`OUTLINE_TOLERANCE_PX` = 8), so thick strokes do not shrink the pull.
- **Indicator.** An outline snap of the moving end adds a small ×
  (`OUTLINE_MARK_PX` = 8 on screen) at the point to `Snapped::guides`;
  the renderer already draws guides. The start point has no mark, since
  `snap_start` returns a bare point.

## Alternatives considered

- **Outline before alignment** — alignment guides are the precise,
  explicit intent; an outline pull would steal a deliberate alignment.
- **Snap to the outline itself (no offset)** — thick strokes visibly
  overlap; with the offset they touch edge to edge.
- **Newton iteration for ellipses** — fast but can diverge or stall for
  points inside or near the centre; bisection on a bracketed root is a
  few dozen cheap steps and always lands.
- **Outline constrained to the aligned axis when only one axis aligned**
  — nicer for a perfectly horizontal arrow into a circle, but needs
  line/outline intersections for every target type; left for later.

## Consequences

- An arrow aimed at a node's centre line (within 6 px) aligns instead of
  stopping at the outline; aim off the centre line or hold the point a
  bit away. A follow-up may combine one-axis alignment with outlines.
- Cost stays O(shapes) per event; each ellipse costs one bounded
  bisection (≤ 160 iterations, usually far fewer).
- Snapping while moving selections and outlines of lines, arrows and
  strokes remain out of scope.

## Rollback plan

Revert T20's commits: `Targets::outlines`, `Outline` and `snap_outline`
go away and `snap_point`/`snap_drag` return to ADR-T17-1's pipeline. Add a
rollback ADR reverting this one.
