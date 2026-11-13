---
id: T162
title: RawEvent and the input collector
status: todo
wave: 68
branch: task/T162-rawevent-and-the-input-collector
depends_on: [T140, T161]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T162 RawEvent and the input collector

## Goal

Slice 1 of [[T156 Platform migration of input and the loop]]: `RawEvent` carries everything the shell reads from miniquad's input, and `Collector`, `HeldMods`, `KeyWatch`, `map_key` and `diag` consume `RawEvent`/`RawKey`/`RawMods` instead of miniquad's `KeyCode`/`KeyMods`/`BackendRecord`. A thin translation from miniquad's events to `RawEvent` keeps the app working.

## Known facts (2026-11-13)

- Split out of [[T156 Platform migration of input and the loop]] after the batch blocked it (too large for one session; findings in the T156 Log). The traits are in [[ADR-T140-1 The Gpu and Platform traits]]; the filter and readback calls in [[ADR-T159-1 Texture filter and readback in the Gpu trait]].
- `vendor/miniquad` is not changed. No pixel change allowed.
- `input_map.rs` is 1,736 lines, about 1,200 of tests driven by miniquad events.
- Owner decision (batch plan): the diag records travel as `RawEvent::Backend(BackendRecord)` with the record types copied into `draw-sys` (one ordered stream, lossless), in an ADR amending ADR-T140-1. The pen pairing (`pair_pen`) must stay lossless too; the shape is the session's choice, recorded.

## Spec

Draft: the spec step refines each criterion into named tests.

- **AC-1** — `RawEvent` has a `Backend` variant and whatever the pen pairing needs; `draw-sys` holds the record types and the pure helpers `device_change` and `pointer_shadowed_by_pen`.
- **AC-2** — `Collector`, `HeldMods`, `KeyWatch`, `map_key` and `diag::event` take `RawEvent`/`RawKey`/`RawMods`; no miniquad type in `input_map.rs` or `diag/` outside one translation module.
- **AC-3** — The migrated `input_map` and `diag` tests drive `RawEvent` and keep their assertions word for word.
- **AC-4** — `e2e.sh --backend all` green.

## Out of scope

- `app::run`, the clipboard, the window and the loop ([[T163 macroquad Platform interpreter and the loop]]); the `MqGpu` thread-local and `main` ([[T164 Generic run over Platform and Gpu]]).

## Files owned

- `crates/draw-sys/src/platform.rs` and `crates/draw-sys/tests/`
- `src/shell/input_map.rs`, `src/shell/diag/`, `src/shell/tablet.rs`
- a new translation module from miniquad events to `RawEvent` in `src/shell/` (e.g. `platform_mq.rs`), `src/shell/mod.rs`, `src/shell/recording.rs`
- call sites in other `src/shell/` files and in `tests/`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T162): specify the slice`
- [ ] tests — `test(T162): add failing tests`
- [ ] models — `feat(T162): add the types`
- [ ] behaviour — `refactor(T162): migrate the slice`
- [ ] quality — `chore(T162): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T156 by the owner, following the batch's recommendation.
