---
title: Pen tablets
task: "[[T34 Pen tablet input]]"
adrs: ["[[ADR-T34-1 Tap slop for pen tablets detected at start-up]]", "[[ADR-T35-1 Vendored miniquad with a pen tablet patch]]", "[[ADR-T35-2 Pressure as per-point width factors on pen strokes]]"]
tutorial: "[[34 Telling a tap from a drag]]"
shortcuts: []
tags: [feature]
---

# Pen tablets

## What it does

A pen tablet draws like a mouse: the tip drives the current tool, hovering
moves the cursor, and the barrel buttons act as the middle (pan) and right
(erase) buttons when the compositor passes them on. With the Pen tool,
**pressure** sets the width along the stroke: a normal (half) press draws
the chosen width, a light touch down to ¼ of it, a hard press up to 2×
(T35). This works on both backends, and the pen also draws on the native
Wayland backend. When a tablet is
attached draw switches to **pen mode**: a tap that stays within 16 px of
where the tip landed counts as a clean click, so a tap selects with Select
instead of dragging the shape, and makes no tiny shape with Rect.

## How to use

| Action | How |
|--------|-----|
| Pen mode | automatic when `/proc/bus/input/devices` lists a pen; the toolbar shows **pen** at the bottom |
| Turn it off (e.g. mouse users with a tablet plugged in) | `DRAW_TABLET=off draw` |
| Force it on | `DRAW_TABLET=on draw` |
| Pan / erase with the pen | barrel buttons, if they reach the app as middle / right (both do on native Wayland) |
| Thin / thick pen strokes | press lighter / harder; `[` / `]` still set the base width |

At start-up pen mode prints one line, e.g.
`draw: pen tablet "HUION 420 Pen" found: taps within 16 px are clicks;
DRAW_TABLET=off turns this off`. An unknown `DRAW_TABLET` value prints a
warning and detects as `auto` would.

**Map the tablet to one monitor.** Spread over two monitors, a 16:9 tablet
stretches strokes 2:1 and a 1 mm slip of the tip becomes 36 px. On
Hyprland: `output = current` in the `input:tablet` section; on X11:
`xinput map-to-output "<pen device>" <output>`.

## How it works

Tap slop: [[T34 Pen tablet input]]. Pressure and native Wayland:
[[T35 Pen pressure and native Wayland tablets]], tutorial
[[35 Reading a pen through a patched window library]].


- `core::tap::TapFilter` sits at the top of `Editor::handle`. After a press
  it holds back moves within the slop, delivers a release inside the slop
  at the press point, and lets the drag through once the pointer leaves the
  slop: the last held move first, then the move that left. Keys, wheel,
  characters and non-finite events pass unchanged. A slop of 0 (the
  default) is a no-op.
- `shell::tablet::choose` decides from `DRAW_TABLET` and the device list. A
  pen is a device with the `BTN_TOOL_PEN` key and the `ABS_X` / `ABS_Y`
  axes. The file is world-readable and visible inside the Flatpak sandbox.
- `shell::app::run` builds the editor with `with_tap_slop(16)` in pen mode;
  `shell::toolbar::pen_badge` is where the badge is drawn.

- **Pressure** (T35) comes from a patched miniquad in `vendor/miniquad`
  ([[ADR-T35-1 Vendored miniquad with a pen tablet patch]]). On X11 it reads
  the pen's `Abs Pressure` valuator from the raw XInput 2 events; on
  Wayland it binds `zwp_tablet_v2` and turns tool frames into pointer
  events. Before each pen pointer event it queues a `miniquad::pen`
  sample; `shell::input_map::Collector` pairs samples with events by
  position and emits `InputEvent::Pressure`.
- The editor hands the pressure to every tool in `Pointer::pressure`; only
  the Pen tool uses it, storing `core::pressure::width_factor` per point in
  `Shape::Stroke::widths`
  ([[ADR-T35-2 Pressure as per-point width factors on pen strokes]]). The
  smoother carries the factors through resampling, EMA and RDP;
  `Batch::polyline_widths` draws them.

## Limits

- **No tilt, rotation or eraser end.** Pressure only changes pen strokes;
  shapes, the eraser and selection ignore it.
- On X11 a pressure change with the pen held still shows up with the next
  movement.
- On the owner's HUION 420 under Hyprland only one barrel button reaches
  Xwayland apps (as middle), although the kernel reports both. The native
  Wayland backend gets both.
- miniquad cannot tell a pen from a mouse, so with a tablet attached the
  mouse gets the 16 px slop too: drags shorter than that become clicks.
