---
id: T163
title: macroquad Platform interpreter and the loop
status: todo
wave: 68
branch: task/T163-macroquad-platform-interpreter-and-the-loop
depends_on: [T162]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T163 macroquad Platform interpreter and the loop

## Goal

Slice 2 of [[T156 Platform migration of input and the loop]]: a macroquad `Platform` interpreter (`MqPlatform`) owns the window metrics, the clipboard, `schedule_update`, `next_frame`, the pen samples and the input repeat, and `app::run` reads them only through a `P: Platform`.

## Known facts (2026-11-13)

- Split out of [[T156 Platform migration of input and the loop]] after the batch blocked it (too large for one session; findings in the T156 Log). The traits are in [[ADR-T140-1 The Gpu and Platform traits]]; the filter and readback calls in [[ADR-T159-1 Texture filter and readback in the Gpu trait]].
- `vendor/miniquad` is not changed. No pixel change allowed.
- `app::run` reads `screen_width`, `screen_dpi_scale`, `clipboard_*`, `schedule_update`, `next_frame`, `pen::take_samples` and `repeat_all_miniquad_input` directly; `ui_text` reads `miniquad::window::dpi_scale`.

## Spec

Draft: the spec step refines each criterion into named tests.

- **AC-1** — `MqPlatform` implements `Platform`; the recording and null platforms keep working with the same trait.
- **AC-2** — `app::run` is generic over `P: Platform` and calls no macroquad window, clipboard, pen or loop function; a source scan test checks it (only `platform_mq.rs` and `gpu_mq.rs` may).
- **AC-3** — A test runs a few frames of the app logic over the recording platform.
- **AC-4** — `e2e.sh --backend all` green.

## Out of scope

- The `MqGpu` thread-local, `run<P, G>` and the `match` in `main` ([[T164 Generic run over Platform and Gpu]]).

## Files owned

- `src/shell/platform_mq.rs`, `src/shell/app.rs` (the loop, window, clipboard and pen calls), `src/shell/backend.rs`, `src/shell/ui_text.rs` (the DPI read), `src/shell/recording.rs`, `src/shell/mod.rs`
- `crates/draw-sys/src/platform.rs` only if the trait misses a call (then an ADR amending ADR-T140-1)
- call sites in other `src/shell/` files and in `tests/`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T163): specify the slice`
- [ ] tests — `test(T163): add failing tests`
- [ ] models — `feat(T163): add the types`
- [ ] behaviour — `refactor(T163): migrate the slice`
- [ ] quality — `chore(T163): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T156 by the owner, following the batch's recommendation.
