---
id: T188
title: Win32 window thread and message loop
status: todo
wave: 78
branch: task/T188-win32-window-thread-and-message-loop
depends_on: [T187]
adrs: []
feature: "[[Windows]]"
tutorial: ""
tags: [task, windows]
---

# T188 Win32 window thread and message loop

## Goal

`draw-sys` opens a Win32 window on its own thread, Per-Monitor V2 DPI
aware, 1280×800 logical pixels, with a dark title bar. It hands resize,
DPI, close and end-of-session events to the app thread, and implements
the window part of `Platform`. No GPU and no input yet.

## Known facts (2026-11-24)

- Decisions: the window thread of the [[Windows port proposal]] §3.4
  and the window details of §3.6, as ADRs written in
  [[T187 Windows ADRs and portable seams]].
- While the user drags or resizes, `DefWindowProc` runs a modal loop;
  the window thread may block there, the app thread must not.
- The window thread never waits on the app thread. Events cross over a
  channel plus an auto-reset event; the app thread waits on its handles
  with `WaitForMultipleObjects` in `next_frame` / `next_idle`. The swap
  chain's frame-latency handle joins that wait in [[T189 Direct3D 11 Gpu]].
- `EnableMouseInPointer(TRUE)` must be called once, before any window
  exists: done here, used by [[T190 Windows keyboard and pointer]].
- Per-Monitor V2 is set at run time with
  `SetProcessDpiAwarenessContext`, so test binaries get it too; the
  app's manifest repeats it in [[T192 Windows platform in main and the e2e suite]].
- Windows tests run only on the Windows CI job (proposal, question 7).

## Spec

Draft; the spec step names the final tests. Pure decoders live in
`win32/decode.rs`, compiled on every OS and tested on Linux; the rest of
`win32/` is `cfg(windows)`.

- **AC-1** — A window opens with title, class and 1280×800 logical
  size, scaled by the monitor's DPI, with the dark title bar.
  Tests: `win32_window_opens_with_title_and_logical_size`,
  `win32_window_uses_the_dark_title_bar`.
- **AC-2** — Resizes and DPI changes reach `window()` and `poll` as
  `Resize`; `WM_DPICHANGED` applies the suggested rectangle.
  Tests: `win32_resize_reaches_window_metrics`,
  `win32_dpi_change_updates_the_scale`, and pure
  `size_from_lparam`, `dpi_from_wparam`, `suggested_rect_from_lparam`.
- **AC-3** — `WM_CLOSE` sets `close_requested`; `WM_QUERYENDSESSION` and
  `WM_ENDSESSION` report a quit that the app flushes like `SIGTERM`.
  Tests: `win32_close_sets_close_requested`,
  `win32_end_session_reports_quit`.
- **AC-4** — `next_idle` sleeps at 0 % CPU until an event or
  `schedule_update`; it wakes while the window thread is in a modal
  loop. Tests: `win32_idle_wakes_on_schedule_update`,
  `win32_app_thread_wakes_during_a_modal_loop`.
- **AC-5** — Dropping the platform stops and joins the window thread and
  destroys the window. Test: `win32_drop_joins_the_window_thread`.
- **AC-6** — Keyboard, pointer, pen and clipboard return nothing yet,
  documented as T190 and T191.
  Test: `win32_platform_input_parts_return_nothing`.

Manual check (the owner's friend): move and resize the window and drag
it between monitors with different scales; the window follows.

## Out of scope

- The GPU and the swap chain ([[T189 Direct3D 11 Gpu]]); input
  (T190, T191); `main` ([[T192 Windows platform in main and the e2e suite]]).

## Files owned

- `crates/draw-sys/src/win32/**` (except `folders.rs`), the `win32`
  lines of `crates/draw-sys/src/lib.rs`
- `crates/draw-sys/Cargo.toml` (`windows` features), `Cargo.lock`
- `crates/draw-sys/tests/win32.rs`, `crates/draw-sys/tests/win32_pure.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T188): specify the Win32 window`
- [ ] tests — `test(T188): add failing tests for the Win32 window`
- [ ] models — `feat(T188): add the Win32 window types`
- [ ] behaviour — `feat(T188): implement the Win32 window thread`
- [ ] quality — `chore(T188): pass clippy and rustfmt`

## Log

- 2026-11-24 — written from the Windows port proposal.
