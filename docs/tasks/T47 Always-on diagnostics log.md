---
id: T47
title: Always-on diagnostics log
status: todo
wave: 31
branch: task/T47-diagnostics-log
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task]
---

# T47 Always-on diagnostics log

## Goal

draw always keeps a cheap record of what it has been doing, without any
command-line flag or environment variable, so that a rare bug (letters
typed twice or three times, a crash after hours open, a segfault on
exit) can be reported from the log of the session where it happened,
instead of having to make it happen again.

## Known facts (2026-11-02)

- Today the only persistent record is the panic report of
  [[ADR-T27-3 Crash report from a panic hook and split debug symbols]]:
  `$XDG_STATE_HOME/draw/crash.log`, written by the panic hook. It does
  not cover segfaults or signals, and records nothing before the crash.
- Other output is ad-hoc `eprintln!` (`src/main.rs`, `src/shell/app.rs`)
  and `DRAW_FRAME_TIMES`; nothing reaches a file.
- The bugs it must help with live below draw, in the vendored miniquad
  Wayland backend (seat, keyboard, repeat, tablet events), so the
  backend must be able to log too.
- [[ADR-0014 Minimal dependencies and no unsafe]]: prefer a small
  hand-written logger over `tracing`/`log` + appenders, unless the ADR
  for this task argues otherwise.
- Must cost almost nothing per frame and per input event (i5 8th gen
  iGPU target, [[ADR-T24-5 Redraw budgets]]).

## Proposal (settle in the spec step)

1. **Flight recorder.** A fixed-size in-memory ring of recent events
   (e.g. the last few thousand: backend and seat events, key down/char
   with timestamps, tool changes, undo/redo, frame-time outliers, idle
   and focus changes). Written to disk when something goes wrong
   (panic, fatal signal, clean exit with an anomaly) and on demand.
2. **Session log on disk.** A small, size-capped, rotated file in
   `$XDG_STATE_HOME/draw/` (e.g. `draw.log`, `draw.log.1`) with
   low-volume lines: start-up (version, build id, backend, compositor,
   seat capabilities, devices), warnings, anomalies, exit reason. Never
   document contents (privacy): key events log keycodes and counts, not
   typed text, or the ADR decides.
3. **Anomaly detectors** that write a line by themselves, e.g. the same
   key `Char` delivered twice for one press, a second `wl_seat`/keyboard,
   a frame over budget, a long gap with no events (idle).
4. **On-demand dump**: a shortcut or `SIGUSR1` writes the ring to a
   timestamped file and shows where, so the owner can save it right
   after noticing a bug.
5. **Verbosity**: `DRAW_LOG=debug|trace` raises detail, but the default
   level is useful on its own.
6. Hook the crash report of ADR-T27-3 and the signal handling of
   [[T48 Clean exit on Ctrl-C]] into the same files.
7. A short doc: where the logs are and what to attach to a bug report.

## Spec

To be written in the spec step (ADR required: format, levels, sizes,
privacy, where the vendored backend logs).

## Out of scope

- Remote telemetry or sending anything over the network.
- Fixing the bugs it helps find (T45, T48, T49).

## Files owned

- `src/shell/` (new logging module), `src/main.rs`, `src/shell/crash.rs`
- `vendor/miniquad/` (logging hooks, `PATCHED.md`)
- new ADR `ADR-T47-*`, a feature note *Diagnostics log*

## Subtasks (one commit each)

- [ ] spec — `docs(T47): …`
- [ ] tests — `test(T47): …`
- [ ] models — `feat(T47): …`
- [ ] behaviour — `feat(T47): …`
- [ ] quality — `chore(T47): …`
- [ ] docs — `docs(T47): …`

## Learning path

Possibly a tutorial on a lock-free ring buffer / flight recorder.

## Log

- 2026-11-02 — note written at the owner's request, first step of the
  road to 2.0.0 ([[T50 Version 2.0.0 refactor]]).
