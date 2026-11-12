---
id: T137
title: Automaton effects queue and time as an input
status: todo
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

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Every request leaves the machine as an `Effect`; the four `take_*` methods are removed.
  Tests: the editor tests for each request pass as effect assertions.
- **AC-2** — `Tick(Instant)` drives the laser fade, the autosave and any timer; no code in `draw-core` reads the clock.
  Test: `core_never_reads_the_clock` (guard over the source).
- **AC-3** — A scripted run with fake time reproduces the laser trail and the autosave schedule exactly.
  Tests: deterministic replays.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/machine/**`, `crates/draw/src/app.rs` (effect interpretation only)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T137): specify effects and time`
- [ ] tests — `test(T137): add failing tests for effects and ticks`
- [ ] behaviour — `refactor(T137): replace the request flags by effects`
- [ ] quality — `chore(T137): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — BLOCKED (batch): Files owned names `crates/draw/src/app.rs`, which does not exist (the shell is `src/shell/app.rs`). Removing the four `take_*` methods also needs `crates/draw-core/src/core/editor.rs`, and the laser and autosave timers live in `crates/draw-core/src/core/laser.rs` and `src/shell/autosave.rs`, all outside the owned paths. Recommended fix: amend Files owned (as for T135) to `machine/**`, `core/editor.rs`, `core/laser.rs`, `src/shell/app.rs`, `src/shell/autosave.rs`, then re-run.
