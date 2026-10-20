---
id: T34
title: Pen tablet input
status: in-progress
wave: 22
branch: task/T34-pen-tablet
depends_on: [T28]
adrs: ["[[ADR-T34-1 Tap slop for pen tablets detected at start-up]]"]
feature: "[[Pen tablets]]"
tutorial: "[[34 Telling a tap from a drag]]"
tags: [task]
---

# T34 Pen tablet input

## Goal

Drawing with a pen tablet (the owner's HUION 420) works as well as with a
mouse. A pen tap is a clean click on every tool, even though the tip slips
a few pixels while it touches down and lifts. draw finds out by itself that a
tablet is attached and says so; `DRAW_TABLET` forces it either way. Pressure
and the native Wayland backend are left to
[[T35 Pen pressure and native Wayland tablets]].

## Findings (2026-10-20, owner's HUION 420 on Hyprland 0.56.2)

Measured with a bare miniquad 0.4.11 window that logged every pointer event:

- **X11 / Xwayland (default backend).** The pen is a pointer: tip →
  left button, hover → motion, ≈ 175 motion events per second, integer
  positions. One barrel button arrives as the middle button (pan); the
  other sends nothing, although the kernel reports both (`BTN_STYLUS`,
  `BTN_STYLUS2`). It is lost in the compositor or Xwayland.
- **Taps slip.** Between press and release a tap moved 10–40 px, so a
  Select tap becomes a marquee, a Shape tap a tiny drag. Today's thresholds
  are 2–3 px (`CLICK_SLOP_PX`, `MIN_DRAG_PX`). Part of the slip is the
  compositor mapping the 105 × 59 mm tablet across both monitors
  (≈ 36 px/mm across, 18 px/mm down); that is fixed in the owner's dotfiles,
  not here. Mapped onto one 1920 × 1080 monitor it is still ≈ 18 px/mm, so a
  1 mm slip is ≈ 18 px.
- **Native Wayland (`DRAW_BACKEND=wayland`).** miniquad does not bind
  `zwp_tablet_manager_v2`; Hyprland sends the pen's motion as pointer motion
  but no tip or button presses, so the pen cannot draw. → T35.
- **Detection.** `/proc/bus/input/devices` is world-readable (the
  `/dev/input/event*` nodes are not): the pen's block has `BTN_TOOL_PEN`
  (key 0x140) and `ABS_X`, `ABS_Y`, `ABS_PRESSURE`; no mouse, touchpad or
  keyboard on the machine has `BTN_TOOL_PEN`.

## Owner decisions (2026-10-20)

- Ergonomics now (1.2.0), pressure later (T35).
- Tap slop turned on automatically when a tablet is attached;
  `DRAW_TABLET=on|off` forces it; the toolbar shows that pen mode is on.
- Barrel buttons keep the mouse mapping (middle = pan, right = eraser).

## Spec

Decision: [[ADR-T34-1 Tap slop for pen tablets detected at start-up]].

- **AC-1 — Tap filter.** `core::tap::TapFilter::new(slop)` is off (every
  event passes unchanged) for a slop that is zero, negative or not finite.
  On: after a press, moves within the slop are held back; a release within
  the slop arrives at the press point; the first move outside the slop
  releases the last held move, then itself, and later events pass until the
  release; a press or release of another button during the hold starts the
  drag the same way; keys, wheel, characters, resizes and non-finite
  pointer events pass without changing the state; presses and releases
  stay balanced and in order.
  Tests (`src/core/tap.rs`): `tap_filter_off_passes_every_event_unchanged`
  (proptest), `tap_filter_tap_within_slop_is_a_click_at_the_press_point`,
  `tap_filter_moves_within_slop_are_held_back`,
  `tap_filter_leaving_the_slop_flushes_the_last_held_move`,
  `tap_filter_drag_passes_through_to_the_release`,
  `tap_filter_other_button_during_hold_starts_the_drag`,
  `tap_filter_keys_and_non_finite_events_pass_without_changing_state`,
  `tap_filter_keeps_presses_and_releases_in_order` (proptest).
- **AC-2 — Editor.** `Editor::new()` has no slop (`tap_slop() == 0`) and
  behaves as before; `with_tap_slop(PEN_TAP_SLOP_PX)` routes every event
  through the filter: a slipping tap selects with Select, makes no shape
  with Rect, commits a dot with Pen, and a stroke that leaves the slop is
  drawn from the press point to the release point.
  Tests (`src/core/editor.rs`): `tap_slop_is_off_by_default`,
  `pen_tap_with_select_selects_instead_of_marquee`,
  `pen_tap_with_rect_makes_no_shape`,
  `pen_tap_with_pen_commits_a_dot`,
  `pen_stroke_beyond_slop_runs_from_press_to_release`.
- **AC-3 — Detection.** `shell::tablet::find_pen(devices)` returns the
  name of the first device with `BTN_TOOL_PEN` and `ABS_X` + `ABS_Y`;
  `choose(value, devices)` follows ADR-T34-1 §3 and returns whether pen
  mode is on, its notice and any warning.
  Tests (`src/shell/tablet.rs`): `find_pen_finds_the_huion_420_pen`,
  `find_pen_ignores_keyboards_mice_and_touchpads`,
  `find_pen_never_panics` (proptest),
  `choose_auto_follows_detection`,
  `choose_on_and_off_force_pen_mode`,
  `choose_ignores_case_and_surrounding_space`,
  `choose_unknown_value_warns_and_detects`,
  `choose_on_gives_the_pen_tap_slop`.
- **AC-4 — Shell.** `shell::app::run` builds the editor with the slop
  from `tablet::from_env()` and prints its notice and warning once; the
  toolbar draws a `pen` badge at the bottom of the strip while the slop is
  on. Tests (`src/shell/toolbar.rs`):
  `pen_badge_sits_in_the_strip_below_the_buttons`; the wiring is checked
  on the target (AC-7).
- **AC-5 — Fuzz.** Sessions on an editor with the pen slop keep every
  invariant, and undo/redo stays symmetric.
  Tests (`tests/fuzz.rs`): `pen_tap_sessions_never_panic_and_keep_invariants`,
  `pen_tap_undo_all_then_redo_all_is_symmetric`.
- **AC-6 — Docs.** README *Pen tablets* section: what works, the tap
  slop, `DRAW_TABLET`, mapping the tablet to one monitor (Hyprland
  `input:tablet:output`, X11 `xinput map-to-output`), the native Wayland
  limitation and T35.
- **AC-7 — Owner check (manual).** On the target, with the HUION 420:
  start-up notice printed, `pen` badge shown, taps select / place text /
  make no Rect, strokes draw normally; `DRAW_TABLET=off` restores the old
  behaviour.

## Out of scope

- Pressure, tilt and the eraser end of the pen (T35).
- Pen input on the native Wayland backend (T35).
- Tablet mapping (area, monitor, aspect): compositor configuration.
- Pad (express key) buttons: configured in the compositor or driver.

## Files owned

- `src/core/tap.rs` (new), `src/core/mod.rs`, `src/core/editor.rs`
- `src/shell/tablet.rs` (new), `src/shell/mod.rs`, `src/shell/app.rs`,
  `src/shell/toolbar.rs`
- `tests/fuzz.rs`, `README.md`
- new ADRs `ADR-T34-*`, feature note `docs/features/Pen tablets.md`,
  tutorial `docs/tutorials/34 Telling a tap from a drag.md`

## Subtasks (one commit each)

- [x] spec — `docs(T34): specify pen tablet acceptance criteria`
- [ ] tests — `test(T34): add failing tests for tap slop and tablet detection`
- [ ] models — `feat(T34): add tap filter and tablet choice types`
- [ ] behaviour — `feat(T34): hold pen taps still and detect tablets`
- [ ] quality — `chore(T34): pass clippy, rustfmt and rustdoc`
- [ ] docs — `docs(T34): add feature note and tutorial`

## Learning path

Step 34 — requires step 8 (editor as an input state machine) and step 28
(choosing a backend at start-up).

## Log

- 2026-10-20 — Created from the owner's request for graphics tablet
  support in 1.2.0, after logging the owner's HUION 420 on both backends
  (findings above).
