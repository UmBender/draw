---
id: ADR-T94-1
title: Autosave captured on input and written off the frame
status: proposed
kind: decision
date: 2026-11-09
task: T94
builds_on: [ADR-T12-1, ADR-T24-5, ADR-T47-1, ADR-T74-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T94-1 Autosave captured on input and written off the frame

## Context

A crash or a stray close loses the whole drawing
([[T94 Autosave and crash recovery]]). `Editor::snapshot` and the snapshot
text format already hold the whole state
([[ADR-T74-1 Line-based editor snapshot format]]). Two limits apply:

- The event loop blocks until an event, and a timer would break the idle
  budget ([[ADR-T12-1 Blocking event loop with cached frame]]).
- An autosave must not disturb drawing. The overlay-only re-render budget
  is 2 ms ([[ADR-T24-5 Redraw budgets]]).

Measured on the development machine, release build,
`cargo test --release --test perf -- --ignored --nocapture perf_autosave`.
These are not numbers from the target laptop (i5-8250U).

| Scene | `Editor::snapshot` (clone) | `Snapshot::to_text` | Text size |
|-------|----------------------------|---------------------|-----------|
| 5 120 strokes (ADR-T24-5 scene) | 1.2 ms | 31 ms | 2.3 MB |
| The same, 64 MiB undo history | 59 ms | 1 170 ms | 77 MiB |

The clone meets the 2 ms budget on the ADR-T24-5 scene. The text does not
meet it, or one 16 ms frame.

## Decision

- **When.** The shell calls `Autosaver::on_input` once per frame that had
  input, after the frame is drawn. It captures when at least 30 s have
  passed since the last capture (the session start counts as the first),
  the document revision differs from the saved one, and no gesture or text
  edit is open. Nothing else wakes the loop.
- **What runs where.** The main thread only clones the state
  (`Editor::snapshot`). A short-lived thread makes the text, writes the
  file and drops the clone. While a writer runs, no new capture starts.
- **Files.** In the state directory: `autosave.snap` and
  `autosave.prev.snap`. The writer writes `autosave.snap.tmp.<pid>` with
  mode `0600`, syncs it, then renames it over `autosave.snap`. The first
  successful autosave of a session first renames the existing
  `autosave.snap` to `autosave.prev.snap`. Rotation comes after the
  temporary file is complete, so a full disk moves nothing.
- **Failures.** A failed write is logged at `ERROR` and changes nothing
  on screen. The same revision is tried again at the next due input, and
  the first-autosave rotation stays pending until a write succeeds.
- **Recovery.** `draw --recover` opens `autosave.snap`, `draw --recover
  prev` the older one. It wins over `DRAW_SNAPSHOT`. Any problem (missing,
  corrupt, wrong version, no state directory, an unknown `--recover`
  value) prints the reason, logs it, and starts blank.
- **Exit.** Nothing deletes the files on exit. A process exit during a
  write leaves the previous file intact.

## Alternatives considered

- **Serialise on the main thread** — the plan's default if under budget;
  it is 31 ms on the scene, about two frames.
- **A timer thread that wakes the loop** — breaks the idle budget.
- **Saving only the document** — needs a second format. The snapshot also
  keeps undo.
- **A fixed temporary name** — two open windows could interleave their
  writes and rename a mixed file.
- **Copying the history less** — the 59 ms worst case needs a persistent
  structure in `core::history`, a much larger change for a rare case.

## Consequences

- Recovered sessions keep undo, the view and the tools.
- The state directory holds at most two autosaves. Each can reach about
  77 MiB with a full history.
- With a full 64 MiB history, the capture costs one long frame (about
  59 ms) at most every 30 s. The ADR-T24-5 budget holds only without such
  a history.
- Two windows share the files: the last write wins, and each session's
  first autosave rotates.
- A `kill -9` during a write can leave one small `autosave.snap.tmp.<pid>`
  behind; it is never read.
- Autosaves follow the snapshot format, so a build with a new `VERSION`
  cannot recover a file from an older one ([[Editor snapshots#Limits]]).

## Rollback plan

Remove `shell::autosave`, its call in `shell::app`, and the `--recover`
parsing in `main`. A new ADR with `kind: rollback` records it. Existing
files in the state directory are harmless.
