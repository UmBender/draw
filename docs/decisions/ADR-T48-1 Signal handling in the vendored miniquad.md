---
id: ADR-T48-1
title: Signal handling in the vendored miniquad
status: proposed
kind: decision
date: 2026-11-02
task: T48
builds_on: [ADR-0014, ADR-T35-1, ADR-T27-3, ADR-T47-1, ADR-T12-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T48-1 Signal handling in the vendored miniquad

## Context

[[T48 Clean exit on Ctrl-C]] was opened from a report of a segfault on
`Ctrl-C`. On 2026-11-02 the crash could not be reproduced: a real
`Ctrl-C` through a pty, `kill -INT` and `kill -TERM` on Wayland and X11,
in release and debug builds, under `cargo run` and in the Flatpak, all
end with the process killed by the signal (shell status 130 / 143) and
no core dump. The only draw segfaults in `coredumpctl` are from
2026-10-14, before 1.1.0. They crashed in glibc's `__run_exit_handlers`
after `main` returned, which is the normal-exit path, not the signal.

draw still has no say in what happens on a signal. The default action
kills the process mid-frame. The window and GL/EGL context are not torn
down in order, the session log ([[ADR-T47-1 Flight recorder and capped
session log]]) gets no exit line, and a fatal signal (`SIGSEGV`,
`SIGABRT`, ...) leaves no dump, although the flight recorder exists for
exactly that case.

Catching a signal needs `sigaction`, `pipe2`, `write` and `raise`. All
of these are `unsafe` FFI, and draw forbids `unsafe`
([[ADR-0014 Minimal dependencies and no unsafe]]). The vendored miniquad
([[ADR-T35-1 Vendored miniquad with a pen tablet patch]]) already
depends on `libc` and already contains the event loops that block in
`poll` / `XNextEvent` ([[ADR-T12-1 Blocking event loop with cached frame]]). A signal has
to wake those loops.

## Decision

The signal code lives in a new module `vendor/miniquad/src/signal.rs`
(`draw patch (ADR-T48-1)`). draw calls its safe API from `src/main.rs`
and `src/shell/app.rs`.

**Quit signals (`SIGINT`, `SIGTERM`)**
- `signal::install_quit()` creates a self-pipe (`pipe2`, `O_NONBLOCK |
  O_CLOEXEC`) and installs a `sigaction` handler with `SA_RESETHAND`.
  The handler stores the signal number in an atomic (the first one wins)
  and writes one byte to the pipe. Both calls are async-signal-safe.
- The event loops wait on the pipe's read end as well as on the display:
  the Wayland loop adds it as a third `pollfd`, and the X11 loops `poll`
  the X connection and the pipe instead of blocking in `XNextEvent` when
  no event is queued. After the wait, a received signal sets
  `quit_ordered`. The loop ends and the window is torn down the normal
  way.
- `SA_RESETHAND` puts the default action back after the first signal,
  so a second `Ctrl-C` kills draw at once even if the loop hangs.
- After the window closes, `main` writes `signal: SIGINT` and the exit
  line (`exit: SIGINT after ...`) to the session log. Then
  `signal::reraise(sig)` restores the default action and raises the
  signal again, so the parent sees draw *killed by* `SIGINT` / `SIGTERM`
  (shell status 130 / 143). Exiting with code 130 instead would break the
  convention that shell loops and `make` rely on to stop on `Ctrl-C`.

**Fatal signals (`SIGSEGV`, `SIGBUS`, `SIGILL`, `SIGFPE`, `SIGABRT`)**
- `signal::install_fatal(hook)` installs an `SA_SIGINFO | SA_ONSTACK |
  SA_RESETHAND` handler and keeps the previous action, such as Rust's
  stack-overflow handler for `SIGSEGV` and `SIGBUS`.
- The handler first arms `alarm(2)`, then calls the hook once. draw's
  hook writes an `ERROR fatal signal: SIGSEGV` line and a `sigsegv` dump
  through `diag::try_with`. It skips both while a panic is in progress,
  because the panic hook already wrote the `panic` dump just before
  `abort`.
- Then the handler puts the previous action back. A fault raised by the
  kernel (`si_code > 0`) returns, the instruction faults again, and the
  previous handler or the default action (a core dump) takes over. A
  signal sent by `kill`/`raise` is raised again with the default action.

The hook is not async-signal-safe: it may allocate and take locks.
That is a known trade-off. `try_lock` avoids a deadlock on the
recorder, and `alarm(2)` bounds a deadlock elsewhere (for example in
`malloc`): `SIGALRM` ends the process after two seconds instead of
leaving it hung. The core dump is lost in that case, but the process
still dies.

## Alternatives considered

- **Do nothing (report as not reproduced).** The default action is
  already "killed by `SIGINT`". But the log would still get no exit line
  and fatal signals would still leave no dump.
- **A tiny dependency (`signal-hook`, `ctrlc`).** That would be a new
  dependency against ADR-0014. It also cannot wake miniquad's blocking
  loops without a patch to miniquad anyway.
- **A watcher thread that calls `order_quit`.** It still cannot wake a
  loop blocked in `poll`/`XNextEvent`, so draw would quit only on the
  next input event.
- **`std::process::exit(128 + sig)`.** It gives the same status number,
  but the parent sees a normal exit, not a death by signal.
- **Replacing Rust's `SIGSEGV` handler outright.** Stack overflows would
  lose their "has overflowed its stack" message. Chaining back to the
  previous action keeps it.

## Consequences

- `Ctrl-C` and `SIGTERM` close the window in order, and the session log
  gets a `signal:` line and an exit line. A second `Ctrl-C` still kills
  draw at once.
- A segfault or abort outside a panic leaves a dump next to the session
  log. In the worst case the dump is lost and the process dies on
  `SIGALRM`.
- `vendor/miniquad` carries one more patch (`PATCHED.md`). An upgrade
  must keep the extra `pollfd` in the Wayland loop and the `poll` before
  `XNextEvent` in the X11 loops.
- The X11 loops no longer block inside `XNextEvent`. `XPending` returns
  0 only when nothing is queued, and the `poll` then sleeps.

## Rollback plan

Revert the behaviour commit of T48. `main` stops calling
`signal::install_quit`/`install_fatal` and the loops go back to
upstream. A rollback ADR `reverts: [ADR-T48-1]`.
