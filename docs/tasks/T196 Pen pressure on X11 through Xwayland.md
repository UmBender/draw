---
id: T196
title: Pen pressure on X11 through Xwayland
status: ready
wave: 78
branch: fix/x11-pen-pressure-xwayland
depends_on: []
adrs: []
feature: "[[Pen tablets]]"
tutorial: ""
tags: [task, bug, x11]
---

# T196 Pen pressure on X11 through Xwayland

## Goal

With `draw --backend x11` on a Wayland session (Xwayland), a pen stroke
gets thicker as the pen is pressed harder, as it does on native Wayland.

## Known facts (2026-11-24)

- Reported by the owner on `main` after T195 (screenshot): KDE Plasma 6.8
  Wayland, `--backend x11`, Huion 420 ("HUION 420 Pen" found at start-up).
  Mouse strokes are drawn thick; every pen stroke is a thin line of the
  same width whatever the pressure.
- The X11 path (T176): `Connection::select_raw_motion` asks for
  `XI_RawMotion` of the master devices on the root window;
  `pressure_axis` finds the valuator labelled `Abs Pressure`
  (`XInternAtom(..., only_if_exists = 1)`); `x11::pen::PressureTracker`
  normalises it and turns motion into `RawEvent::Tablet` only when the
  source device has that axis (`crates/draw-sys/src/x11/pen.rs`,
  `xlib.rs`, `platform.rs`).
- Not known yet which step fails under Xwayland. Suspects, to check in
  order: (1) Xwayland's tablet slave device ("xwayland-tablet stylus…")
  has no axis labelled `Abs Pressure`, or the label atom does not exist
  yet when interned with `only_if_exists`; (2) raw events of the tablet
  do not reach the root window, or carry the master as source;
  (3) the axis range (Xwayland uses 0..65535) is normalised wrongly so
  the value stays at the minimum. `xinput list` / `xinput test-xi2 --root`
  (package `xinput`, not installed on the owner's machine) show 1 and 2;
  a `--log debug` line with the axis found per device would show it in
  the app.
- The e2e suite cannot inject pen pressure on X11 today; Xvfb has no
  tablet. A unit test over the decoded event (device id, mask, values)
  can cover the fix.
- Native Wayland pressure works (T40); not re-checked by the owner today.

## Spec

Draft; the spec step names the final tests.

- **AC-1** — Regression: a raw motion from an Xwayland-like pen device
  (its axis labels and range, source = slave device) gives
  `RawEvent::Tablet` with a pressure that follows the valuator.
  Test: `regression_T196_xwayland_pen_pressure_follows_the_valuator`.
- **AC-2** — The diagnostics log names, per pen device found, the
  pressure axis and its range (`--log debug`).
- Manual check (owner): `draw --backend x11` on KDE Wayland, light and
  hard strokes differ in width.

## Out of scope

- Native Wayland pressure (works); Windows (T191).

## Files owned

- `crates/draw-sys/src/x11/pen.rs`, `crates/draw-sys/src/x11/xlib.rs`,
  `crates/draw-sys/src/x11/platform.rs`
- `crates/draw-sys/tests/x11_input_pure.rs`, `crates/draw-sys/tests/x11_input.rs`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T196): specify X11 pen pressure under Xwayland`
- [ ] regression — `test(T196): add regression test for constant X11 pen pressure`
- [ ] behaviour — `fix(T196): read the pen pressure under Xwayland`
- [ ] quality — `chore(T196): pass clippy and rustfmt`
- [ ] docs — `docs(T196): update the notes`

## Log

- 2026-11-24 — written from the owner's screenshot; cause not found yet.
