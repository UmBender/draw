---
id: T197
title: Idle frame wait ends early on X11
status: review
wave: 78
branch: fix/x11-idle-frame-wait
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, bug, x11, ci]
---

# T197 Idle frame wait ends early on X11

## Goal

`X11Platform::next_frame` waits out `FRAME_WAIT` (16 ms) when idle even
when X11 events that `X11Platform::poll` ignores arrive or are pending,
while still returning at once for input, resizes, visibility changes,
selection requests, a scheduled update, or a quit signal; and
`x11_platform_schedule_update_ends_the_wait` passes reliably on every CI
run.

## Known facts (2026-11-24)

- Failed once in the Nightly `fuzz` job of T194 (run 45495469810): the
  idle `next_frame` returned after 5.07 ms, the test wants >= 8 ms
  (`crates/draw-sys/tests/x11.rs`). Alone, it passes 40/40 locally, but
  running the full `tests/x11.rs` binary failed on run 15 of 20.
- Probing `wait_events` in `tests/x11.rs` showed two separate causes:
  1. **Test interference:** `x11_idle_sleeps_until_wakeup` lives in the
     same `tests/x11.rs` binary and calls `signal::install_quit()` plus
     `raise(SIGTERM)` after 50 ms + 60 ms. Because `signal::RECEIVED`
     and `signal::wake_fd()` are process-wide, that `SIGTERM` wakes
     `x11_platform_schedule_update_ends_the_wait`'s `wait_events` early
     (`rc=1, x_revents=0, wake_revents=1`). Raising the test's drain to
     100 ms aligned its 16 ms wait directly with the 110 ms signal,
     reproducing the wake immediately.
  2. **Platform idle wait:** `X11Platform::next_frame` swaps, skips the
     wait whenever `has_pending()` is true, and otherwise calls
     `wait_events(16 ms)` once. Any X11 event that `poll` ignores
     (`Expose`, `GraphicsExpose`, `NoExpose`, extension events like
     `ShmCompletion` or Present `GenericEvent` with `raw: None`,
     non-close `ClientMessage`, same-size `ConfigureNotify`,
     `SelectionNotify`, and `PropertyNotify` outside an outgoing `INCR`
     transfer) ends the wait early and shortens frame pacing on X11;
     `next_idle` is not affected.

## Spec

- **AC-1** — Regression: events that `X11Platform::poll` ignores do not
  end `X11Platform::next_frame`'s idle wait early; `next_frame` drops
  them and waits out the remaining `FRAME_WAIT` measured with `Instant`.
  Tests: `regression_T197_ignored_events_do_not_end_the_idle_wait`
  (`crates/draw-sys/tests/x11.rs`),
  `event_is_actionable_ignores_non_poll_events_and_keeps_real_input` and
  `wait_timeout_ms_rounds_positive_durations_up_to_milliseconds`
  (`crates/draw-sys/src/x11/platform.rs`).
- **AC-2** — An actionable event (input, resize, close, visibility or
  map change, raw pen motion, selection request/clear, or active `INCR`
  property delete), a scheduled update, or a quit signal still ends
  `next_frame` immediately without dropping the pending event.
  Tests: `x11_next_frame_wakes_immediately_on_actionable_event_without_dropping_it`
  and `x11_platform_schedule_update_ends_the_wait`
  (`crates/draw-sys/tests/x11.rs`, 20 consecutive runs with
  `scripts/repeat.sh`).
- **AC-3** — Regression: `x11_idle_sleeps_until_wakeup`'s process-wide
  `SIGTERM` is isolated from concurrent `X11Platform` tests in
  `tests/x11.rs`, and `next_idle` behaviour stays unchanged.
  Tests: `scripts/repeat.sh 20 cargo test -p draw-sys --test x11`,
  `x11_idle_skips_swap_and_wakes_on_scheduled_update`,
  `x11_idle_sleeps_until_wakeup` (`crates/draw-sys/tests/x11.rs`).

## Files owned

- `crates/draw-sys/src/x11/platform.rs`, `crates/draw-sys/src/x11/xlib.rs`
- `crates/draw-sys/tests/x11.rs`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T197): specify the idle frame wait`
- [x] regression — `test(T197): add regression test for the early idle wake`
- [x] behaviour — `fix(T197): keep swap events from ending the idle wait`
- [x] quality — `chore(T197): pass clippy and rustfmt`
- [x] docs — `docs(T197): update the notes`

No models step: no new public type.

## Log

- 2026-11-24 — split out of T194.
- 2026-11-24 — probed `tests/x11.rs` and `X11Platform::next_frame`:
  isolated `x11_platform_schedule_update_ends_the_wait` passed 40/40,
  while the full `tests/x11.rs` binary failed on run 15/20; logging
  `wait_events` caught `rc=1 x_revents=0 wake_revents=1` caused by
  `x11_idle_sleeps_until_wakeup` raising `SIGTERM` concurrently. Fixed
  scope covers both the platform's ignored-event wait loop (owner
  decision) and isolating the signal test in `tests/x11.rs`.
- 2026-11-24 — implemented `event_is_actionable`, `has_actionable_event`,
  and `wait_timeout_ms` in `crates/draw-sys/src/x11/platform.rs` so
  `X11Platform::next_frame` drops ignored X11 events and waits out the
  remaining `FRAME_WAIT` measured with `Instant`, keeping `next_idle`
  unchanged; isolated `x11_idle_sleeps_until_wakeup` in
  `crates/draw-sys/tests/x11.rs` with an `RwLock` and joined its signal
  thread.
- 2026-11-24 — verified `scripts/repeat.sh 20 cargo test -p draw-sys --test x11`
  (20/20 green) and `scripts/check.sh` (all checks passed).
