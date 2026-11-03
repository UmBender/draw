---
title: Clean exit on signals
task: "[[T48 Clean exit on Ctrl-C]]"
adrs: ["[[ADR-T48-1 Signal handling in the vendored miniquad]]", "[[ADR-T47-1 Flight recorder and capped session log]]"]
tutorial: "[[48 Signals, self-pipes and async-signal safety]]"
shortcuts: ["Ctrl-C (in the terminal)"]
tags: [feature]
---

# Clean exit on signals

## What it does

`Ctrl-C` in the terminal that started draw, or `kill` (`SIGTERM`), closes
the window the normal way instead of killing draw mid-frame. The session
log records the signal and the exit. draw still ends *killed by* that
signal, so the shell shows status 130 / 143 and stops loops and scripts
as usual. A crash signal (`SIGSEGV`, `SIGABRT`, ...) writes a diagnostics
dump before draw dies.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Quit draw from its terminal | `Ctrl-C` |
| Quit at once if draw does not close | `Ctrl-C` again |
| Quit from a script | `kill <pid>` or `pkill -x draw` |

After a quit by signal, `draw.log` in the state directory
(`$XDG_STATE_HOME/draw`, or `~/.local/state/draw`) ends with:

```
INFO signal: SIGINT, window closed in order
INFO exit: SIGINT after 12m 3s with 0 anomalies
```

After a crash signal it has `ERROR fatal signal: SIGSEGV` and a
`dump-<time>-sigsegv.log` next to it. draw prints the dump's path on
stderr.

## How it works

`vendor/miniquad/src/signal.rs` installs the handlers. `src/main.rs`
calls `install_fatal(app::on_fatal_signal)` right after the
diagnostics log starts, and `install_quit()` right before the window
opens. The quit handler records the signal and writes a byte to a
self-pipe. The Wayland and X11 event loops `poll` that pipe next to the
display, so they wake up and order the quit. After the window is torn
down, `main` writes the log lines (`app::signal_line`,
`app::exit_reason`) and calls `signal::reraise`. The fatal handler runs
`app::on_fatal_signal` once, bounded by `alarm(2)`, then hands the
signal back to the previous action. See
[[ADR-T48-1 Signal handling in the vendored miniquad]].

## Limits

- The fatal hook is not async-signal-safe. If the crash happened inside
  `malloc` or while a lock was held, the dump can be skipped (recorder
  locked) or cut short by `SIGALRM` after two seconds. In that last case
  the core dump is lost too.
- A crash during a panic writes no second dump: the panic hook already
  wrote the `panic` one.
- `SIGQUIT`, `SIGHUP` and `SIGKILL` keep their default action.
- Fatal signals on threads draw does not create (Mesa, for instance) run
  the hook on that thread's stack. A stack overflow there loses the dump.
