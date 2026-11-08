---
id: T75
title: End-to-end test suite
status: in-progress
wave: 34
branch: task/T75-e2e-tests
depends_on: [T73, T74]
adrs: ["[[ADR-T75-1 Headless sway session for end-to-end tests]]"]
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

Design in [[ADR-T75-1 Headless sway session for end-to-end tests]].

Everything needs `sway`, `wtype` and `grim`. Cases live in
`tests/e2e/<case>/`: `input`, optional `start.snap`, `expected.snap`, and
`expected.ppm` for the 5 cases where pixels are the point.

- **AC-1 One command.** `scripts/e2e.sh` runs every case on a machine
  with the listed packages and no display of its own, and exits non-zero
  on any failure. *Test:* `e2e_every_case`; output in *Log*.
- **AC-2 Real path.** The keyboard and pointer cases go through the
  compositor and `shell::input_map`, not straight into `Editor`.
  *Test:* case `text-formula`, whose text holds shifted symbols typed
  through `wtype`; case `creation-tools` for the pointer.
- **AC-3 State and pixels.** A case asserts on the final snapshot and,
  when it has a golden, a screenshot within its tolerance; a mismatch
  prints a text diff or saves a diff image. *Test:*
  `compare_snapshot_wrong_golden_prints_diff`,
  `compare_image_wrong_golden_reports_and_draws_diff`,
  `compare_image_within_tolerance_matches`.
- **AC-4 No sleeps.** The harness waits on the app's idle signal with a
  timeout, never on fixed sleeps; a hung app fails the case after the
  timeout with its log attached. *Test:*
  `wait_for_sync_hung_app_times_out_with_log`.
- **AC-5 Clean exit.** Every case ends cleanly: exit code 0, or for the
  SIGINT case death by that signal as [[ADR-T48-1 Signal handling in the vendored miniquad]]
  requires; no `crash.log`; no `WARN` or `ERROR` line in `draw.log`.
  *Test:* `check_exit_*`, run for every case by `e2e_every_case`.
- **AC-6 CI.** The nightly workflow runs the suite and uploads
  `target/e2e/` on failure. *Test:* one manual `workflow_dispatch` run,
  linked in *Log*.
- **AC-7 Test hooks are inert.** Without the env vars the binary behaves
  as before: no files, and the sync chord is not intercepted. *Test:*
  `hooks_from_env_unset_is_none`, `hooks_from_env_empty_is_none`,
  `hooks_is_sync_only_for_the_chord`,
  `hooks_write_nothing_until_a_sync`.
- **AC-8 Never the desktop.** The harness refuses to start any tool
  unless the sockets are inside the private runtime directory of
  `scripts/e2e.sh`. *Test:* `session_guard_refuses_*`, and
  `script_isolates_the_session`.

### Cases

`launch-close`, `launch-sigint`, `ctrl-q-unbound`, `creation-tools`,
`text-formula`, `key-repeat-modifiers`, `toolbar-clicks`,
`pan-zoom-5000`, `selection-move-undo`, `window-resize`.

Screenshot goldens: `creation-tools`, `text-formula`, `toolbar-clicks`,
`window-resize`, `selection-move-undo` (the fifth, chosen because it
crosses the document layer and the overlay).

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
- 2026-11-08 — UNATTENDED CHOICE: the app has no quit key (`Ctrl+Q` is
  not bound), so "close with a key" is not possible. Case `ctrl-q-unbound`
  proves the chord changes nothing, and the quit paths are the compositor
  close and `SIGINT`. Adding a quit key is an owner decision.
- 2026-11-08 — UNATTENDED CHOICE: the final snapshot is written at every
  sync, not on exit: the exit path is in `src/main.rs`, which this task
  does not own, and the last sync holds the same state.
- 2026-11-08 — UNATTENDED CHOICE: `swaymsg seat - cursor` needs a pointer
  on the seat, and a headless seat has none. The harness holds one
  virtual pointer open to announce it; `swaymsg` still drives every event.
- 2026-11-08 — Bug found, not fixed here (`vendor/miniquad` is not owned):
  `keyboard_handle_keymap` asserts on `mmap` and panics on a `no_keymap`
  event (size 0). Triggered by a new virtual keyboard while the seat has
  none active. The harness avoids it; it deserves its own task, with a
  `tests/e2e/regression-…` case.
