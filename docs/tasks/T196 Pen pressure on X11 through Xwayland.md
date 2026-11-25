---
id: T196
title: Pen pressure on X11 through Xwayland
status: in-progress
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

- Root cause reproduced locally under Xvfb (2026-11-24): `XiApi::open`
  calls `XIQueryVersion` with `(2, 0)`. In XInput 2.0 the X server stops
  delivering `XI_RawMotion` while any pointer grab is active — including
  the implicit passive grab held from `ButtonPress` (tip down) until
  `ButtonRelease` (tip up). While the pen hovers before touching down,
  `XI_RawMotion` arrives with `Abs Pressure = 0`, setting `PenTracker`
  to `Some((device, 0.0))`; during the entire stroke no `XI_RawMotion`
  arrives, so every `MotionNotify` emits `RawEvent::Tablet { pressure: 0.0 }`
  (`width_factor(0.0) = 0.25`, a constant thin line). Mouse hover clears
  `PenTracker` to `None`, so mouse strokes draw at `1.0` (thick). Asking
  for XInput 2.1 (`XI_VERSION = (2, 1)`) restores `XI_RawMotion` during
  the implicit grab (verified under Xvfb with `xdotool mousedown 1` +
  `mousemove`).
- Secondary defects found in the same path: `pressure_axis` interns
  `"Abs Pressure"` with `only_if_exists = 1` (fails if queried before
  Xwayland creates the atom), and `fetch_generic` passes `head.sourceid`
  without guarding against `0` (`XIAllDevices`).

## Spec

Design: negotiate XInput 2.1 (`x11::pen::XI_VERSION = (2, 1)`) in
`XiApi::open` so `XI_RawMotion` keeps arriving during the implicit grab
of a button press; intern `"Abs Pressure"` with `only_if_exists = 0`;
resolve the event source via pure `x11::pen::raw_source(device_id, source_id)`
so `0` (`XIAllDevices`) never reaches a single-device `XIQueryDevice`;
extract device valuator probing into pure `x11::pen::DeviceProbe`,
`probe_valuators` and `probe_log_line`. `X11Platform` probes all devices
at start-up (and any unseen raw source during `poll`), queues one debug
line per device with valuators via `take_debug_log`, and `Closing` in
`src/shell/gpu_native.rs` forwards those lines to
`diag::log(Level::Debug, ...)`. `X11Platform::raw_motion_count` counts
received `XI_RawMotion` events for the Xvfb grab regression test. No new
ADR: this fixes the XI2 version negotiation and diagnostics inside the
existing T176 design.

- **AC-1** — Regression:
  - Pure (`crates/draw-sys/tests/x11_input_pure.rs`):
    `regression_t196_xwayland_pen_pressure_follows_the_valuator` probes
    an Xwayland-like `xwayland-tablet stylus` device (axis 2 =
    `Abs Pressure`, range `0..65535`), feeds raw motion (slave sourceid,
    varying pressure and missing-valuator frames) through `raw_source`
    and `PenTracker`, checks `RawEvent::Tablet` follows the valuator,
    checks `XI_VERSION >= (2, 1)`, and checks `raw_source` falls back
    from `0` to `device_id`.
  - Integration (`crates/draw-sys/tests/x11_input.rs`):
    `regression_t196_raw_motion_arrives_during_button_grab` holds button
    1 down under Xvfb and moves the pointer, confirming
    `X11Platform::raw_motion_count` advances during the implicit grab.
- **AC-2** — Diagnostics log (`--log debug`):
  - Pure (`crates/draw-sys/tests/x11_input_pure.rs`):
    `pen_probe_log_line_names_pen_axis_and_range`,
    `pen_probe_log_line_notes_valuators_without_pressure`,
    `pen_probe_log_line_skips_devices_without_valuators`.
  - Integration (`crates/draw-sys/tests/x11_input.rs`):
    `x11_startup_queues_debug_lines_for_valuator_devices` checks
    `X11Platform::take_debug_log` emits debug lines for Xvfb's valuator
    devices.
- Manual check (owner): `draw --backend x11 --log debug` on KDE Plasma
  Wayland with the Huion 420 — the log shows the pen's pressure axis,
  and light and hard strokes differ in width.

## Out of scope

- Native Wayland pressure (works); Windows (T191).

## Files owned

- `crates/draw-sys/src/x11/pen.rs`, `crates/draw-sys/src/x11/xlib.rs`,
  `crates/draw-sys/src/x11/platform.rs`
- `crates/draw-sys/tests/x11_input_pure.rs`, `crates/draw-sys/tests/x11_input.rs`
- `src/shell/gpu_native.rs` (logging call site for AC-2, allowed by batch plan)
- `docs/features/Pen tablets.md`, this note

## Subtasks (one commit each)

- [x] spec — `docs(T196): specify X11 pen pressure under Xwayland`
- [ ] regression — `test(T196): add regression test for constant X11 pen pressure`
- [ ] behaviour — `fix(T196): read the pen pressure under Xwayland`
- [ ] quality — `chore(T196): pass clippy and rustfmt`
- [ ] docs — `docs(T196): update the notes`

## Log

- 2026-11-24 — written from the owner's screenshot; cause not found yet.
- 2026-11-24 — root cause reproduced under Xvfb: `XiApi::open` asked
  `XIQueryVersion` for `(2, 0)`, which suppresses `XI_RawMotion` during
  the implicit pointer grab of a button press (tip down to tip up),
  pinning `PenTracker` at the hover pressure `0.0` (0.25× width) for the
  whole stroke. UNATTENDED CHOICE: forward `X11Platform::take_debug_log`
  to `diag::log(Level::Debug, ...)` in `src/shell/gpu_native.rs`
  (`Closing::new` and `Closing::poll`) as the logging call site permitted
  by `.batch/plan.md`, and use lowercase `regression_t196_...` test names
  to satisfy Rust's `non_snake_case` lint.
