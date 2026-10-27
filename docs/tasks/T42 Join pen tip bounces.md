---
id: T42
title: Join pen tip bounces
status: ready
wave: 28
branch: fix/pen-tip-bounce
depends_on: [T35]
adrs: []
feature: "[[Pen tablets]]"
tutorial: "[[35 Reading a pen through a patched window library]]"
tags: [task]
---

# T42 Join pen tip bounces

## Goal

A pen line drawn in one movement is one stroke. Today (1.4.3) the tablet
sometimes reports the tip lifting off for a split second mid-line. The
line then breaks into several strokes, each undone on its own.

## Known facts (2026-10-27, owner's HUION 420 on Hyprland, `DRAW_BACKEND=wayland`)

- A traced run (temporary `eprintln!` in `Collector` and `dispatch`, not
  committed) logged 226 pen down/up pairs. The gaps between a pen-up and
  the next pen-down form two clusters:

  ```
   10 ms ##########
   20 ms ##########################
   30 ms ################
   40 ms ###########
   50 ms #############
   60 ms ###
   ...   (sparse, 70–290 ms)
  300+ ms ###########################################  real lifts
  ```

- In the 10–60 ms cluster the next pen-down often lands 40–90 px further
  along the line: the hand kept moving, and the tip contact bounced.
- Some strokes last 0–10 ms: bounce fragments that end up as near-dots.
- The bounce reaches draw as a real tip up/down
  (`zwp_tablet_tool_v2.up`/`down` → `PenButton(Left)`), so the editor
  ends the stroke as told. Both backends hand pen presses to the same
  `Collector` (`src/shell/input_map.rs`).
- `Editor` has no clock (`src/core` is pure), so it cannot tell a bounce
  from a lift by itself.

## Proposal (settle in the spec step)

A new ADR: the shell holds back a **pen** left release for a bounce
window (about 60 ms). A pen left press within the window cancels both
events, and the stroke goes on, including the hover moves in between.
Otherwise the release is handed on when the window closes. A held
release also goes out before any key, scroll or other button event, so
the order of events is kept. Mouse buttons are never held. The
blocking event loop must wake when the window closes (`schedule_update`).

## Spec

_To be written in the spec step._

## Out of scope

- Pressure thresholds or libinput/compositor tip settings.
- The earlier report of one Ctrl+Z undoing several strokes: it could
  not be reproduced, and the traced run showed one undo per tap.

## Files owned

- `src/shell/input_map.rs`, `src/shell/app.rs`
- `docs/features/Pen tablets.md`,
  `docs/tutorials/35 Reading a pen through a patched window library.md`
- new ADR `ADR-T42-*`

## Subtasks (one commit each)

- [ ] spec
- [ ] tests
- [ ] models
- [ ] behaviour
- [ ] quality
- [ ] docs

## Learning path

No new tutorial step; extend step 35 with the bounce window.

## Log

- 2026-10-27 — Created from the owner's report that pen lines break
  while drawing, with the traced gap histogram above.
