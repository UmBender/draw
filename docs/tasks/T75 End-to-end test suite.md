---
id: T75
title: End-to-end test suite
status: todo
wave: 34
branch: task/T75-e2e-tests
depends_on: [T73, T74]
adrs: []
feature: "[[End-to-end tests]]"
tutorial: ""
tags: [task, testing]
---

# T75 End-to-end test suite

## Goal

A test suite that runs the real `draw` binary in a real (headless)
display session. It sends real OS input, takes screenshots and checks the
result, so the parts the core tests never reach get tested too: the
macroquad/miniquad window and backend, `shell::input_map` key and pointer
translation, the renderer and its caches, the toolbar, the start-up
preflight, the T47 log and a clean exit. Each case starts from a
snapshot ([[T74 Editor snapshots for tests]]), plays an input script, and
checks the final editor snapshot and/or a screenshot against golden
files. It runs locally with one command and nightly on GitHub
([[T73 Continuous integration on GitHub]]).

## Known facts (2026-11-03)

- Everything in `src/core/` is tested headless (unit, `tests/fuzz.rs`,
  `tests/soak.rs`). `src/shell/` (`app.rs`, `backend.rs`, `input_map.rs`,
  `render.rs`, `toolbar.rs`, `tablet.rs`, `glyphs.rs`, `preflight.rs`,
  `crash.rs`, `diag/`) has only unit tests of its pure helpers.
- Native Wayland is the default backend; X11/Xwayland is deprecated
  ([[T46 Native Wayland by default and Xwayland deprecated]]). The
  patched miniquad lives in `vendor/miniquad` (ADR-T35-1) and `dlopen`s
  X11, GL, EGL, Wayland and libdecor.
- Past shell bugs that only showed up in the running app: doubled keys
  ([[T44 One keyboard per Wayland seat]],
  [[T45 Doubled keys after a long idle on Wayland]]), Ctrl-C segfault
  ([[T48 Clean exit on Ctrl-C]]), pen pressure on Wayland
  ([[T40 Pen pressure on native Wayland]]). These are the first
  candidates for e2e cases.
- Frame-time checks use `DRAW_FRAME_TIMES` and the perf scenes
  (500 text blocks / 5 000 strokes).

## Proposal (settle in the spec step)

1. **Session**: a headless Wayland compositor with software GL (e.g.
   `sway` with `WLR_BACKENDS=headless`, Mesa llvmpipe), a fixed output
   size and scale. Pick in the spec, with an ADR; Xvfb + `xdotool` stays
   a fallback for the X11 path only.
2. **Input**: real OS input through the compositor (e.g. `wtype` for
   keys, compositor IPC or a virtual pointer for the mouse), so the
   `input_map` translation is exercised. If a gesture can't be injected
   this way (pen pressure, tablets), say so in the case and use a
   scripted source instead.
3. **Driver hooks in the app**, test-only, gated by env vars:
   `DRAW_SNAPSHOT=<file>` to start (from T74),
   `DRAW_E2E_DUMP=<file>` to write the final snapshot on exit, and a
   "frame idle" signal (a log line or a file) so the harness knows when
   rendering has settled instead of sleeping.
4. **Cases** as files in `tests/e2e/<case>/`: `start.snap`, `input`
   (a small script: keys, text, moves, drags, waits),
   `expected.snap` and/or `expected.ppm` (screenshot via `grim -t ppm`,
   so no image decoder dependency), and a tolerance for the screenshot
   comparison (llvmpipe is deterministic, but fonts and AA may differ by
   a few pixels).
5. **Runner**: `scripts/e2e.sh` starts the compositor and runs an
   `#[ignore]`d integration test `tests/e2e.rs` that, per case, launches
   the release binary, plays the input, waits for idle, dumps, closes with
   a key and with `SIGINT`, and compares. On failure it keeps the actual
   snapshot, screenshot, diff image and `draw.log` in
   `target/e2e/<case>/`. `DRAW_UPDATE_SNAPSHOTS=1` refreshes goldens.
6. **Starter cases**: launch and quit (`Ctrl+Q`, `SIGINT`, close button)
   with a clean log; each creation tool draws once; typing a text block
   with a formula; key repeat and held modifiers (no doubled letters);
   toolbar clicks; pan/zoom over the 5 000-stroke scene; undo/redo after
   a selection move; resize of the window.
7. **CI**: a job in `nightly.yml` (T73) installs the compositor and
   tools, runs `scripts/e2e.sh`, and uploads `target/e2e/` on failure.

## Spec

To be written in the spec step. Expected acceptance criteria:

- **AC-1 One command.** `scripts/e2e.sh` runs every case on a machine
  with the listed packages and no display of its own, and exits non-zero
  on any failure. *Test:* the runner itself; output in *Log*.
- **AC-2 Real path.** At least the keyboard and pointer cases go through
  the compositor and `shell::input_map`, not straight into `Editor`.
  *Test:* a case whose expected result depends on the key translation
  (e.g. a shifted symbol in a text block).
- **AC-3 State and pixels.** A case can assert on the final snapshot, a
  screenshot within tolerance, or both; a mismatch prints a text diff or
  saves a diff image. *Test:* `tests/e2e.rs` self-tests with a
  deliberately wrong golden.
- **AC-4 No sleeps.** The harness waits on the app's idle signal with a
  timeout, never on fixed sleeps; a hung app fails the case after the
  timeout with its log attached. *Test:* a case with a timeout.
- **AC-5 Clean exit.** Every case ends with exit code 0, no
  `crash.log`, and no anomaly line in `draw.log`. *Test:* checked by the
  runner for every case.
- **AC-6 CI.** The nightly workflow runs the suite and uploads
  artifacts on failure. *Test:* one manual `workflow_dispatch` run,
  linked in *Log*.
- **AC-7 Test hooks are inert.** Without the env vars the binary
  behaves as before (no extra files, no idle signal). *Test:* unit tests
  of the env parsing.

## Out of scope

- Real pen tablet hardware (manual checklist, as in T49 AC-6).
- Frame-time budgets on CI (noisy runners; [[T14 Performance and release validation]]
  budgets stay on the target laptop).
- X11-only behaviour beyond a smoke case (Xwayland is deprecated).

## Files owned

- `tests/e2e.rs` (new), `tests/e2e/` (new cases and goldens)
- `scripts/e2e.sh` (new)
- `src/shell/app.rs`, `src/shell/diag/` (dump-on-exit and idle-signal
  hooks only)
- `.github/workflows/nightly.yml` (the e2e job only)
- `docs/features/End-to-end tests.md` (new), `docs/process/Quality Gates.md`
  (an *E2E* row), `docs/decisions/ADR-T75-1 …` (compositor and input choice)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T75): …`
- [ ] tests — `test(T75): …` (cases and goldens; the runner fails)
- [ ] models — `feat(T75): …` (env hooks, case format)
- [ ] behaviour — `feat(T75): …` (runner, comparison, CI job)
- [ ] quality — `chore(T75): …`
- [ ] docs — `docs(T75): …`

## Learning path

Optional tutorial: "Writing an end-to-end case", after the snapshot
tutorial of T74.

## Log

- 2026-11-03 — note written at the owner's request.
