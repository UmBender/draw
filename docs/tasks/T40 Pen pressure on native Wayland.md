---
id: T40
title: Pen pressure on native Wayland
status: in-progress
wave: 26
branch: fix/wayland-pen-pressure
depends_on: [T35]
adrs: ["[[ADR-T40-1 Tablet stream wins over copied wl_pointer input]]"]
feature: "[[Pen tablets]]"
tutorial: "[[35 Reading a pen through a patched window library]]"
tags: [task]
---

# T40 Pen pressure on native Wayland

## Goal

With `DRAW_BACKEND=wayland`, pen strokes get thicker the harder the pen
presses, as they already do on X11. Today (1.4.1) every pen stroke on the
native Wayland backend has a constant width, although the compositor
sends pressure.

## Known facts (2026-10-22, owner's HUION 420 on Hyprland, Flatpak 1.4.1)

- X11 backend: pressure works.
- Native Wayland backend: the pen draws, but every stroke has a constant
  width. `WAYLAND_DEBUG=client` shows `zwp_tablet_tool_v2.pressure`
  events with sensible values (0 to about 60000 of 65535), so the
  compositor side is fine.
- Hyprland sends **both** `wl_pointer.motion` and
  `zwp_tablet_tool_v2.motion` for the pen at identical coordinates. Hover
  motion is copied with a real timestamp; right before every tip-down
  there is a synthetic copy with time `0`. The log had the same sequence
  before all 7 downs:

  ```
  zwp_tablet_tool_v2.pressure(10539)
  wl_pointer.motion(0, 226.60546875, 121.92968750)
  zwp_tablet_tool_v2.motion(226.60546875, 121.92968750)
  zwp_tablet_tool_v2.down(96358)
  zwp_tablet_tool_v2.frame(...)
  ```

  No `wl_pointer.button` events came for the tip. During the stroke there
  were no more `wl_pointer.motion` copies.
- **Root cause.** The tablet frame pushes two `miniquad::pen` samples at
  the same position (one for the motion, one for the tip-down), but three
  pointer events reach `shell::input_map::Collector`: the `wl_pointer`
  copy, the pen motion and the pen down. `Collector::pair_pen` gives
  each event the *oldest* sample within `PEN_MATCH_PX`, so the
  `wl_pointer` copy takes the motion's sample, the pen motion takes the
  down's, and the `PointerDown` gets none (`InputEvent::Pressure { None }`).
- A stroke whose first kept point has no width factor never gets widths:
  `Smoother::push_width` only carries widths once the first point had one
  (`prev_width.zip(...)`, `src/core/smoothing.rs:184`). So pressure from
  later points is dropped and the whole stroke is drawn with
  `widths: None`.
- `pointer_handle_motion` / `pointer_handle_button`
  (`vendor/miniquad/src/native/linux_wayland.rs:800`, `:817`) know nothing
  about the tablet. The tablet tool state already tracks proximity in
  `PenTool::focused` (set on `proximity_in`, cleared on `proximity_out`).

## Proposal (settle in the spec step)

1. In the vendored miniquad Wayland backend, drop `wl_pointer` motion and
   button events while any tablet tool is in proximity of the window
   (`focused`). The tool frames already deliver the pen's motion and
   buttons, so nothing is lost, and a mouse works again once the pen
   leaves proximity. Mark it `draw patch (ADR-T35-1)` and list it in
   `vendor/miniquad/PATCHED.md`.
2. Needs an ADR (or an amendment ADR to ADR-T35-1) recording that the
   compositor may copy tablet input onto `wl_pointer` and that the
   backend prefers the tablet stream.
3. Open: where the regression test lives. The vendored crate is not a
   workspace member and its callbacks are FFI. Options: a pure helper in
   `vendor/miniquad/src/pen.rs` (e.g. "should this pointer event be
   dropped") tested from draw, or a `Collector` test that replays the
   logged sequence (wl copy, pen motion, pen down with two samples) and
   asserts the down gets pressure once the copy is filtered.
4. Open: also make the Pen tool robust when the down has no pressure but
   later points do (start widths at factor 1 for the leading points)?
   This hides any future pairing slip, but changes AC-5 of T35 ("without
   pressure, `widths` is `None`").

## Spec

Decision: [[ADR-T40-1 Tablet stream wins over copied wl_pointer input]].
Proposal 1 is taken; the regression test is a pure helper in
`vendor/miniquad/src/pen.rs` tested from draw plus a `Collector` replay
of the logged sequence (proposal 3); proposal 4 is not taken (T35 AC-5
stays).

`miniquad::pen::pointer_shadowed_by_pen(in_proximity)` takes whether each
known tablet tool is in proximity of the window and returns whether a
`wl_pointer` motion or button event is dropped. The Wayland backend calls
it in `pointer_handle_motion` and `pointer_handle_button` with
`PenTool::focused` of every tool.

- **AC-1** — a `wl_pointer` event is dropped while at least one tool is
  in proximity: `pointer_shadowed_by_pen_with_a_tool_in_proximity_is_true`.
- **AC-2** — it is kept with no tools, or with no tool in proximity, so
  the mouse works after the pen leaves:
  `pointer_shadowed_by_pen_without_tools_in_proximity_is_false`.
- **AC-3** — replaying the logged Hyprland tip-down (copied
  `wl_pointer.motion`, tool motion, tool down; two samples at that
  position) through the filter, the `PointerDown` is paired with the
  down's pressure and no `Pressure { None }` is emitted:
  `collector_replayed_wayland_down_with_pen_in_proximity_gets_pressure`.
- **AC-4** — after proximity out the same `wl_pointer.motion` passes the
  filter and carries no pressure:
  `collector_replayed_wayland_motion_after_proximity_out_has_no_pressure`.
- **AC-5** — owner check on the HUION 420 (see *Log*): light and hard
  presses give thin and thick strokes on `DRAW_BACKEND=wayland`, the mouse
  works after the pen leaves, barrel buttons still pan and erase.

## Out of scope

- X11 (works).
- Tilt, rotation, eraser end, pad buttons.
- Other compositors' quirks beyond the copied `wl_pointer` stream.

## Files owned

- `vendor/miniquad/src/native/linux_wayland.rs`, `vendor/miniquad/src/pen.rs`,
  `vendor/miniquad/PATCHED.md`
- `src/shell/input_map.rs` (tests), possibly `src/core/tools/pen.rs` and
  `src/core/smoothing.rs` (proposal 4)
- `README.md` (Pen tablets), `docs/features/Pen tablets.md`
- new ADR `ADR-T40-*`

## Subtasks (one commit each)

- [x] spec — `docs(T40): specify pen pressure on native Wayland`
- [ ] tests — `test(T40): ...`
- [ ] behaviour — `fix(T40): ...`
- [ ] quality — `chore(T40): pass clippy and rustfmt`
- [ ] docs — `docs(T40): ...`

## Learning path

No new tutorial step; extend step 35 (reading a pen through a patched
window library) with the copied `wl_pointer` stream.

## Log

- 2026-10-22 — Created from the owner's report: no pressure on the
  native Wayland backend in the 1.4.1 Flatpak (the Flatpak override set
  `DRAW_BACKEND=wayland`; the owner unset it to use X11 meanwhile).
  Diagnosed from a `WAYLAND_DEBUG=client` log of one session.
- Owner check after the fix: on `DRAW_BACKEND=wayland` with the HUION 420,
  light and hard presses give thin and thick strokes; the mouse still
  works after the pen leaves proximity; barrel buttons still pan and
  erase.
