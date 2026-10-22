---
id: ADR-T40-1
title: Tablet stream wins over copied wl_pointer input
status: accepted
kind: decision
date: 2026-10-22
task: T40
builds_on: ["[[ADR-T35-2 Pressure as per-point width factors on pen strokes]]"]
amends: ["[[ADR-T35-1 Vendored miniquad with a pen tablet patch]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T40-1 Tablet stream wins over copied wl_pointer input

## Context

[[ADR-T35-1 Vendored miniquad with a pen tablet patch]] turns each
`zwp_tablet_tool_v2` frame into pointer events plus one `miniquad::pen`
sample per event, and the shell pairs samples with pointer events by
position. It assumed the pen reaches the window only through the tablet
protocol.

Hyprland also copies the pen onto `wl_pointer`: hover motion, and a
synthetic `wl_pointer.motion` (time `0`) right before every tip-down, at
the same position as the tool's own motion. That copy has no sample, so
it takes the tool motion's sample, the tool motion takes the tip-down's,
and the press gets no pressure. A stroke whose first point has no width
factor never gets widths (ADR-T35-2), so every stroke on the native
Wayland backend has a constant width ([[T40 Pen pressure on native Wayland]]).

## Decision

While any tablet tool is in proximity of the window, the Wayland backend
drops `wl_pointer.motion` and `wl_pointer.button` events. The tool frames
already deliver the pen's motion and buttons, so nothing is lost; once the
tool leaves proximity, the mouse is read as before. The rule lives in
`miniquad::pen::pointer_shadowed_by_pen`, a pure function the backend
calls and draw's tests exercise. `wl_pointer.enter`, `leave` and `axis`
are unchanged.

The change is a `draw patch (ADR-T35-1)` like the rest and is listed in
`vendor/miniquad/PATCHED.md`.

## Alternatives considered

- **Pair in the shell instead** (e.g. skip pointer events with no sample
  at the position, or newest-first matching). The shell cannot tell a
  copied event from a mouse event at the same spot, and any matching rule
  breaks for some order of copies; the backend knows the tool is in
  proximity.
- **Start widths at factor 1 when the first point has no pressure.**
  Hides any pairing slip, but changes T35's AC-5 (no pressure → `widths`
  is `None`) and would still draw the stroke's start at the wrong width.
  Not taken.
- **Drop only the time-`0` copies.** Relies on one compositor's
  timestamps; the hover copies with real timestamps would still steal
  samples.

## Consequences

- Pen strokes on native Wayland start with the tip-down's pressure.
- A mouse moved while the pen hovers over the window is ignored until the
  pen leaves proximity. Using both at once is not a supported workflow.
- If a compositor sends `proximity_in` but never `proximity_out`, the
  mouse would stay ignored; tablet-unstable-v2 requires `proximity_out`
  when the tool leaves the surface or the tablet is removed.

## Rollback plan

`revert(T40)` the backend change and drop its `PATCHED.md` entry; add a
rollback ADR reverting this one.
