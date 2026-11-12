---
id: ADR-T136-1
title: One gesture variant per tool in the machine
status: accepted
kind: decision
date: 2026-11-12
task: T136
builds_on: [ADR-T134-1, ADR-T135-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T136-1 One gesture variant per tool in the machine

## Context

The editor keeps the gesture in progress as a private
`Gesture::{Pan, Minimap, Tool { tool, fresh }}`: one variant for all tools,
with the tool as data. A tool-specific half-finished state cannot be named
or matched on its own.

## Decision

- `machine/gesture/` defines `Gesture` with one variant per tool
  (`Pen, Line, Arrow, Rect, Ellipse, Grid, Text, Eraser, Bucket, Select,
  Hand`) plus `Pan` and `Minimap`. A tool variant carries a `Press`: the
  driving button and a `Stage` (`Down`: no motion since the press;
  `Active`: the pointer moved). `Stage` replaces the old `fresh` flag.
- A gesture ends in an `End` (`Commit` on the release of its button,
  `Cancel` on any command, mode switch or Escape), never in a third state.
  The editor ends a gesture only through these two paths.
- `ActiveGesture` moves to the same module and keeps its public path.
- The sub-states a tool keeps inside itself (select's marquee, move and
  resize; the text caret) stay in `core/tools/**`. Their tests pass
  unchanged. UNATTENDED CHOICE: the tool sub-automata are not rewritten
  now, only named at the machine level.
- One history transaction per committed gesture is kept as it is: tools
  commit only on `Down` or `Up`, inside one `track_document` call.
- The snapshot format is unchanged: the `gesture tool <button> <tool>
  <fresh>` record is read and written from `Stage`.

## Alternatives considered

- **Rewrite each tool as a state machine inside the machine** — a rewrite
  of ~5k lines of tested tool code with no behaviour gain.

## Consequences

A gesture variant can be matched per tool. Snapshots do not change.

## Rollback plan

`git revert` the T136 behaviour commit.
