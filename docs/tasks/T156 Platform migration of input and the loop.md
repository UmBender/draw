---
id: T156
title: Platform migration of input and the loop
status: todo
wave: 68
branch: task/T156-platform-migration-of-input-and-the-loop
depends_on: [T140, T155]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T156 Platform migration of input and the loop

## Goal

Input, the clipboard, the pen, the seat and the main loop go through the `Platform` trait, and `main` picks the interpreter with a `match`: `fn run<P: Platform, G: Gpu>()`.

## Known facts (2026-11-13)

- Split out of [[T140 Platform and Gpu traits with recording interpreters]] AC-2 after the batch blocked it. See [[ADR-T140-1 The Gpu and Platform traits]].
- `input_map.rs`: `Collector` implements miniquad's `EventHandler` and keys on `KeyCode` (also `HeldMods`, `KeyWatch`, `diag::event::key_tag`). About 1,200 lines of its tests drive it with miniquad events and must move to `RawEvent`.
- `diag/` reads the patched miniquad's `BackendRecord`, `seat::device_change` and `pen::take_samples`/`pointer_shadowed_by_pen`; `RawEvent` must carry them losslessly.
- The pen and seat paths can only be checked by hand with a tablet, so this task needs the owner at the end.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `input_map`, `diag`, pen, seat, clipboard and the loop consume `RawEvent` and `Platform`; no miniquad type outside the macroquad interpreter.
- **AC-2** — The migrated `input_map` tests drive `RawEvent` and keep their assertions.
- **AC-3** — `main` monomorphises over `P` and `G` with a `match`; `e2e.sh --backend all` green.
- **AC-4** — Hand check with a tablet (pen pressure, tilt, eraser, seat changes), recorded in the Log by the owner.

## Out of scope

- Drawing (see [[T155 Gpu migration of the drawing]]); anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `src/shell/input_map.rs`, `diag/`, pen, seat and clipboard code, the loop in `app.rs`, `src/main.rs`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T156): specify the Platform migration`
- [ ] tests — `test(T156): move the input tests to RawEvent`
- [ ] models — `feat(T156): add the macroquad Platform interpreter`
- [ ] behaviour — `refactor(T156): run the shell over Platform and Gpu`
- [ ] quality — `chore(T156): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T140 AC-2 by the owner, following the batch's recommendation.
