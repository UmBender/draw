---
id: T137
title: Automaton effects queue and time as an input
status: done
wave: 67
branch: task/T137-automaton-effects-queue-and-time-as-an-i
depends_on: [T133, T134]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T137 Automaton effects queue and time as an input

## Goal

The polling flags of the shell become one queue of effects, and timers become `Tick` inputs.

## Known facts (2026-11-12)

- Flags today: `take_export_request`, `take_svg_export_request`, `take_copy_image_request`, `take_save_request`, joined by `serve_requests` in `shell/app.rs`; timers for the laser, the autosave and the 100 ms settle.

## Spec

See [[ADR-T137-1 One effect queue and time as a parameter]].

- **AC-1** — Every request leaves the editor as an `Effect` through `Editor::take_effects`; the four `take_*` methods are removed.
  Tests: the existing editor tests for each request (export, save, SVG export, copy image) assert effects; new `take_effects_repeated_request_is_queued_once`, `take_effects_keeps_request_order`; machine test `step_ctrl_s_returns_save_effect`.
- **AC-2** — Time enters only as a value; no code in `draw-core` outside tests reads the clock.
  Test: `core_never_reads_the_clock` (guard over the source).
- **AC-3** — A scripted run with fake time reproduces the laser trail and the autosave schedule exactly.
  Tests: `laser_scripted_run_with_fake_time_is_reproducible` (core/laser.rs), `autosave_schedule_with_fake_time_is_reproducible` (shell/autosave.rs).

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/machine/**`, `crates/draw-core/src/core/editor.rs`, `crates/draw-core/src/core/laser.rs`, `src/shell/app.rs`, `src/shell/autosave.rs` (effect interpretation only)
- `tests/fuzz.rs` (only the call to the removed `take_copy_image_request`)
- this note, `ADR-T137-1`

## Subtasks (one commit each)

- [x] spec — `docs(T137): specify effects and time`
- [x] tests — `test(T137): add failing tests for effects and ticks`
- [x] behaviour — `refactor(T137): replace the request flags by effects` (models and behaviour in one commit)
- [x] quality — `chore(T137): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — amendment (owner triage, plan): `crates/draw/src/app.rs` does not exist; files owned are the list above, so the blocker below is resolved. `tests/fuzz.rs` added because it calls a removed method. A stash `batch-T137-spec` on the T136 branch holds a duplicate of these spec edits and can be dropped.
- 2026-11-12 — done: `Editor::take_effects` replaces the four `take_*` methods; the shell interprets effects (`serve_effects`, `arm_export`). `core_never_reads_the_clock` guards the core; the laser and the autosaver already took time as a value, so AC-3 is covered by replay tests on fake time (UNATTENDED CHOICE, ADR-T137-1: they are not moved into `Machine`). The machine test for AC-1 is the existing `step_ctrl_s_becomes_a_save_effect`. `scripts/check.sh` and `scripts/e2e.sh` green. Owner: try the laser trail (Q + move) and an autosave by hand.
- 2026-11-12 — (earlier run) BLOCKED (batch): Files owned names `crates/draw/src/app.rs`, which does not exist (the shell is `src/shell/app.rs`). Removing the four `take_*` methods also needs `crates/draw-core/src/core/editor.rs`, and the laser and autosave timers live in `crates/draw-core/src/core/laser.rs` and `src/shell/autosave.rs`, all outside the owned paths. Recommended fix: amend Files owned (as for T135) to `machine/**`, `core/editor.rs`, `core/laser.rs`, `src/shell/app.rs`, `src/shell/autosave.rs`, then re-run.
