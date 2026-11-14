---
id: T163
title: macroquad Platform interpreter and the loop
status: done
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

- **AC-1** — `MqPlatform` implements `Platform`; the recording platform keeps working with the same trait. Tests: `mq_platform_is_a_platform` (platform_mq.rs, a generic bound), `mq_events_collect_raw_events_in_arrival_order` (platform_mq.rs, the miniquad handler fills a `Vec<RawEvent>`), `tablet_events_are_empty_without_a_window` (platform_mq.rs, replaces `pen_samples_are_empty_without_a_window`, same assertion), `recording_platform_replays_queued_events` (recording.rs).
- **AC-2** — `app::run` is generic over `P: Platform` and calls no macroquad window, clipboard, pen or loop function. Test: `app_names_no_window_clipboard_pen_or_loop_function` (app.rs, scans the non-test source for the names; a call through the platform has a `.` before it).
- **AC-3** — A few frames of the app logic run over the recording platform. Test: `frames_over_the_recording_platform_resize_and_dispatch_input` (app.rs): a `Resize` and an `R` key become an editor resize and the rectangle tool; the calls are `Window`, `NextFrame` per frame.
- **AC-4** — `e2e.sh --backend all` green.

## Decisions

- Backend records and pen samples reach the collector as `RawEvent::Backend` and `RawEvent::Tablet` from `MqPlatform::poll`, in order with the input (ADR-T162-1). The count of records the queue dropped is logged to `diag` by `MqPlatform`, since `RawEvent` has no variant for it.
- The input phase of the loop is a function over `P: Platform` so a test can run it without a window. `ui_text` and `render` keep their own DPI reads: they draw through the macroquad context and belong to [[T164 Generic run over Platform and Gpu]].

## Out of scope

- The `MqGpu` thread-local, `run<P, G>` and the `match` in `main` ([[T164 Generic run over Platform and Gpu]]).

## Files owned

- `src/shell/platform_mq.rs`, `src/shell/app.rs` (the loop, window, clipboard and pen calls), `src/shell/backend.rs`, `src/shell/ui_text.rs` (the DPI read), `src/shell/recording.rs`, `src/shell/mod.rs`
- `crates/draw-sys/src/platform.rs` only if the trait misses a call (then an ADR amending ADR-T140-1)
- call sites in other `src/shell/` files and in `tests/`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T163): specify the slice`
- [x] tests — `test(T163): add failing tests`
- [x] models — merged into the behaviour commit (see Log)
- [x] behaviour — `refactor(T163): migrate the slice`
- [x] quality — `chore(T163): pass clippy and rustfmt`

## Log

- 2026-11-13 — done, `status: review`. `scripts/check.sh` and `scripts/e2e.sh --backend all` green.
  - Deviation: no separate `feat` models commit; `app::run`'s signature and the platform types changed together, so a stubbed step would not have built. The tests commit came first and did not compile.
  - `app::run` opens `MqPlatform` and calls the generic `run_on<P: Platform>`; the input phase is `gather_input`. `CachedFrame` has a `wake` flag and `LaserLayer::fading` for the wake-ups that called `schedule_update`.
  - Kept as is: `render.rs` and `ui_text.rs` read the DPI from macroquad when they draw; they go with the Gpu migration ([[T164 Generic run over Platform and Gpu]]).
  - Test change: `pen_samples_are_empty_without_a_window` became `tablet_events_are_empty_without_a_window` (same assertion); `mq_input_replays_events_in_order` feeds the collector from a `Vec<RawEvent>`.
  - Owner check: pen and tablet by hand (pen samples now arrive as `RawEvent::Tablet`).

- 2026-11-13 — split from T156 by the owner, following the batch's recommendation.
