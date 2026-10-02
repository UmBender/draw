---
id: T20
title: Outline snapping
status: done
wave: 9
branch: task/T20-outline-snapping
depends_on: [T17]
adrs: ["[[ADR-T17-1 Snapping order and tolerances]]", "[[ADR-T20-1 Outline snapping]]"]
feature: "[[Snapping]]"
tutorial: "[[20 Snapping to outlines]]"
tags: [task]
---

# T20 Outline snapping

## Goal

Dragged points can land anywhere on the border of a nearby shape, not just
on its edges, centre and middles: an arrow drawn to a graph node stops right
at the circle's outline, and a box corner can sit on the edge of another box
or circle. The outlines touch without overlapping. The T17 alignment snaps
keep the highest priority.

## Spec

Tolerances are screen pixels converted with the camera (as in T17):
`OUTLINE_TOLERANCE_PX` = 8, `OUTLINE_MARK_PX` = 8. Design and trade-offs:
[[ADR-T20-1 Outline snapping]].

- **AC-1** — Nearest outline point: `Outline::nearest(p)` gives the point
  of a target's outline nearest to `p`, plus a unit normal used when `p`
  lies exactly on it. `Targets::from_shapes` collects one `Outline` per
  `Rect`, `Ellipse` and `Grid` (with its stroke width).
  - `Rect`: nearest of the four sides.
  - `Ellipse`: exact for circles (`c + r·(p − c)/|p − c|`, `p = c` gives
    the rightmost point); other ellipses use a bracketed root search
    (Eberly's distance-to-ellipse, bisection in `f64`) instead of Newton,
    because it cannot diverge for `p` inside, on an axis or at the centre.
    A flat ellipse (one zero semi-axis) is its major-axis segment.
  - `Grid`: nearest of its lines (`shape::grid_lines`).

  *Tests:* `snap::tests::outline_rect_nearest_side`,
  `outline_rect_inside_point`, `outline_circle_exact`,
  `outline_circle_centre_is_on_circle`, `outline_ellipse_axis_points`,
  `outline_ellipse_inside_and_centre`, `outline_ellipse_flat_is_segment`,
  `outline_grid_inner_line`, `targets_collect_outlines`, and proptests
  `outline_ellipse_point_lies_on_ellipse` (`(x/a)² + (y/b)² ≈ 1`) and
  `outline_ellipse_is_nearest_sampled_point` (no farther than any of 360
  sampled outline points, within ε).
- **AC-2** — No overlap: `snap_outline` moves `p` to `q + n·offset` where
  `q` is the nearest outline point, `n = (p − q)/|p − q|` (the outline
  normal when `p = q`, pointing out of the shape) and `offset` = half the
  target's stroke width + half the dragged shape's width
  (`camera.world_len(view.style.width_px)`). `p` outside stays outside,
  `p` inside stays inside, the strokes touch without overlapping. It fires
  when `|p − q| ≤ tolerance + offset`; the nearest outline wins.
  *Tests:* `snap::tests::outline_offset_outside`, `outline_offset_inside`,
  `outline_on_line_goes_outward`, `outline_out_of_tolerance_is_none`.
- **AC-3** — Priority (smart snap only). Start point: grid → align →
  outline. Moving end of line, arrow, rectangle and ellipse drags:
  grid → size → align → outline → round. Outline runs **only when the
  alignment matched nothing** on either axis (an anchor within tolerance
  counts as a match even if the point did not move); when outline fires,
  round is skipped. *Tests:* `snap::tests::alignment_beats_outline`,
  `outline_beats_round`, `outline_snaps_line_end_to_circle`,
  `outline_snaps_box_corner`, `outline_snaps_start_point`.
- **AC-4** — Indicator: an outline snap of the moving end adds a small ×
  (two diagonal guide segments spanning `OUTLINE_MARK_PX` on screen per
  axis, centred on the snapped point) to `Snapped::guides`. No renderer
  change; the start point gets no mark (`snap_start` returns a bare point
  and `shape_tool.rs` is not touched). *Test:*
  `snap::tests::outline_snap_marks_point`.
- **AC-5** — `Alt` and `Shift` overrides stay as in ADR-T17-1 (`Alt`: no
  snap; `Shift`: grid only), through `snap_start`/`snap_end`. *Test:*
  `snap::tests::outline_respects_overrides`.
- **AC-6** — Total and finite for any finite input, including degenerate
  (zero-size) targets and any widths; a step whose result is not finite
  is skipped. *Tests:* `snap::tests::snap_is_finite_for_any_input`
  (extended with ellipses and widths), `outline_is_finite_for_any_input`.

## Out of scope

Outlines of lines, arrows and freehand strokes as targets; snapping while
moving selections; rotated shapes (none exist).

## Files owned

`src/core/snap.rs`, `docs/features/Snapping.md` (new section only), new
`ADR-T20-*`. Runs in parallel with [[T19 Auto-numbering]], which owns
`src/core/tools/shape_tool.rs`: if T20 turns out to need that file, stop and
coordinate rather than editing it.

## Subtasks (one commit each)

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs

## Learning path

Step 20 — requires step 17.

## Log

- 2026-10-01 spec: AC-1…AC-6 name their tests; ADR-T20-1 amends
  ADR-T17-1. Ellipses use Eberly's bisection instead of Newton (always
  converges, also inside / at the centre). Tolerance measured as
  `|p − q| ≤ tolerance + offset`.
- tests: red commit (missing API). `snap_point`/`snap_drag` gained a
  `width` argument, so T17 tests were updated to the new signatures.
- models: `Outline`, `OutlineKind`, `Nearest`, `Targets::outlines`,
  `snap_outline`, `OUTLINE_TOLERANCE_PX` = `OUTLINE_MARK_PX` = 8.
- behaviour: green. Fixed one wrong test assertion
  (`outline_snaps_start_point` assumed grid snap keeps (190, 190); it
  rounds to (200, 200)), noted in the commit.
- quality: clippy (`float_cmp`, single-char names, negated compare) and
  a redundant rustdoc link. `scripts/check.sh` green; long fuzz
  (`PROPTEST_CASES=20000`) of `snap` green.
- `shape_tool.rs` untouched: `snap_start`/`snap_end` derive the dragged
  width from `ToolView::style.width_px`. The start point gets no ×.
- Observed trade-off of the specified priority: near a rectangle side
  or a circle's centre line, alignment matches first, so the point lands
  on the line (strokes overlap by half widths) instead of beside it.
  Recorded in the feature note; a follow-up could let outline refine a
  one-axis alignment.

