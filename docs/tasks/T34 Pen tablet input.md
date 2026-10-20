---
id: T34
title: Pen tablet input
status: ready
wave: 22
branch: task/T34-pen-tablet
depends_on: [T28]
adrs: []
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

To be refined in the spec step.

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

- [ ] spec — `docs(T34): specify pen tablet acceptance criteria`
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
