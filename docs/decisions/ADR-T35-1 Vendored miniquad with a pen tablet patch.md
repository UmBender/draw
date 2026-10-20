---
id: ADR-T35-1
title: Vendored miniquad with a pen tablet patch
status: accepted
kind: decision
date: 2026-10-20
task: T35
builds_on: ["[[ADR-T28-1 Opt-in native Wayland backend]]", "[[ADR-T34-1 Tap slop for pen tablets detected at start-up]]", "[[ADR-T12-1 Blocking event loop with cached frame]]"]
amends: ["[[ADR-0014 Minimal dependencies and no unsafe]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T35-1 Vendored miniquad with a pen tablet patch

## Context

The owner wants pen pressure and a pen that draws on the native Wayland
backend ([[T35 Pen pressure and native Wayland tablets]]). miniquad 0.4.11,
which macroquad uses for the window, offers neither:

- **X11.** It selects only `XI_RawMotion` on the root window and reads the
  first two raw values as a mouse delta. The pen's pressure valuator
  (labelled `Abs Pressure`, axis 2 on the Xwayland tablet device) arrives in
  the same raw events but is never read.
- **Wayland.** It never binds `zwp_tablet_manager_v2`, so Hyprland sends
  the pen's motion as pointer motion and drops tip and barrel presses.

`/dev/input/event*` is not readable by the user, so reading evdev
ourselves is not an option. [[ADR-0014 Minimal dependencies and no unsafe]]
allows only crates.io macroquad (plus fontdue, ADR-T32-1) and forbids
`unsafe`. Tablet input needs FFI, and only miniquad does that.

macroquad's `EventHandler` has no method for pressure, and macroquad
replays only the events it knows to input subscribers.

## Decision

1. **Vendor miniquad 0.4.11** unchanged into `vendor/miniquad/` (its
   licences included) and point Cargo at it with
   `[patch.crates-io] miniquad = { path = "vendor/miniquad" }`. macroquad
   stays the crates.io release. `vendor/miniquad/PATCHED.md` lists every
   change against upstream. ADR-0014 is amended for this one crate:
   `unsafe` stays forbidden in draw's own crate, and the vendored crate is
   FFI code like upstream.
2. **Side channel for pressure.** The patch adds `miniquad::pen` with
   `PenSample { x, y, pressure }` (physical pixels, pressure in `[0, 1]`)
   and `take_samples()`. Just before a backend hands a pen's pointer
   motion, press or release to the event handler, it pushes a sample at
   the same position. macroquad is not patched.
3. **X11.** On each `XI_RawMotion` the patch looks up the source device
   (`XIQueryDevice`, cached per device id) for a valuator labelled
   `Abs Pressure`. If the device has one, it keeps the normalised pressure
   `(v − min) / (max − min)`; any other device clears it. Core pointer
   motion, presses and releases then push a sample while a pressure is
   kept.
4. **Wayland.** The patch binds `zwp_tablet_manager_v2` (version 1) when
   the compositor offers it and takes the tablet seat of the `wl_seat`. On
   each tool `frame` it sends motion as pointer motion, tip `down`/`up` as
   the left button, `BTN_STYLUS` (0x14b) as the middle button and
   `BTN_STYLUS2` (0x14c) as the right button, each with a sample of the
   latest pressure (`value / 65535`). On `proximity_in` the tool gets the
   pointer's cursor. Pads and their groups, rings and strips are bound so
   their events can be read, then ignored.
5. **draw pairs samples with events.** Each frame the shell drains the
   samples into the input `Collector`. A pointer event takes the pressure
   of the oldest sample at its physical position (within 0.01 px), and
   older samples are dropped. An event with no matching sample has no
   pressure. The collector emits `InputEvent::Pressure` before a pointer
   event whenever the pressure changes, including to `None`.

## Alternatives considered

- **Another window library** (winit + glow, SDL) — rewrites the shell and
  renderer, and every backend quirk already handled (ADR-T28-1, ADR-T34-1)
  would come back.
- **Also patch macroquad** to pass a pressure event through — a second
  vendored crate, about four times larger. A position-matched side channel
  is enough because macroquad replays every event in order.
- **One pressure value per frame** — simpler, but strokes at 175 Hz would
  get stepped widths.
- **Select XI2 device events (`XI_Motion`) on the window** — XI2 events then
  replace the core events miniquad relies on for every device.

## Consequences

- `cargo update` no longer moves miniquad. An upgrade means re-applying
  `PATCHED.md` to the new release, under a new ADR.
- On Wayland the second barrel button (lost in Xwayland) now works as
  right = eraser, and the pen draws natively.
- On X11 a pressure-only change (pen still) produces no sample until the
  pointer moves. At 175 Hz this is not visible.
- The vendored crate is not a workspace member: `cargo fmt --all` and
  `cargo clippy` skip it. Cargo shows a path dependency's compiler
  warnings, so the patch also fixes miniquad's two upstream warnings.

## Rollback plan

Remove `[patch.crates-io]` and `vendor/miniquad/`, and make
`Collector::set_pen_samples` unused (strokes then carry no widths). Record
it as a rollback ADR reverting this one.
