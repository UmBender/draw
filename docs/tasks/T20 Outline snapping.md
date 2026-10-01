---
id: T20
title: Outline snapping
status: ready
wave: 9
branch: task/T20-outline-snapping
depends_on: [T17]
adrs: ["[[ADR-T17-1 Snapping order and tolerances]]"]
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

To be refined at the spec step. Tolerances are screen pixels (as in T17).
Starting point:

- **AC-1** — Nearest outline point: pure functions give the point on a
  shape's outline nearest to `p`. `Rect`: nearest of the four sides.
  `Ellipse`: exact for circles (`c + r·(p − c)/|p − c|`), Newton iteration
  for other ellipses (also for `p` inside and `p` at the centre). `Grid`:
  nearest of its lines (`shape::grid_lines`). *Tests:*
  `snap::tests::outline_rect_*`, `outline_circle_*`, `outline_ellipse_*`,
  `outline_grid_*`, plus a proptest that the result lies on the outline
  (within ε) and is no farther than any sampled outline point.
- **AC-2** — No overlap: the snapped point is offset from the outline,
  away from the target and towards the side `p` came from, by half the
  target's outline width plus half the dragged shape's width (from
  `ToolView::style.width_px`, converted with the camera). The two strokes
  touch without overlapping. *Tests:* `snap::tests::outline_offset_outside`,
  `outline_offset_inside`.
- **AC-3** — Priority: smart snap only. For the start point and the end of
  line, arrow, rectangle and ellipse drags, outline snap runs **only when
  the T17 alignment matched nothing** on either axis, and only within
  `OUTLINE_TOLERANCE_PX`. When it fires, the box is not then made round
  (outline wins over round, alignment wins over outline). Grid and size
  snaps run before it, as in T17. *Tests:*
  `snap::tests::alignment_beats_outline`, `outline_beats_round`,
  `outline_snaps_line_end_to_circle`, `outline_snaps_box_corner`.
- **AC-4** — Indicator: an outline snap adds a small cross (two guide
  segments of `OUTLINE_MARK_PX` on screen) at the snapped point to
  `Snapped::guides`. No renderer change. *Test:*
  `snap::tests::outline_snap_marks_point`.
- **AC-5** — `Alt` and `Shift` overrides stay as in ADR-T17-1 (`Alt`: no
  snap; `Shift`: grid only). *Test:* `snap::tests::outline_respects_overrides`.
- **AC-6** — Total and finite for any finite input, including degenerate
  (zero-size) targets. *Test:* `snap::tests::snap_is_finite_for_any_input`
  (extended with ellipses and widths).

ADR to write at the spec step: `ADR-T20-1 Outline snapping`, amending
[[ADR-T17-1 Snapping order and tolerances]] (pipeline order and targets).

## Out of scope

Outlines of lines, arrows and freehand strokes as targets; snapping while
moving selections; rotated shapes (none exist).

## Files owned

`src/core/snap.rs`, `docs/features/Snapping.md` (new section only), new
`ADR-T20-*`. Runs in parallel with [[T19 Auto-numbering]], which owns
`src/core/tools/shape_tool.rs`: if T20 turns out to need that file, stop and
coordinate rather than editing it.

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 20 — requires step 17.

## Log
