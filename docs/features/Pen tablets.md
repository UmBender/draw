---
title: Pen tablets
task: "[[T34 Pen tablet input]]"
adrs: ["[[ADR-T34-1 Tap slop for pen tablets detected at start-up]]"]
tutorial: "[[34 Telling a tap from a drag]]"
shortcuts: []
tags: [feature]
---

# Pen tablets

## What it does

A pen tablet draws like a mouse: the tip drives the current tool, hovering
moves the cursor, and the barrel buttons act as the middle (pan) and right
(erase) buttons when the compositor passes them on. When a tablet is
attached draw switches to **pen mode**: a tap that stays within 16 px of
where the tip landed counts as a clean click, so a tap selects with Select
instead of dragging the shape, and makes no tiny shape with Rect.

## How to use

| Action | How |
|--------|-----|
| Pen mode | automatic when `/proc/bus/input/devices` lists a pen; the toolbar shows **pen** at the bottom |
| Turn it off (e.g. mouse users with a tablet plugged in) | `DRAW_TABLET=off draw` |
| Force it on | `DRAW_TABLET=on draw` |
| Pan / erase with the pen | barrel buttons, if they reach the app as middle / right |

At start-up pen mode prints one line, e.g.
`draw: pen tablet "HUION 420 Pen" found: taps within 16 px are clicks;
DRAW_TABLET=off turns this off`. An unknown `DRAW_TABLET` value prints a
warning and detects as `auto` would.

**Map the tablet to one monitor.** Spread over two monitors, a 16:9 tablet
stretches strokes 2:1 and a 1 mm slip of the tip becomes 36 px. On
Hyprland: `output = current` in the `input:tablet` section; on X11:
`xinput map-to-output "<pen device>" <output>`.

## How it works

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

## Limits

- **No pressure, tilt or eraser end.** miniquad 0.4 does not report them;
  see [[T35 Pen pressure and native Wayland tablets]].
- **Native Wayland backend:** the pen moves the cursor but cannot draw
  (miniquad has no `zwp_tablet_v2`, and Hyprland does not turn tip
  presses into clicks for such clients). Use the default X11 backend.
- On the owner's HUION 420 under Hyprland only one barrel button reaches
  Xwayland apps (as middle), although the kernel reports both.
- miniquad cannot tell a pen from a mouse, so with a tablet attached the
  mouse gets the 16 px slop too: drags shorter than that become clicks.
