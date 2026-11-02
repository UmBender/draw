---
id: T49
title: Long-running stability
status: todo
wave: 32
branch: fix/long-running-stability
depends_on: [T47]
adrs: []
feature: ""
tutorial: ""
tags: [task]
---

# T49 Long-running stability

## Goal

draw can stay open for a whole contest day (hours, with long idle
periods, screen lock and suspend) without input going wrong or crashing.
Each bug found this way gets its own fix and test; this task finds them
and adds a way to keep finding them.

## Known facts (2026-11-02)

- Owner's reports: letters typed twice or three times after the app has
  been open for a while ([[T44 One keyboard per Wayland seat]],
  [[T45 Doubled keys after a long idle on Wayland]]), and crashes with
  no obvious cause after prolonged use. No crash reports or backtraces
  saved yet.
- The panic hook writes `$XDG_STATE_HOME/draw/crash.log`; check it
  first. Segfaults leave only a core dump (`coredumpctl list draw`).
- Candidates for slow-building problems: compositor events after idle,
  suspend/resume or device re-plug (seat, keyboard, tablet), GL context
  loss after resume, unbounded growth (history, glyph atlas, caches,
  position index), timers and key repeat across focus changes.

## Proposal (settle in the spec step)

1. Prerequisite: [[T47 Always-on diagnostics log]], so every occurrence
   during normal use leaves a log.
2. Collect what exists: `crash.log`, `coredumpctl`, and owner sessions
   with the T47 log.
3. Soak run: a scripted long session (synthetic input through the core
   `Editor`, plus a manual idle/lock/suspend checklist on the target)
   that watches memory, frame times and the T47 anomaly detectors.
4. A bounded-growth test in core: after N thousand edits/undos the
   memory held by history, caches and indices stays within a budget.
5. One sub-entry in *Log* per bug found; a bug that needs a design
   change becomes its own task.

## Spec

To be written after the first evidence is collected.

## Out of scope

- The Ctrl-C segfault ([[T48 Clean exit on Ctrl-C]]).
- The broad refactor ([[T50 Version 2.0.0 refactor]]), though findings
  feed it.

## Files owned

- Decided per bug found.

## Subtasks (one commit each)

- [ ] spec — `docs(T49): …`
- [ ] tests — `test(T49): …`
- [ ] models — `feat(T49): …`
- [ ] behaviour — `fix(T49): …`
- [ ] quality — `chore(T49): …`
- [ ] docs — `docs(T49): …`

## Learning path

None expected.

## Log

- 2026-11-02 — note written from the owner's reports.
