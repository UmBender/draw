---
id: T26
title: Pan and zoom without re-rendering
status: in-progress
wave: 14
branch: task/T26-gesture-redraw
depends_on: [T24]
adrs: ["[[ADR-T12-1 Blocking event loop with cached frame]]", "[[ADR-T24-1 Document layer keyed by a document revision]]", "[[ADR-T24-5 Redraw budgets]]", "[[ADR-T26-1 Reuse the document layer during pan and zoom]]", "[[ADR-T26-2 Gesture frame budget]]"]
feature:
tutorial: "[[26 Reusing a frame during a gesture]]"
tags: [task]
---

# T26 Pan and zoom without re-rendering

## Goal

Panning and zooming a 5 000-stroke scene is smooth at any zoom level.
Today every pan or zoom event changes the camera, which is part of the
document layer's key
([[ADR-T24-1 Document layer keyed by a document revision]]), so every
event re-renders the whole layer. That takes ≈ 15 ms zoomed out and
40–60 ms at medium zoom on the target, over the 16 ms budget of
[[ADR-T24-5 Redraw budgets]].

Found during [[T24 Smooth redraws with large documents]] (2026-10-03),
from the owner's idea of deferring MSAA until a gesture ends. MSAA is
mostly GPU time and the remaining cost is CPU (vertices and uploads), so
this task skips the whole re-render during a gesture instead.

## Spec

How it works: [[ADR-T26-1 Reuse the document layer during pan and zoom]];
budget: [[ADR-T26-2 Gesture frame budget]]. Unit tests live in
`#[cfg(test)] mod tests` of `shell::app`.

- **AC-1 — Reuse during a gesture.** While only the camera changed since
  the cached document layer, `plan_redraw` returns `Redraw::Moved` and the
  frame draws that layer moved and scaled (one textured quad, linear
  filter) to `blit_rect(cached, current, layer)`; the overlay is drawn as
  today. The frame timer names these frames `moved`.
  *Tests:* `plan_redraw_camera_only_is_moved`,
  `blit_rect_same_camera_is_layer`, `blit_rect_pan_moves_by_screen_delta`,
  `blit_rect_zoom_scales_about_anchor`,
  `blit_rect_maps_corners_through_cameras` (proptest),
  `blit_filter_moved_is_linear`, `blit_filter_at_rest_is_nearest`,
  `frame_log_line_names_moved`.
- **AC-2 — Settle.** Once the camera has been still for `SETTLE`
  (100 ms), `settle` turns `Moved` into `Full`: the layer is re-rendered
  at the current camera with MSAA. While a frame is `Moved` it calls
  `miniquad::window::schedule_update` from the main thread, so the
  blocking loop runs again instead of sleeping (a timer thread cannot
  wake the X11 loop, see the ADR). *Tests:*
  `settle_moved_before_quiet_period_stays_moved`,
  `settle_moved_after_quiet_period_is_full`,
  `settle_other_redraws_are_unchanged`, `settle_period_is_100_ms`.
  Wake-up: manual check on the target.
- **AC-3 — Margin.** The layer covers the window plus `layer_margin`
  physical pixels on each side (⅛ of the longer side, no layer side over
  `MAX_LAYER_SIDE` = 8192), rendered through `layer_camera` (the view
  camera shifted by the margin); `render::draw_underlay` takes the camera
  so the dots fill the margin too. At rest the blit is 1:1 at −margin.
  Size and memory are in the ADR. *Tests:*
  `layer_margin_is_eighth_of_longer_side`,
  `layer_margin_keeps_layer_within_max_side`,
  `layer_size_adds_margin_on_both_sides`,
  `layer_camera_shows_window_origin_at_margin`.
- **AC-4 — Any other change wins.** A document, hidden-set, size or
  underlay change, alone or together with a camera change, re-renders as
  today. *Tests:* `plan_redraw_changed_key_is_full` (the camera-only case
  moves to AC-1), `plan_redraw_camera_and_other_change_is_full`,
  `plan_redraw_append_with_other_change_is_full` and the other existing
  `plan_redraw_*` tests unchanged.
- **AC-5 — Budget.** With the ADR-T24-5 scene, `moved` frames cost < 2 ms
  CPU at any zoom; the settle re-render is listed separately. *Owner on
  the target* with `DRAW_FRAME_TIMES=1`; numbers in the *Log*, then
  ADR-T26-2 becomes `accepted`.
- **AC-6 — No regressions.** Output after settling looks the same as
  today (1:1 nearest blit of a layer rendered with the same zoom);
  `scripts/check.sh` and the T14 perf tests (`tests/perf.rs`) stay green.

## Out of scope

- Cutting the cost of the settle re-render itself (geometry,
  [[T25 Linear-time selection with large documents]]).
- Retained GPU buffers or anything needing `unsafe`.

## Files owned

- `src/shell/app.rs`
- `src/shell/render.rs` (blit helpers, and `draw_underlay` taking the
  camera — agreed with the owner in the spec step)
- new ADRs `ADR-T26-*`, tutorial `docs/tutorials/26 Reusing a frame during a gesture.md`
- `docs/architecture/Rendering performance options.md`,
  `docs/architecture/Architecture.md` (record what was done)

## Subtasks (one commit each)

- [x] spec — `docs(T26): specify gesture redraw acceptance criteria`
- [ ] tests — `test(T26): …`
- [ ] models — `feat(T26): …`
- [ ] behaviour — `feat(T26): …`
- [ ] quality — `chore(T26): …`
- [ ] docs — `docs(T26): …`

## Learning path

Step 26 — requires step 24 (layered redraws) and step 12 (app loop).

## Log

- 2026-10-03 — Created from T24's AC-4 result (medium zoom 40–60 ms);
  not started. Best measured after T25 lands, so the settle numbers do
  not include the selection lookups.
- 2026-10-03 — Spec: read miniquad 0.4.11's X11 loop. `schedule_update`
  from another thread is not seen while the loop sleeps in `XNextEvent`
  and races the loop's `try_lock().unwrap()`, so the settle wake-up is
  main-thread polling while `Moved` (ADR-T26-1). The margin needs
  `render::draw_underlay` to take a camera; the owner agreed to widen
  *Files owned* for that one signature.
