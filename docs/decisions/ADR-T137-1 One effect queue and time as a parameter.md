---
id: ADR-T137-1
title: One effect queue and time as a parameter
status: accepted
kind: decision
date: 2026-11-12
task: T137
builds_on: [ADR-T134-1, ADR-T136-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T137-1 One effect queue and time as a parameter

## Context

The editor kept four request flags (`save`, PNG export, SVG export, copy
image) with one `take_*` method each; the shell polled all four. The laser
and the autosaver already take the time as a parameter, but nothing
guarded the core against reading the clock.

## Decision

- The editor keeps one queue of `Effect` values. A command that asks the
  shell for something pushes `Effect::Save`, `ExportPng`, `ExportSvg` or
  `CopyImage(bounds)`; a request already queued is not queued twice (the
  old flags were idempotent). `Editor::take_effects` drains the queue, in
  the order of the requests. The four `take_*_request` methods are removed.
- `step` returns the drained effects; the shell, which still drives the
  editor directly, drains the same queue and interprets each effect.
- Time enters only as a value: `Input::Tick(Instant)` for the machine, an
  `f64` for the laser, an `Instant` for the autosaver. No file of
  `draw-core` outside test code reads the clock; the guard test
  `core_never_reads_the_clock` scans the sources.
- UNATTENDED CHOICE: the laser trail and the autosaver stay where they are
  (shell layer and `shell/autosave.rs`); they already depend only on the
  time passed in, so a scripted run with fake time is deterministic. Moving
  them into `Machine` is left out: the laser consumes events before the
  editor and the autosaver is shell I/O.

## Alternatives considered

- **Move laser and autosave state into `Machine`** — a larger change with
  no behaviour gain now.

## Consequences

One place lists what the shell must do. Tests assert effects.

## Rollback plan

`git revert` the T137 behaviour commit.
