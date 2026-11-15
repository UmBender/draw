---
id: T141
title: Fuzz of the shell through the recording platform
status: done
wave: 68
branch: task/T141-fuzz-of-the-shell-through-the-recording-
depends_on: [T140, T167]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T141 Fuzz of the shell through the recording platform

## Goal

Raw input sequences are fuzzed against the input translation and the machine with no window.

## Known facts (2026-11-12)

- Today the fuzz covers the core, not `shell::input_map` or the drawing path.

## Spec

- **AC-1** — `tests/shell_fuzz.rs` generates raw events (key codes, repeats, modifiers, pointer, tablet, wheel, resize, focus, seats, typed characters) and runs `app::run` over a scripted `Platform` and `NullGpu` (which rejects bad sizes and handles); no panic, and the loop keeps asking for frames.
  Test: `shell_session_never_panics_and_runs_every_frame` (proptest; 256 cases locally, `PROPTEST_CASES=20000` in the nightly job).
- **AC-2** — No unbounded growth: per frame the platform sees at most 4 `schedule_update` calls and at most 4 clipboard writes.
  Test: `shell_session_platform_calls_per_frame_are_bounded`.
- **AC-3** — The fuzz is safe on the machine: it never generates `Ctrl+S`, `Ctrl+E`, any Ctrl+Shift chord (they write to `$HOME` or the state dir) nor Tab (the toolbar starts hidden, see the Log).
  Test: `generated_events_avoid_chords_that_write_files`.
- **AC-4** — The nightly job runs the fuzz with 20000 cases and uploads its regressions file on failure; `tests/ci.rs` stays green.
- **AC-5** — Every bug found has a `regression_T141_<slug>` test.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `tests/shell_fuzz.rs`, `tests/shell_fuzz.proptest-regressions`
- `.github/workflows/nightly.yml` (the 20000-case run)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T141): specify the shell fuzz`
- [x] tests — `test(T141): add the shell fuzz`
- [x] fixes — none: the fuzz found no bug at 256 cases (no `fix` commit, no regression test)

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-15 — owner decisions for the batch: the nightly job may be edited for the 20000 cases; the fuzz starts with the toolbar hidden (Tab), as the T167 frame test does, because the menu label measure needs a macroquad context until [[T149 Cutover removing miniquad and macroquad]].
- 2026-11-15 — implemented. `tests/shell_fuzz.rs` drives `app::run` over a scripted platform and `NullGpu`. Deviations: (1) `?` (shortcut sheet) is not generated, for the same macroquad measure reason as Tab, until T149; (2) the editor is owned by `run`, so the machine invariants are not read directly: the checks are no panic, every frame runs, and bounded `schedule_update` and clipboard calls per frame; (3) no Ctrl+S, Ctrl+E or Ctrl+Shift chords, which write to `$HOME`. Only the default 256 cases ran here (a larger `PROPTEST_CASES` run was not allowed in the batch): no bug found, so no `fix` commit. The nightly job runs 20000 cases; the owner should look at its first result. The lint pass (`chore`) removed a no-op `drop` and wildcard matches.
