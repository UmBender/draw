---
id: T198
title: Raw motion in the X11 tests on the Ubuntu runners
status: done
wave: 79
branch: fix/x11-raw-motion-xdotool
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, bug, ci]
---

# T198 Raw motion in the X11 tests on the Ubuntu runners

## Goal

The `gates` job of CI and the `fuzz` job of the Nightly are green again:
the X11 tests that need `XI_RawMotion` get it from `xdotool` on the
Ubuntu runners as well as on Fedora, and no X11 test passes without the
raw motion it is about.

## Known facts (2026-11-26)

- CI of `v2.0.1` (runs 45669104831, 45669099285) and the Nightly of
  2026-11-26 (45833907616) fail on one test only,
  `regression_t196_raw_motion_arrives_during_button_grab`
  (`crates/draw-sys/tests/x11_input.rs`):
  `XI_RawMotion stopped during button 1 grab (before=0, after=0)`.
  T196 never ran on CI before the merge; it passed locally.
- `before=0`: no raw motion arrived at all, not even before the button
  press. The fault is in the test input, not in the XInput 2.1 grab.
- Reproduced in `ubuntu:26.04` (the runner image) with the test binary
  built on Fedora: xdotool `1:3.20160805.1`, Xvfb `21.1.22`, libXi
  `1.8.2`. Fedora 44 has xdotool `3.20211022`, Xvfb `21.1.26`, libXi
  `1.8.3`, and passes.
- `xinput test-xi2 --root` in that container: `xdotool mousemove X Y`
  gives `Motion` but no `RawMotion` (the 2016 xdotool moves to an
  absolute point with `XWarpPointer`, which makes no raw event);
  `xdotool mousemove_relative DX DY` gives `RawMotion` (XTest relative
  motion). Buttons give `RawButtonPress`/`RawButtonRelease` in both.
- `x11_xi2_raw_motion_from_xtest_is_not_a_pen` passes on the runners
  without any raw motion, so it never checks what its name says.
- The same kind of xdotool version gap as T194
  (`getwindowclassname`).

## Spec

- **AC-1** — Regression: `x11_xi2_raw_motion_from_xtest_is_not_a_pen`
  fails unless raw motion arrived (`raw_motion_count() > 0`), and
  `regression_t196_raw_motion_arrives_during_button_grab` fails with
  its own message when no raw motion arrives before the press.
  Check: both red in `ubuntu:26.04` before the fix.
- **AC-2** — The X11 tests that need raw motion move the pointer with
  `xdotool mousemove_relative`, which sends XTest motion on every
  xdotool version.
  Check: `x11_input` green on Fedora and in `ubuntu:26.04`, and the
  `gates` job of CI and the `fuzz` job of the Nightly green on this
  branch.

## Out of scope

- The absolute moves of the other X11 tests and of `tests/e2e.rs`: they
  check core `Motion`, which `XWarpPointer` gives.
- Running the X11 tests in an Ubuntu container from `check.sh`.

## Files owned

- `crates/draw-sys/tests/x11_input.rs`
- `coverage-baseline.toml` (`x11`)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T198): specify raw motion in the X11 tests`
- [x] regression — `test(T198): require raw motion in the X11 raw motion tests`
- [x] behaviour — `fix(T198): move the pointer with XTest in the raw motion tests`
- [x] quality — `chore(T198): pass clippy and rustfmt`
- [x] behaviour — `fix(T198): record the x11 coverage the raw motion tests reach`
- [x] docs — `docs(T198): update the notes`

No models step: no new type.

## Log

- 2026-11-26 — written from the failed CI of `v2.0.1` and the Nightly,
  at the owner's request, so that 2.0.2 passes the Nightly.
- 2026-11-26 — AC-1: both tests red in `ubuntu:26.04` with the new
  asserts. AC-2: `x11_input` green on Fedora (`scripts/repeat.sh 20`,
  20/20) and in `ubuntu:26.04` (20/20 runs of the raw motion tests);
  `scripts/check.sh` green.
- 2026-11-27 — branch on CI (45929163015) and the Nightly run by hand
  (45929163265): every test passed and the Nightly was green; the
  `coverage` job failed only on the ratchet, `x11` rose from 90.3 to
  95.7 because the raw motion paths now run on the runners. The
  baseline takes that value (the others moved by 0.1).
- 2026-11-27 — CI green on `47673d0` (45933016302); merged into `main`
  for 2.0.2.
