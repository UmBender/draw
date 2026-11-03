---
title: Diagnostics log
task: "[[T47 Always-on diagnostics log]]"
adrs: ["[[ADR-T47-1 Flight recorder and capped session log]]", "[[ADR-T27-3 Crash report from a panic hook and split debug symbols]]"]
tutorial: "[[47 A flight recorder for rare bugs]]"
shortcuts: ["Ctrl+Shift+L"]
tags: [feature]
---

# Diagnostics log

## What it does

draw always keeps a short record of what it has been doing: the last
4 096 input, backend and frame events in memory, and a small log file on
disk. When something goes wrong (a panic, a doubled key, a burst of key
repeats), the last moments before it are written to a dump file next to
the log. You can attach that file to a bug report instead of trying to
make the bug happen again. Nothing is sent anywhere, and the record
never contains what you typed.

## How to use

| Action | Shortcut / setting |
|--------|--------------------|
| Write a dump of the last moments right now (e.g. just after seeing a bug) | `Ctrl+Shift+L` (the path is printed on stderr and in `draw.log`) |
| Say more or less in `draw.log` | `DRAW_LOG=off\|error\|warn\|info\|debug\|trace` (default `info`) |

**Where the files are:** `$XDG_STATE_HOME/draw/`, usually
`~/.local/state/draw/`:

| File | What it holds | Size |
|------|---------------|------|
| `draw.log` (and `draw.log.1`) | One line per session event: start-up (version, build id, backend, desktop, tablet), seat and device changes, key repeat settings, input after a long idle, slow frames, anomalies, dumps, exit | at most 512 KiB each |
| `dump-<UTC time>-<reason>.log` | The flight recorder: one line per recent event, oldest first, then the anomaly counts | about 400 KiB at most; the 5 newest are kept |
| `crash.log` | The panic report of [[ADR-T27-3 Crash report from a panic hook and split debug symbols]] | one report per panic |

**What to attach to a bug report:** `draw.log`, the newest `dump-*.log`
(press `Ctrl+Shift+L` right after you notice the bug if no dump was
written by itself) and `crash.log` if draw crashed.

Levels: `info` is enough for most reports. `debug` adds keyboard focus
changes and frames over the 16 ms budget. `trace` copies every record
into `draw.log` too, which fills the 512 KiB quickly. `off` writes no log
and no automatic dumps, but `Ctrl+Shift+L` still works.

## How it works

`src/shell/diag/` ([[ADR-T47-1 Flight recorder and capped session log]]):

- `ring.rs`: a fixed-capacity ring, allocated once. `event.rs`: the
  32-byte `Copy` records and the privacy rule. A plain text key is
  `text` and a character is only its class (`letter`, `digit`, …).
  Named keys and `Ctrl`/`Alt` shortcuts keep their names.
- `detect.rs`: the anomaly detectors, which are *held key pressed
  again*, *several chars for one key press*, *key repeat burst*,
  *keyboard re-created* and *backend queue overflow*, plus the idle
  watch and the line limiter (10 lines per topic and session).
- `file.rs`: UTC timestamps, the rotating `SessionLog`, and dump names
  and pruning.
- `mod.rs`: `Diag` and the global recorder. The app loop pushes one
  frame's records under one lock (`shell::app::FrameRecorder`). The
  panic hook (`shell::crash`) uses `try_lock`.
- `vendor/miniquad/src/diag.rs`: the Wayland backend queues seat,
  keyboard, repeat timer, tablet and protocol error events. draw drains
  them every frame.

## Limits

- Fatal signals (`SIGSEGV`, `SIGABRT`) and `Ctrl-C` do not write a dump
  yet. `SIGUSR1` does not either. That is [[T48 Clean exit on Ctrl-C]].
- Backend events come only from the native Wayland backend. X11 records
  the app-level input and frames only.
- After a panic the dump is skipped if the recorder was locked at that
  moment. The crash report is still written.
- The idle line and the anomalies use frame time, so events within one
  frame share a timestamp. Backend events carry their own time.
