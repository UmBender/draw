---
id: T48
title: Clean exit on Ctrl-C
status: in-progress
wave: 32
branch: fix/clean-exit-on-sigint
depends_on: [T47]
adrs: [ADR-T48-1]
feature: ""
tutorial: ""
tags: [task]
---

# T48 Clean exit on Ctrl-C

## Goal

Stopping draw with `Ctrl-C` in the terminal (`SIGINT`), or with
`SIGTERM`, ends the process cleanly with exit code 130 / 143 instead of a
segmentation fault.

## Known facts (2026-11-02)

- Owner's report: `Ctrl-C` in the terminal that started draw ends in a
  segfault.
- draw installs no signal handler; the only hook is the panic hook of
  ADR-T27-3, which does not see signals. `panic = "abort"` in release.
- `unsafe_code = "forbid"` in draw ([[ADR-0014 Minimal dependencies and
  no unsafe]]), so a handler cannot live in draw's crate itself; the
  vendored miniquad (which already has `unsafe` FFI) or a tiny
  dependency would host it.
- Not yet known: whether it happens on both backends (X11 and
  `DRAW_BACKEND=wayland`), in the Flatpak and in `target/release`, and
  where it crashes (likely the default `SIGINT` killing the process while
  a GL/EGL or Wayland thread is still running, or atexit handlers
  tearing down the GL driver under a live context).

## Proposal (settle in the spec step)

1. Reproduce on both backends and get a backtrace (`coredumpctl`,
   `gdb`, `addr2line` with `draw.debug`).
2. Catch `SIGINT`/`SIGTERM` (self-pipe or an atomic flag checked by the
   event loop) and turn them into the normal quit path
   (`miniquad::window::order_quit`), so the window and GL context are
   torn down in order. A second `Ctrl-C` exits at once.
3. Log the signal through [[T47 Always-on diagnostics log]], and on
   `SIGSEGV`/`SIGABRT` write the flight recorder before dying.
4. ADR on where the `unsafe` signal code lives.

## Spec

Reproduction (2026-11-02, Hyprland 0.56.2, draw 1.4.3 at `54dbc22`): no
segfault. A real `Ctrl-C` through a pty, `kill -INT` and `kill -TERM`, on
Wayland and X11 (`DRAW_BACKEND=x11`), in `target/release` and
`target/debug`, under `cargo run` and in the installed Flatpak: every run
ends killed by the signal (shell status 130 / 143) with no core dump.
Closing the window exits 0 on both backends. The only draw segfaults in
`coredumpctl` are from 2026-10-14 (pre-1.1.0). They crashed in glibc's
`__run_exit_handlers` after `main` returned, which is the normal-exit
path. The owner chose to make the quit graceful anyway: the window is
torn down in order, the signal and exit reach the session log, and fatal
signals leave a dump.

Design: [[ADR-T48-1 Signal handling in the vendored miniquad]]. Code:
`vendor/miniquad/src/signal.rs` (new, `pub mod signal`), the wait in the
Wayland and both X11 event loops, `src/main.rs` and `src/shell/app.rs`.
The miniquad API is reached as `macroquad::miniquad::signal` and tested
from `app::tests`, because Cargo does not run a path dependency's tests.

- **AC-1 Signal names.** `QuitSignal` covers `SIGINT` (2) and `SIGTERM`
  (15). `FatalSignal` covers `SIGSEGV`, `SIGBUS`, `SIGILL`, `SIGFPE` and
  `SIGABRT`. Each one maps to and from its Linux number and has its
  `SIG*` name. Any other number maps to `None`.
  `app::tests::quit_signal_numbers_and_names_match_linux`,
  `fatal_signal_numbers_and_names_match_linux`,
  `signal_from_unknown_number_is_none`.
- **AC-2 A caught signal is recorded.** After `install_quit()`, a
  `SIGTERM` sent to the process does not kill it. `received()` then
  returns `Some(QuitSignal::Terminate)` within a second, and the first
  signal wins over later ones.
  `app::tests::install_quit_records_a_sent_sigterm`.
- **AC-3 Exit reason.** With no signal the exit reason is
  `window closed`. With a signal it is the signal's name, and the log
  gets `signal: SIGINT, window closed in order` before the exit line.
  `app::tests::exit_reason_without_signal_is_window_closed`,
  `exit_reason_with_signal_is_its_name`, `signal_line_names_the_signal`.
- **AC-4 Fatal dump reason.** A fatal signal's dump is named for the
  lowercase signal (`sigsegv`), and its log line is
  `fatal signal: SIGSEGV`.
  `app::tests::fatal_dump_reason_is_lowercase_name`,
  `fatal_line_names_the_signal`.
- **AC-5 End to end (manual, on the target).** On Wayland and on X11, in
  `target/release`: a `Ctrl-C` through a pty and a `kill -TERM` end draw
  within 1 s, killed by the same signal (status 130 / 143), with no core
  dump. The session log shows `signal: SIGINT, window closed in order`
  and `exit: SIGINT after ...`. Closing the window still exits 0 with
  `exit: window closed`. A `SIGINT` while the loop cannot quit (second
  signal) kills at once. `kill -SEGV` leaves a `sigsegv` dump and a core
  dump. Results go in *Log*.

## Out of scope

- Saving the document on exit (draw has no file format).

## Files owned

- `src/main.rs`, `src/shell/app.rs`
- `vendor/miniquad/` (if the handler lives there, `PATCHED.md`)
- new ADR `ADR-T48-*`

## Subtasks (one commit each)

- [x] spec — `docs(T48): …`
- [ ] tests — `test(T48): …`
- [ ] models — `feat(T48): …`
- [ ] behaviour — `fix(T48): …`
- [ ] quality — `chore(T48): …`
- [ ] docs — `docs(T48): …`

## Learning path

Possibly a tutorial on async-signal-safe handlers.

## Log

- 2026-11-02 — note written from the owner's report; not reproduced yet.
- 2026-11-02 — not reproduced on any backend or build (see *Spec*); owner
  chose a graceful quit anyway. ADR-T48-1 proposed.
