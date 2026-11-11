---
id: T110
title: Minimap
status: in-progress
wave: 59
branch: task/T110-minimap
depends_on: []
adrs: ["[[ADR-T110-1 Minimap geometry, routing and overview cache]]"]
feature: "[[Infinite canvas, pan and zoom]]"
tutorial: ""
tags: [task]
---

# T110 Minimap

## Goal

While panning — `Space` held, or the middle button dragging — a small overview of the whole drawing shows in a corner, with a frame for the current view that follows the pan, so you can see where you are moving to. Clicking in it while `Space` is held jumps the view there.

## Known facts (2026-11-10)

- `Space` + drag and middle-button drag pan the view; `F` fits all content in the window ([[Infinite canvas, pan and zoom]]).
- The proposal had an `M` toggle; the owner chose to show the minimap only while panning, with no toggle.
- The document layer is cached and re-rendered only on change (ADR-T24-5); idle CPU is 0 % ([[ADR-0006 Redraw on demand]]).
- The proposal's risk: permanent UI and work on every edit, so the overview must cost nothing while hidden.

## Spec

Owner decisions are marked (owner); the rest is settled in
[[ADR-T110-1 Minimap geometry, routing and overview cache]]. The geometry
is `core::minimap` (`Fit`, `Minimap`), the routing a `Gesture::Minimap`
in `core::editor`, the cached overview and the drawing `shell::minimap`.

- **AC-1** (owner) — The minimap shows in the bottom-right corner while `Space` is held and while a middle-button pan is in progress, and hides as soon as both end. It shows every shape scaled to fit, and a frame for the visible area that moves with the pan. In text edit mode `Space` types a space and shows nothing.
  Tests (`core::minimap::tests`): `panel_rect_sits_in_the_bottom_right_corner`,
  `panel_rect_small_viewport_is_none`, `new_world_is_content_union_view`,
  `content_rect_lies_inside_the_drawing_area`, `frame_follows_the_view`,
  `frame_tiny_view_keeps_a_minimum_size`;
  (`core::editor::tests`): `minimap_shows_while_space_is_held`,
  `minimap_shows_during_a_middle_pan`, `minimap_hidden_for_a_hand_tool_pan`,
  `minimap_space_in_text_edit_shows_nothing`,
  `minimap_show_and_hide_ask_for_a_redraw`,
  `minimap_space_repeat_asks_for_no_redraw`,
  `minimap_frame_moves_with_the_pan`. The look is a manual check.
- **AC-2** (owner) — While `Space` is held, a click in the minimap centres the view on that point and dragging there moves the view continuously; those clicks never reach the canvas tools. Outside the minimap, `Space` + drag pans as before.
  Tests (`core::minimap::tests`): `world_at_inverts_point`,
  `world_at_outside_the_drawing_area_clamps`;
  (`core::editor::tests`): `minimap_click_centres_the_view`,
  `minimap_drag_moves_the_view_continuously`,
  `minimap_drag_keeps_the_world_box_of_its_press`,
  `minimap_click_never_reaches_the_tool`,
  `minimap_press_without_space_goes_to_the_tool`,
  `space_drag_outside_the_minimap_pans`,
  `minimap_drag_ends_after_space_is_released`,
  `minimap_drag_survives_a_snapshot_round_trip`,
  `minimap_drag_is_cancelled_by_undo`.
- **AC-3** — While hidden it costs nothing. While shown, it is drawn from its own cache, re-rendered only when the document changed since the last time it was shown; the view frame is an overlay, so panning does not re-render it.
  Tests (`core::editor::tests`): `minimap_hidden_computes_no_content_bounds`,
  `minimap_content_bounds_follow_the_document_revision`;
  (`core::minimap::tests`): `place_same_world_is_the_drawing_area`,
  `place_shrinks_when_the_view_leaves_the_content`,
  `place_degenerate_image_is_none`;
  (`shell::minimap::tests`): `overview_key_ignores_the_camera`,
  `overview_key_changes_with_the_document`,
  `overview_key_changes_with_the_target_size`,
  `needs_render_only_for_a_new_key`, `marker_only_for_small_content`.
  Idle CPU and `DRAW_FRAME_TIMES` are a manual check.
- **AC-4** — An empty document shows an empty minimap; huge or far-apart content keeps every coordinate finite (fuzzed).
  Tests (`core::minimap::tests`): `content_bounds_empty_document_is_none`,
  `new_empty_document_shows_the_view_only`,
  `fit_zero_extent_world_is_finite`, `fit_huge_world_is_finite`,
  proptest `minimap_coordinates_stay_finite`;
  (`core::editor::tests`): `minimap_empty_document_has_no_content`;
  fuzz (`tests/fuzz.rs`): the minimap part of `check_invariants`,
  `arb_minimap_drag` in `arb_session`, `sessions_reach_the_minimap`.

## Out of scope

- A key that keeps the minimap on; choosing the corner or size; showing it in PNG export.

## Files owned

- a new `src/core/minimap.rs` (geometry), `src/core/mod.rs` (its `pub mod` line), `src/core/editor.rs`, `src/core/input.rs`
- `src/shell/` (overview cache, input routing, drawing)
- `docs/features/Infinite canvas, pan and zoom.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T110-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T110): specify the minimap`
- [ ] tests — `test(T110): add failing tests for the minimap`
- [ ] models — `feat(T110): add the minimap geometry`
- [ ] behaviour — `feat(T110): show the minimap while panning`
- [ ] quality — `chore(T110): pass clippy and rustfmt`
- [ ] docs — `docs(T110): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from V2 of the [[Feature proposal]].
- 2026-11-10 — owner: show it while `Space` is held (panning) instead of an `M` toggle.
- 2026-11-10 — spec: test names per AC; [[ADR-T110-1 Minimap geometry, routing and overview cache]] settles size, world box, routing, snapshot record and cache.
- 2026-11-10 — UNATTENDED CHOICE: `src/core/mod.rs` added to *Files owned* for the `pub mod minimap;` line.
- 2026-11-10 — UNATTENDED CHOICE: the new `gesture minimap` snapshot record is additive, so `snapshot::VERSION` is not bumped (as ADR-T91-1 did).
- 2026-11-10 — UNATTENDED CHOICE: a hand-tool drag does not show the minimap (the owner listed `Space` and the middle button only).
