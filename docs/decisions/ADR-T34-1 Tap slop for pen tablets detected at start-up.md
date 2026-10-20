---
id: ADR-T34-1
title: Tap slop for pen tablets detected at start-up
status: accepted
kind: decision
date: 2026-10-20
task: T34
builds_on: ["[[ADR-0003 Headless core and thin shell]]", "[[ADR-T28-1 Opt-in native Wayland backend]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T34-1 Tap slop for pen tablets detected at start-up

## Context

On X11 a pen tablet is just a pointer to draw: tip = left button, one
barrel button = middle. A pen tap, though, slips 10–40 px between press and
release on the owner's HUION 420 (≈ 18 px per mm of pen travel even when
mapped to one monitor), while the tools treat anything beyond 2–3 px as a
drag (`select::CLICK_SLOP_PX`, `shape_tool::MIN_DRAG_PX`). A Select tap
becomes a marquee, a Rect tap a tiny rectangle, a Text tap a drag.

miniquad 0.4 reports no device for a pointer event, so draw cannot tell a
pen tap from a mouse click. `/proc/bus/input/devices` is world-readable and
lists every input device with its key and axis bitmaps; the device nodes
under `/dev/input` are not readable by a normal user.

## Decision

1. **One filter in front of the editor.** `core::tap::TapFilter` sits at
   the top of `Editor::handle`. After a press it holds back the pointer
   moves while the pointer stays within the *slop* (a radius in screen
   pixels) of the press point. A release inside the slop arrives at the
   press point, so every tool sees a click with no movement. The first move
   outside the slop releases the last held move and itself, and from then on
   events pass unchanged until the release. Other buttons, keys, wheel and
   characters pass through. Non-finite pointer events pass through untouched,
   and the editor drops them as before. Only the last held move is kept, so
   the filter never allocates.
2. **Slop.** `tap::PEN_TAP_SLOP_PX = 16.0` screen pixels when on, `0`
   (filter off, events pass unchanged) by default. `Editor::with_tap_slop`
   sets it; `Editor::tap_slop` reads it.
3. **Turned on by the shell at start-up.** `shell::tablet::choose` is a
   pure function over the value of `DRAW_TABLET` and the text of
   `/proc/bus/input/devices`:
   - unset, empty or `auto` → on if a device has the `BTN_TOOL_PEN` key
     (0x140) and the `ABS_X` and `ABS_Y` axes, off otherwise;
   - `on` / `off` → forced; values are trimmed and ASCII-case-insensitive;
   - anything else → one warning line, then `auto`.
   Pen mode prints one notice line naming the pen and the way to turn it off,
   and the toolbar shows a `pen` badge at the bottom of the strip.
4. **Bitmaps.** A `B: KEY=` / `B: ABS=` value is a list of hexadecimal
   words, most significant first, each `usize::BITS` bits wide (the
   kernel's `long`).

## Alternatives considered

- **Raise every tool's threshold.** Each tool has its own rule (click,
  minimum drag, double-click re-edit, grid sizing). Changing all of them
  is many edits and still leaves the tool drawing the slip while the tip
  touches down.
- **Slop for everyone, always.** Mouse users would lose drags shorter than
  16 px (small dashes, small moves). The owner chose detection.
- **Time-based taps (press shorter than N ms).** Input events carry no
  time, and a slow, careful tap is still a tap.
- **Read the pen from evdev or Xinput2.** evdev needs the `input` group;
  Xinput2 or the Wayland tablet protocol needs a patched miniquad. That is
  [[T35 Pen pressure and native Wayland tablets]].
- **Keep every held move and replay them all.** Inside a 16 px circle the
  lost detail is invisible after smoothing, and a single slot needs no
  buffer.

## Consequences

- With a tablet attached, a mouse on the same machine gets the slop too:
  drags shorter than 16 px become clicks. `DRAW_TABLET=off` turns it off.
- A pen stroke's first 16 px show only after the pen leaves the circle, then
  catch up at once.
- The fuzz harness runs sessions with the slop on, so every invariant holds
  through the filter as well.

## Rollback plan

Construct the editor without `with_tap_slop` in `shell::app` (one line), or
revert the T34 merge; no document format or other decision depends on it.
