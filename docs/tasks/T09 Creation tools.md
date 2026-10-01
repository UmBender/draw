---
id: T09
title: Creation tools
status: review
wave: 5
branch: task/T09-creation-tools
depends_on: [T04, T08]
adrs: ["[[ADR-0007 Anti-tremor pipeline]]", "[[ADR-0013 World-space widths and zoom limits]]"]
feature: "[[Drawing tools]]"
tutorial: "[[09 Building drawing tools]]"
tags: [task]
---

# T09 Creation tools

## Goal

Pen (smoothed), line, arrow, rectangle and ellipse tools.

## Spec

Tools follow the API fixed by [[ADR-T08-1 Tool context and gesture overlay]]:
`on_pointer` / `preview` / `cancel`, gesture state in each module's `State`.
Positions arrive in screen pixels and are converted with `ctx.camera` at the
moment of each event.

- **AC-1** — Pen: `Down` starts a `Smoother` with `ctx.smoothing.params()` and
  `px_to_world = camera.world_len(1.0)` and pushes the world position; `Move`
  pushes; `preview()` returns one `Shape::Stroke` with the live smoothed points
  (empty overlay when idle); `Up` pushes, finishes and commits one `Stroke` with
  `ctx.commit` (one transaction). A click without movement commits a one-point
  stroke (a dot). `Move`/`Up` without a `Down` do nothing. Colour is
  `ctx.style.color`.
  *Tests:* `pen::tests::pen_drag_commits_one_stroke`,
  `pen_preview_shows_live_stroke`, `pen_preview_is_empty_when_idle`,
  `pen_click_without_movement_commits_dot`,
  `pen_points_are_in_world_coordinates`, `pen_uses_style_color`,
  `pen_move_without_down_does_nothing`, `pen_up_without_down_commits_nothing`.
- **AC-2** — Stroke width = `style.width_px / zoom`, sampled on `Down`
  (both tools). *Tests:* `pen::tests::pen_width_is_zoom_independent_on_screen`,
  `shape_tool::tests::shape_width_is_zoom_independent_on_screen`.
- **AC-3** — Shape tool (`Line`, `Arrow`, `Rect`, `Ellipse`, chosen by
  `ctx.tool` on `Down`): `a` = world position of `Down`, `b` = world position
  of the latest event; preview while dragging; commit on `Up` (`Rect`/`Ellipse`
  unfilled, `Arrow` head at `b`). A drag whose screen length
  (`camera.screen_len(|b − a|)`, unconstrained) is shorter than
  `MIN_DRAG_PX = 2.0` commits nothing. A non-shape `ctx.tool` is ignored.
  *Tests:* `shape_tool::tests::shape_drag_commits_line`,
  `shape_drag_commits_arrow`, `shape_drag_commits_rect`,
  `shape_drag_commits_ellipse`, `shape_preview_while_dragging`,
  `shape_drag_shorter_than_two_px_commits_nothing`,
  `shape_drag_of_two_px_commits`, `shape_tool_ignores_non_shape_tool`,
  and the property `shape_gesture_never_panics_and_is_finite`.
- **AC-4** — `Shift` (taken from the latest pointer event) constrains: rect →
  square, ellipse → circle (side = `max(|dx|, |dy|)`, keeping the drag's
  signs), line/arrow → projection of the drag onto the nearest multiple of 45°.
  *Tests:* `shift_constrains_rect_to_square`,
  `shift_constrains_ellipse_to_circle`,
  `shift_constrains_line_to_nearest_45_degrees`,
  `shift_constrains_arrow_to_diagonal`, property
  `shift_constrained_line_angle_is_multiple_of_45_degrees`.
- **AC-5** — `cancel()` (Esc / tool switch) discards the gesture with no
  document change; returns whether a gesture was discarded; a following `Up`
  commits nothing. *Tests:* `pen::tests::cancel_discards_gesture`,
  `shape_tool::tests::cancel_discards_gesture`,
  `cancel_when_idle_returns_false` (both modules).
- **AC-6** — Each completed gesture is exactly one undo step.
  *Tests:* `pen::tests::one_gesture_one_undo`,
  `shape_tool::tests::one_gesture_one_undo`.

## Out of scope

Text, pressure, curve fitting.

## Files owned

`src/core/tools/pen.rs`, `src/core/tools/shape_tool.rs`

## Subtasks (one commit each)

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs

## Learning path

Step 9 — requires steps 4 and 8.

## Log

- 2026-10-01 — spec: named the tests per AC; width sampled on `Down`, drag
  threshold measured on screen on the unconstrained drag, 45° snap by
  projection. No new ADR (behaviour stays inside the ADR-T08-1 tool API).
- tests: pen and shape tool unit tests plus two proptests
  (`shape_gesture_never_panics_and_is_finite`,
  `shift_constrained_line_angle_is_multiple_of_45_degrees`); red on the
  missing `MIN_DRAG_PX`.
- models: `pen::State`/`LiveStroke`, `shape_tool::State`/`Drag`/`Kind`,
  `MIN_DRAG_PX`.
- behaviour: all green. Deviation: a rejected (< 2 px) drag returns `true`
  from `on_pointer` because its preview must be erased; the threshold test
  was adjusted to assert the redraw and the unchanged document.
- quality: clippy (`expect_used` in test helpers, negated float comparison)
  and rustfmt fixes.
- docs: [[Drawing tools]] feature note and [[09 Building drawing tools]]
  tutorial (step 9).
