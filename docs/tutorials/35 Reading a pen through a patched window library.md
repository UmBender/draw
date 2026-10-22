---
title: Reading a pen through a patched window library
step: 35
requires: ["[[34 Telling a tap from a drag]]", "[[24 Layered redraws and measuring a frame]]"]
feature: "[[Pen tablets]]"
code: ["vendor/miniquad/src/pen.rs", "vendor/miniquad/src/native/linux_x11/xi_input.rs", "vendor/miniquad/src/native/linux_wayland.rs", "vendor/miniquad/src/native/linux_wayland/extensions/tablet.rs", "src/shell/input_map.rs", "src/core/pressure.rs", "src/core/smoothing.rs", "src/core/tools/pen.rs", "src/shell/render.rs"]
tags: [tutorial]
---

# Reading a pen through a patched window library

> **Learning path step 35.** Requires: step 34 (telling a tap from a drag),
> step 24 (layered redraws, the stroke strip of ADR-T24-4) · Next: —

## Why this matters here

A pen's pressure is what makes a stroke look hand-drawn: thin where the pen
barely touches, thick where it presses. draw's window library, miniquad,
reports a pen only as a mouse: x, y and buttons. On X11 the pressure was
already in the events it receives, but it was never read. On native Wayland
the pen could not even click, because miniquad never asked the compositor
for tablet events. Getting pressure meant changing the window library
itself, and getting it from there to the tools without changing macroquad,
which sits in between.

## The concept

**1. Vendoring a dependency.** Cargo's `[patch.crates-io]` replaces a crate
everywhere in the dependency graph with a local copy:

```toml
[patch.crates-io]
miniquad = { path = "vendor/miniquad" }
```

macroquad still says `miniquad = "0.4"` and gets our copy. The cost is
ownership: upgrades are manual, so every change is listed in
`vendor/miniquad/PATCHED.md` and marked in the source.

**2. A side channel matched by position.** macroquad calls
`mouse_motion_event(x, y)` on its handler and queues the event for draw;
nothing in that path can carry a pressure. So the patched backend pushes a
`PenSample { x, y, pressure }` into a global queue *right before* it sends a
pen's pointer event. draw drains the queue once per frame and pairs each
pointer event with the oldest sample at the same position:

```
backend:   sample(3,4,.2) motion(3,4)  sample(5,4,.6) motion(5,4)  motion(9,9)   (mouse)
draw:      Pressure(.2)   Move(3,4)    Pressure(.6)   Move(5,4)    Pressure(None) Move(9,9)
```

Nothing is matched by time or by count, so a dropped sample, or an event with
no sample, cannot shift every later pairing. The editor sees an ordinary
event stream with a `Pressure` event whenever the pressure changes.

**3. Width as a factor.** The stroke keeps its chosen width, and each point
stores a factor, `clamp(2p, 0.25, 2)`. A half press is exactly today's
width, mouse strokes have no factors at all, and `[`/`]` still mean the
same thing.

## How draw implements it

**X11** (`xi_input.rs`). miniquad already listens to `XI_RawMotion` on the
root window. A raw event names its source device and carries only the
valuators that *changed*, packed in the order of the set bits of a mask.
`PenTracker` asks the server once per device (`XIQueryDevice`) for a
valuator labelled `Abs Pressure`; if there is one, that device is a pen and
the value is scaled by the axis' `min`/`max`. A pressure missing from an
event has not changed, so the last one stays. The core `MotionNotify` that
follows the raw event pushes a sample with it.

**Wayland** (`extensions/tablet.rs`, `linux_wayland.rs`). The tablet
protocol announces each tool as an object. Its events (`motion`,
`pressure`, `down`, `button`, …) come in groups closed by `frame`.
The patch buffers a frame and replays it as motion, left button (tip),
middle (`BTN_STYLUS`) and right (`BTN_STYLUS2`), each with a sample. The
protocol tables are written by hand: libwayland must know the interface of
every object an event creates (`tool_added` creates a tool, `pad_added` a
pad, which creates groups, rings and strips), even the ones draw ignores.

**Two streams for one pen** (T40). The compositor may *also* send the pen
as plain `wl_pointer` events. Hyprland copies hover motion, and right
before every tip-down a `wl_pointer.motion` with time `0` at the pen's
position. That copy has no sample, but pairing is by position, so it took
the tool motion's sample, the tool motion took the down's, and the press
got none; a stroke whose first point has no factor never gets widths.
The backend now drops `wl_pointer` motion and buttons while any tool is
in proximity (`pen::pointer_shadowed_by_pen`): one device, one stream.
The `Collector` tests replay the logged sequence through that filter.

**Pairing** (`Collector::pair_pen` in `src/shell/input_map.rs`) runs in
physical pixels, before the conversion to logical pixels, so the
comparison is exact up to 0.01 px.

**Smoothing** (`Smoother::push_width`). Width factors ride along with the
points: resampling keeps or drops both together, the EMA smooths the
factor with the same α, and Ramer–Douglas–Peucker
(`simplify_rdp_widths`) also keeps a point whose factor differs by more
than 0.1 from what the chord would interpolate there. Without that, a
straight stroke pressed harder in the middle would collapse to its two
end widths.

**Rendering** (`Batch::polyline_widths`). The ADR-T24-4 strip already
offsets every point by a half-width along the mitre; now each point uses
its own. `Batch::polyline` calls it with a constant width, and a proptest
checks that the mesh for mouse strokes stays exactly the same.

## Try it

1. Change `MAX_WIDTH_FACTOR` in `src/core/pressure.rs` to `3.0`, run
   `cargo test pressure` and see which tests pin the curve.
2. Set `WIDTH_EPSILON` to `10.0` and draw a straight line, pressing harder
   in the middle: the bulge disappears, because RDP no longer keeps the
   point.
3. Run `WAYLAND_DEBUG=client DRAW_BACKEND=wayland draw 2>&1 | grep tablet`
   and bring the pen near the tablet: you see `tool_added`,
   `proximity_in`, then `motion`/`pressure`/`frame` bursts.
4. Run `WAYLAND_DEBUG=client DRAW_BACKEND=wayland draw 2>&1 | grep -E
   'wl_pointer.motion|tablet_tool_v2.(motion|down)'` and touch down: on
   Hyprland a `wl_pointer.motion` copy comes right before the tool's
   `motion` and `down`. Make `pointer_shadowed_by_pen` return `false` and
   run `cargo test replayed_wayland` to see the press lose its pressure.

## Further reading

- [[ADR-T35-1 Vendored miniquad with a pen tablet patch]],
  [[ADR-T35-2 Pressure as per-point width factors on pen strokes]],
  [[ADR-T40-1 Tablet stream wins over copied wl_pointer input]]
- Wayland tablet protocol: `tablet-v2.xml` in wayland-protocols.
- XInput 2 raw events: `XI2proto.txt`, section *Raw events*.
- The Cargo Book, *Overriding dependencies* (`[patch]`).
