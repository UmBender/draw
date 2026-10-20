---
id: T35
title: Pen pressure and native Wayland tablets
status: todo
wave: 23
branch: task/T35-pen-pressure
depends_on: [T34]
adrs: []
feature: "[[Pen tablets]]"
tutorial: "[[35 Reading a pen through a patched window library]]"
tags: [task]
---

# T35 Pen pressure and native Wayland tablets

## Goal

Pen strokes get thicker the harder the pen presses, and the pen draws on the
native Wayland backend too. miniquad 0.4 reports no tablet events at all, so
this needs a patched miniquad. That overrides
[[ADR-0014 Minimal dependencies and no unsafe]] for one vendored crate and
needs its own ADR.

## Known facts (2026-10-20, see [[T34 Pen tablet input]])

- miniquad 0.4.11 X11: Xinput2 is loaded only for `XI_RawMotion`;
  tablet valuators (pressure is axis 2 on the Xwayland tablet device) are
  never selected.
- miniquad 0.4.11 Wayland: no `zwp_tablet_manager_v2`; Hyprland then sends
  pen motion as pointer motion but no tip or button presses.
- The kernel reports `ABS_PRESSURE` and both stylus buttons for the
  HUION 420 pen; `/dev/input/event*` is not readable by the user, so evdev
  is not an option.

## Open questions (owner decides in the spec step)

1. Vendor and patch miniquad (`[patch.crates-io]`), or move to another
   window library?
2. Pressure → width: which curve, and does it apply only to the pen tool or
   to shapes too?
3. Stroke model: per-point widths in `Shape::Stroke` (document format,
   tessellation, budgets ADR-T24-4).

## Spec

To be written in the spec step.

## Out of scope

- Tilt and rotation.

## Files owned

To be listed in the spec step.

## Subtasks (one commit each)

- [ ] spec — `docs(T35): …`
- [ ] tests — `test(T35): …`
- [ ] models — `feat(T35): …`
- [ ] behaviour — `feat(T35): …`
- [ ] quality — `chore(T35): …`
- [ ] docs — `docs(T35): …`

## Learning path

Step 35 — requires step 34 (pen tablets).

## Log

- 2026-10-20 — Created with T34: the owner chose pen ergonomics for 1.2.0
  and pressure later.
