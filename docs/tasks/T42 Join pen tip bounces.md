---
id: T42
title: Join pen tip bounces
status: in-progress
wave: 28
branch: fix/pen-tip-bounce
depends_on: [T35]
adrs: ["[[ADR-T42-1 Pen tip bounces join the stroke]]"]
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

Decision: [[ADR-T42-1 Pen tip bounces join the stroke]] — the proposal
is taken, with a window of 60 ms. Tests are `Collector` unit tests in
`src/shell/input_map.rs`; time is the frame time set with `set_now`.

- **AC-1** — a pen left release followed by a pen left press within the
  window emits neither; a move to the press position goes out instead:
  `collector_pen_bounce_within_window_joins_the_stroke`.
- **AC-2** — a held pen release, and the moves after it, stay back until
  the window has passed; `flush_bounce` then emits the release and the
  last held move, and says whether a release is still held:
  `collector_pen_release_is_held_inside_the_window`,
  `collector_pen_release_after_the_window_is_handed_on`.
- **AC-3** — a pen press after the window gets the release first, then
  starts a new stroke:
  `collector_pen_press_after_the_window_starts_a_new_stroke`.
- **AC-4** — a key, character, wheel or other button event while a
  release is held emits the release first:
  `collector_key_flushes_the_held_release_first`,
  `collector_other_button_flushes_the_held_release_first`.
- **AC-5** — a mouse release (no pen sample), and any release with no
  window set, passes at once:
  `collector_mouse_release_is_never_held`,
  `collector_without_a_window_passes_pen_releases`.
- **AC-6** — owner check: with `DRAW_BACKEND=wayland` and the HUION 420,
  quick lines no longer break, and separate strokes stay separate (see
  *Log*).

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

- [x] spec — `docs(T42): specify pen tip bounce acceptance criteria`
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
