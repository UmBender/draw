---
id: T197
title: Idle frame wait ends early on X11
status: ready
wave: 78
branch: fix/x11-idle-frame-wait
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, bug, x11, ci]
---

# T197 Idle frame wait ends early on X11

## Goal

`x11_platform_schedule_update_ends_the_wait` passes on every CI run: an
unscheduled `next_frame` waits its `FRAME_WAIT` (16 ms) unless input
arrives, or the test learns which event legitimately ends the wait.

## Known facts (2026-11-24)

- Failed once in the Nightly `fuzz` job of T194 (run 45495469810): the
  idle `next_frame` returned after 5.07 ms, the test wants >= 8 ms
  (`crates/draw-sys/tests/x11.rs`). It passes locally 15/15 and under a
  parallel stress run.
- `X11Platform::next_frame` swaps, then `wait_events(16 ms)` unless an
  update is scheduled, the stash is not empty or the window has pending
  events. Any X event ends the wait early, including events the platform
  then ignores.
- Suspect: an event caused by the previous swap (GLX on Xvfb with
  software Mesa: `NoExpose`/`GraphicsExpose`, Present events) arrives
  during the next wait. Replacing the test's fixed 50 ms sleep by a
  100 ms drain did not help (1 failure in 8 local runs), which points at
  the swap rather than the map events.
- If swaps wake every idle frame, frame pacing on X11 is shorter than
  intended; idle at zero (`next_idle`) is not affected.

## Spec

Draft; the spec step names the final tests.

- **AC-1** — Find the event that ends the wait (log its type in the
  test) and either keep it from waking `next_frame` or make the test
  account for it. Test: `x11_platform_schedule_update_ends_the_wait`,
  20 runs in a row without failure (`scripts/repeat.sh`).

## Files owned

- `crates/draw-sys/src/x11/platform.rs`, `crates/draw-sys/src/x11/xlib.rs`
- `crates/draw-sys/tests/x11.rs`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T197): specify the idle frame wait`
- [ ] regression — `test(T197): add regression test for the early idle wake`
- [ ] behaviour — `fix(T197): keep swap events from ending the idle wait`
- [ ] quality — `chore(T197): pass clippy and rustfmt`
- [ ] docs — `docs(T197): update the notes`

## Log

- 2026-11-24 — split out of T194.
