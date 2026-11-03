---
id: ADR-T47-1
title: Flight recorder and capped session log
status: accepted
kind: decision
date: 2026-11-02
task: T47
builds_on: [ADR-0014, ADR-T27-3, ADR-T35-1, ADR-T44-1, ADR-T24-5]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T47-1 Flight recorder and capped session log

## Context

The bugs still open ([[T45 Doubled keys after a long idle on Wayland]],
[[T48 Clean exit on Ctrl-C]], [[T49 Long-running stability]]) are rare.
They show up after hours of use, and nobody can make them happen on
purpose. Before T47 the only record was the panic report of
[[ADR-T27-3 Crash report from a panic hook and split debug symbols]].
That report starts at the panic and says nothing about the input or
backend events that came before it. Most of these bugs live below draw,
in the vendored miniquad Wayland backend ([[ADR-T35-1 Vendored miniquad
with a pen tablet patch]], [[ADR-T44-1 Wayland seat devices follow the
announced capabilities]]).

The log must be on by default, so the session where the bug happens is
already recorded. It must cost close to nothing per frame and per
event on the target ([[ADR-T24-5 Redraw budgets]]). It must keep disk
use bounded however long draw stays open. It must not record what the
user types, and it must not add a dependency or `unsafe`
([[ADR-0014 Minimal dependencies and no unsafe]]).

## Decision

A hand-written `shell::diag` module with two outputs: a fixed-size ring
in memory (the *flight recorder*) and a small *session log* file.

**Flight recorder**
- A ring of the last **4 096** records, allocated once at start-up.
  A record is a timestamp (µs since start) plus a `Copy` event enum, at
  most **32 bytes**, so the ring is at most 128 KiB. Pushing a record
  allocates nothing, formats nothing and makes no system call.
- It records: backend events (seat capabilities, devices created or
  released, keyboard enter/leave, `wl_keyboard.key`, repeat info, repeat
  timer expirations, keymap, tablet tools and proximity, protocol
  errors), input (key down/up and char with their `repeat` flag,
  pointer down/up per button, wheel, a per-frame count of pointer
  moves), one record per drawn frame (redraw kind, CPU time, shape
  count), tool and document-revision changes, idle gaps, anomalies and
  dump requests.
- The app collects one frame's records in a reused buffer and pushes
  them under **one** lock per frame. The global is a `Mutex` because the
  panic hook must reach it. The hook uses `try_lock`, so a panic while
  the lock is held skips the dump rather than deadlocking.
- Text formatting happens only when a dump is written.

**Dumps**
- A dump writes the ring, oldest first, to
  `$XDG_STATE_HOME/draw/dump-<UTC yyyymmddThhmmssZ>-<reason>.log`:
  a header (version, build id, start time, reason, level), one line per
  record (at most 96 bytes), then the session's anomaly counts. So one
  dump is at most about 400 KiB.
- It is written on a panic (from the ADR-T27-3 hook), on the first
  anomaly of each kind (at most **3** automatic dumps per session) and
  on demand with **`Ctrl+Shift+L`**. The shell catches the chord
  before the editor, so it works in every mode. The path goes to stderr
  and to the session log.
- At most **5** dump files are kept. Writing a new one deletes the
  oldest.
- Fatal signals (`SIGSEGV`, `SIGABRT`) and `SIGUSR1` need a signal
  handler, and that needs `unsafe`. Where that code lives is
  [[T48 Clean exit on Ctrl-C]]'s decision. T47 only provides the
  `diag::dump(reason)` entry point that T48 will call.

**Session log**
- `$XDG_STATE_HOME/draw/draw.log`, with the directory chosen as for
  `crash.log` (fallback `~/.local/state/draw/`). One line per entry:
  `<UTC ISO 8601 with ms> <uptime> <LEVEL> <message>`.
- Capped at **512 KiB**. A line that would push the file over the cap
  first renames it to `draw.log.1`, replacing any older one. A file
  already over the cap at start-up is rotated the same way. Disk use is
  at most 1 MiB for the log plus 5 dumps.
- The file is opened once and written with one `write_all` per line.
  No flush and no `fsync`.
- Levels: `error`, `warn`, `info`, `debug`, `trace`. The default is
  `info`. `DRAW_LOG=off|error|warn|info|debug|trace` (case-insensitive)
  changes it. An unknown value keeps `info` and logs a warning.
  - `info` writes the start-up line (version, build id, pid, backend,
    session type, desktop, tablet mode, level), seat and device changes,
    keyboard repeat info, tablet tools, input after an idle gap,
    frames of 100 ms or more, dumps written, and the exit line (reason,
    uptime, anomaly counts).
  - `warn` writes anomalies; `error` writes panics and protocol errors.
  - `debug` adds keyboard focus changes and frames over the 16 ms
    budget.
  - `trace` also copies every ring record to the file, still under the
    cap.
- Each anomaly kind and the slow-frame line write at most **10** lines
  per session, then one "further … suppressed" line.

**Anomaly detectors** (pure structs, unit-tested)
- *Held key pressed again*: a non-repeat key down for a key that is
  already held. Window focus changes forget the held keys.
- *Several chars per press*: a second non-repeat char after one
  non-repeat key down.
- *Repeat burst*: one wake of the repeat timer reports more than 2
  expirations, as happens when the timer runs on through a suspend.
- *Keyboard re-created*: a second `wl_keyboard` within one session.
- *Backend queue overflow*: the backend dropped events before the app
  took them.
- *Idle*: input after more than 5 minutes without any. This writes an
  `info` line, not an anomaly.

**Privacy**
- A plain text key (a letter, digit, punctuation or space, with no
  `Ctrl` or `Alt`) is recorded as `text`, without saying which key. A
  char is recorded only by class (letter, digit, space, punctuation,
  other). Named keys (arrows, `Esc`, `Enter`, modifiers, …) and
  shortcuts (`Ctrl`/`Alt` + key) keep their identity, because they are
  what explains an editor action.
- The detectors see the real key, but only in memory, and keep only a
  held-key set. Nothing leaves the machine.

**Backend (vendored miniquad, draw patch ADR-T47-1)**
- A new `miniquad::diag` module holds a queue like the one in `pen.rs`:
  the Wayland callbacks push `(Instant, BackendEvent)`, and draw drains
  it once per frame with `take_events()`. The queue keeps at most 4 096
  events and counts the ones it drops. The raw evdev code sits in this
  queue for at most one frame. draw classifies it with the privacy
  rules above before it reaches the ring.

## Alternatives considered

- **`log`/`tracing` with a file appender** — two or more dependencies,
  formatting at every call site, and no ring buffer without a third
  crate. What draw needs is about 600 lines it can test.
- **Write every event to disk** — the file would grow with use and
  cost a system call per key. A crash would only show the events that
  happened to be flushed already.
- **A lock-free ring (atomics)** — almost every push happens on the
  main thread, once per frame. An uncontended `Mutex` costs tens of ns,
  and atomics would add complexity without a measurable gain.
- **Record typed characters** — would make doubled letters obvious, but
  every dump would then contain whatever the user typed. The class plus
  the detectors are enough to see the doubling.

## Consequences

- Every session leaves `draw.log`. After an anomaly, a panic or
  `Ctrl+Shift+L` it also leaves a dump the owner can attach to a bug
  report.
- Disk use is bounded by about 3 MiB in total. `crash.log` stays
  append-only as ADR-T27-3 says (one report per panic).
- The miniquad patch grows by one module and a few `push` calls. An
  upgrade has to carry them (`vendor/miniquad/PATCHED.md`).
- T48 hooks its signal handling into `diag::dump`. The detectors give
  T49 its soak-run signal.

## Rollback plan

Remove the `diag::init` call in `main` (nothing is recorded or written
then), or revert the T47 commits. `vendor/miniquad/src/diag.rs` and its
`push` calls come out with them. A rollback is a new ADR that reverts
this one.
