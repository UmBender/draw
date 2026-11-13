---
id: T156
title: Platform migration of input and the loop
status: todo
wave: 68
branch: task/T156-platform-migration-of-input-and-the-loop
depends_on: [T140, T161]
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
- `crates/draw-sys/src/platform.rs`: `RawEvent::Backend(BackendRecord)` with the record types copied into `draw-sys`, plus the pure seat and pen helpers (`device_change`, `pointer_shadowed_by_pen`) if the shell still needs them
- a new macroquad `Platform` interpreter file in `src/shell/` (e.g. `platform_mq.rs`), `src/shell/recording.rs` (recording and null platforms), `src/shell/backend.rs`, `src/shell/tablet.rs`, `src/shell/mod.rs`
- call sites of the migrated API in other `src/shell/` files and in `tests/`
- the ADR this task writes (amending ADR-T140-1)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T156): specify the Platform migration`
- [ ] tests — `test(T156): move the input tests to RawEvent`
- [ ] models — `feat(T156): add the macroquad Platform interpreter`
- [ ] behaviour — `refactor(T156): run the shell over Platform and Gpu`
- [ ] quality — `chore(T156): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T140 AC-2 by the owner, following the batch's recommendation.
- 2026-11-13 — planned for a batch by the owner: *Files owned* widened in advance. The diag records travel as a `RawEvent::Backend` variant (one ordered stream, lossless), recorded in an ADR amending ADR-T140-1. `vendor/miniquad` stays as it is. AC-4 (tablet) is the owner's: the batch stops at `review`.
- 2026-11-13 — BLOCKED (batch): the task is too large for one unattended session and AC-3 hides a second migration. Findings: (1) `Collector` (1,736 lines, ~1,200 of tests) keys on `KeyCode` and `KeyMods` through `KeyWatch`, `HeldMods`, `diag::event::key_tag`, `map_key`; all must move to `RawKey`/`RawMods`, and `RawEvent` lacks the diag `BackendRecord` variant and a wheel/pen pairing shape that keeps `pair_pen` lossless. (2) `app::run` still reads `screen_width`, `screen_dpi_scale`, `clipboard_*`, `schedule_update`, `next_frame`, `pen::take_samples` and `repeat_all_miniquad_input` directly. (3) Rendering still reaches `MqGpu` through a thread-local (`render::with_gpu`, `render::submit`, `ui_text`, `CachedFrame`, `blit*`), so `fn run<P: Platform, G: Gpu>()` needs `CachedFrame`, the toolbar, the minimap, export and `ui_text` made generic over `G` first; T155/T157–T161 wrapped the drawing in `wrap_pass::<MqGpu>` but did not remove the thread-local. Nothing was changed in code. Recommended fix: split into T156a (RawEvent in `draw-sys` incl. `Backend` variant and the ADR; `Collector` and `diag` on `RawEvent`; the tests moved), T156b (macroquad `Platform` interpreter and `app::run` over `P`), T156c (drop the `MqGpu` thread-local, `run<P, G>` and the `match` in `main`, `e2e.sh --backend all`), then AC-4 by hand.
