---
id: ADR-T42-1
title: Pen tip bounces join the stroke
status: proposed
kind: decision
date: 2026-10-27
task: T42
builds_on: ["[[ADR-T34-1 Tap slop for pen tablets detected at start-up]]", "[[ADR-T35-1 Vendored miniquad with a pen tablet patch]]", "[[ADR-0003 Headless core and thin shell]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T42-1 Pen tip bounces join the stroke

## Context

On the owner's HUION 420 (Hyprland, 1.4.3), a pen line drawn in one
movement sometimes breaks into several strokes. A traced run showed the
tablet reporting a tip-up and a tip-down 10–60 ms apart, often 40–90 px
apart along the line. Deliberate lifts between strokes were 300 ms or
longer, with few gaps in between (see the T42 task note). draw ends a
stroke on every tip-up, so each bounce leaves a break and an extra undo
step.

The editor (`src/core`) has no clock and cannot tell a bounce from a
lift. The shell's `Collector` sees every event in order, knows which
pointer events came from the pen (they pair with a pen sample,
ADR-T35-1), and runs once per frame.

## Decision

1. **The shell holds back a pen's left release.** With a bounce window
   set, `Collector` keeps a left button release that paired with a pen
   sample instead of emitting it, and stamps it with the frame time
   (`Collector::set_now`).
2. **A press within the window joins.** A pen left press arriving while
   the release is held, no later than the window after it, drops both
   events and emits a `PointerMove` to the press position. The gesture
   goes on as one stroke.
3. **Moves are held with it.** Pointer moves while a release is held are
   not emitted; only the last one is kept, so memory stays constant.
   Pressure events still pass.
4. **Otherwise the release goes out.** `Collector::flush_bounce` emits
   the held release, then the last held move, once the window has
   passed. It returns whether a release is still held, and the app then
   calls `schedule_update` so the blocking loop wakes to flush it. Any
   key, character, wheel or other button event first flushes the held
   release, so events keep their order. A pen press after the window
   flushes the release and then starts a new stroke.
5. **Window.** `PEN_BOUNCE_WINDOW = 60 ms`, set only in pen mode
   (`tablet::Choice::pen`). The default is zero: nothing is held. Mouse
   releases (no pen sample) are never held.

## Alternatives considered

- **Filter in the core, like `TapFilter`.** The core input events carry
  no time, and adding timestamps to every `InputEvent` touches all
  tools and tests for one shell concern.
- **Join by distance only.** Bounces land up to 90 px away while
  deliberate lifts can be closer, so distance does not separate them.
  Time does.
- **Ask the user to change the tablet's pressure threshold.** Neither
  Hyprland nor libinput gives a per-device tip threshold for this
  tablet, and other apps would still need it.

## Consequences

- Broken lines from tip bounces become one stroke and one undo step.
- A release reaches the editor up to 60 ms (plus one frame) late. A
  tool's `Up` work (committing a stroke, a click) waits that long.
- Two deliberate strokes drawn less than 60 ms apart become one, with a
  straight segment between them. The trace had no deliberate lift that
  short.
- While a release is held, the loop runs every frame (about four frames)
  instead of blocking.

## Rollback plan

Set the window to zero in `app::run` (or revert the T42 commits). The
collector then passes every release unchanged, as in 1.4.3. Record the
rollback in a new ADR.
