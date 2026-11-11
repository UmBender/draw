---
id: T106
title: Fading laser pointer
status: done
wave: 59
branch: task/T106-laser-pointer
depends_on: []
adrs: [ADR-T106-1, ADR-T106-2]
feature: "[[Infinite canvas, pan and zoom]]"
tutorial: ""
tags: [task]
---

# T106 Fading laser pointer

## Goal

Holding `Q` with the left hand turns the pen or mouse in the right hand into a laser pointer: a bright trail follows it and fades away, and nothing is drawn, selected or added to the history.

## Known facts (2026-11-10)

- `Q` is unbound as a bare key (only listed among the keys `keymap.rs` knows). The proposal had `L`; the owner chose `Q` so the left hand holds it while the right hand draws.
- draw redraws only on input ([[ADR-0006 Redraw on demand]]): a fading trail needs frames after the last event, until it has faded, then idle CPU must go back to 0 %.
- Overlays (selection outline, previews) are drawn over the cached document layer without re-rendering it (ADR-T24-5): the trail belongs there.

## Spec

Owner decisions are marked (owner); the rest is settled in
[[ADR-T106-1 Laser pointer in front of the editor]] and
[[ADR-T106-2 Laser key, fade and look]]. The laser is a
`core::laser::Laser` that the shell puts in front of the toolbar and the
editor; times are seconds passed in by the shell.

- **AC-1** (owner) — While `Q` is held, pointer moves (pen or mouse, button down or not) add points to a trail instead of reaching the current tool. Releasing `Q` stops adding points; the tool, selection and any open text edit are as before. In text edit mode `Q` types a `q`.
  Tests (`core::laser::tests`): `route_q_down_holds_and_is_consumed`,
  `route_move_while_held_adds_a_point`, `route_move_without_q_is_forwarded`,
  `route_q_up_stops_adding_points`, `route_presses_while_held_are_swallowed`,
  `route_q_repeat_keeps_one_piece`, `route_ctrl_or_alt_q_is_forwarded`,
  `route_other_events_while_held_are_forwarded`,
  `route_q_in_text_edit_is_forwarded`,
  `laser_in_front_of_editor_q_types_q_in_text_edit`,
  `laser_in_front_of_editor_keeps_tool_and_selection`;
  `core::keymap::tests::q_is_not_bound`.
- **AC-2** — Each trail point fades out linearly over `FADE_SECS` = 1 s; the trail is drawn as a 6 px (`WIDTH_PX`) stroke in `theme::RED`, in screen space, the same width at every zoom.
  Tests: `alpha_fades_linearly_over_the_fade_time`,
  `segments_join_points_of_one_hold`, `segments_restart_after_release`,
  `segment_takes_the_older_points_alpha`,
  `tick_drops_points_older_than_the_fade`, `trail_is_bounded`,
  `trail_stays_in_screen_space_when_zooming`,
  `route_non_finite_position_or_time_adds_no_point`. The drawing itself
  is a manual check.
- **AC-3** — The trail never changes the document, the selection or the history: no undo step, no snapshot change, no autosave trigger.
  Tests: `laser_in_front_of_editor_changes_no_state` (snapshot text,
  document revision and undo depth unchanged); fuzz
  `laser_sessions_keep_invariants_and_undo_symmetry` and
  `laser_only_chunks_leave_the_snapshot_unchanged` in `tests/fuzz.rs`.
- **AC-4** — Frames are requested only while a trail point is still visible; once it has faded the app goes back to waiting for events (idle 0 %). Only the overlay is redrawn, never the document layer.
  Tests: `tick_redraws_while_points_are_alive`,
  `tick_after_the_fade_redraws_once_then_idles`,
  `empty_trail_never_asks_for_frames`,
  `tick_non_finite_time_clears_the_trail`. The shell part (an `Overlay`
  redraw, `schedule_update` only while `is_fading`) is a manual check
  with `DRAW_FRAME_TIMES=1` and `top`.
- **AC-5** — Holding `Q` during a gesture (a stroke or drag in progress) ends nothing: the gesture keeps going and the trail starts with the next move after it. Fuzzed with `Q` presses and releases mixed into input sessions.
  Tests: `route_during_a_gesture_forwards_pointer_events`,
  `route_first_move_after_a_gesture_starts_a_new_piece`,
  `laser_in_front_of_editor_q_mid_stroke_finishes_the_stroke`; fuzz
  `laser_sessions_keep_invariants_and_undo_symmetry`.

## Out of scope

- Laser colour or width settings; a laser that stays on without holding a key.
- Recording or replaying the trail; showing it in PNG export.

## Files owned

- `src/core/keymap.rs`, `src/core/editor.rs`, `src/core/input.rs`, a new `src/core/laser.rs`
- `src/shell/` (key release, frame requests, overlay drawing)
- `docs/features/Infinite canvas, pan and zoom.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`
- `src/core/mod.rs` (one `pub mod laser;` line)
- this note, and new `ADR-T106-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T106): specify the fading laser pointer`
- [x] tests — `test(T106): add failing tests for the laser pointer`
- [x] models — `feat(T106): add the laser trail model`
- [x] behaviour — `feat(T106): draw the fading trail while Q is held`
- [x] quality — `refactor(T106): pass clippy and rustfmt`
- [x] docs — `docs(T106): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from V3 of the [[Feature proposal]]; key changed from `L` to `Q` for the left hand.
- 2026-11-10 — spec: the laser is `core::laser`, routed by the shell in front of the editor (ADR-T106-1); 1 s linear fade, 6 px, `theme::RED`, 1024 points (ADR-T106-2). UNATTENDED CHOICE: routing outside the editor instead of a held flag in it, because an editor flag is saved state and would need a snapshot version bump (every fixture, e2e golden and autosave) for a transient key.
- 2026-11-10 — UNATTENDED CHOICE: added `src/core/mod.rs` to *Files owned* to declare the new `laser` module.
- 2026-11-10 — tests: `tests/fuzz.rs` gains `arb_laser_session` (ordinary sessions mixed with `Q` held around gestures, pressed mid-gesture, and lone `Q` presses/releases with any modifiers), a coverage check and two properties; `check_undo_redo_symmetric` was split so laser sessions reuse its undo/redo half (no check weakened). No snapshot or regression fixture changed: the snapshot format stays at version 1.
- 2026-11-10 — behaviour: the shell sets `had_input` (autosave) only for events the laser forwards, so laser use never wakes the autosave. The routing, clock and frame requests live in a `LaserLayer` in `shell/app.rs`. `editor.rs` and `input.rs` were not touched: `Key::Q` and `KeyUp` already reached the core, the same path as `Space`.
- 2026-11-10 — quality: `scripts/check.sh --fuzz` green; `scripts/e2e.sh` green (all cases, no golden changed). No e2e case for the laser: the harness's `hold` directive cannot move the pointer while a key is down, and `tests/e2e.rs` is not owned.
- 2026-11-10 — manual checks left for the owner: hold `Q` and wave the pen and the mouse; the trail fades in about 1 s and CPU returns to idle (`top`); release `Q` mid-stroke and press `Q` mid-stroke (the stroke finishes, the trail starts after it); `q` in an open text block; redraw budget (ADR-T24-5) with `DRAW_FRAME_TIMES=1` while waving: frames should log `overlay`, never `full`.
