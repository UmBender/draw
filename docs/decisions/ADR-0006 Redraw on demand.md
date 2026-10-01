---
id: ADR-0006
title: Redraw on demand
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0002, ADR-0003]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0006 Redraw on demand

## Context

A game-style loop redraws at the monitor refresh rate even when nothing
changes, which wastes battery and CPU on a weak laptop during a long contest.

## Decision

- `Editor::handle` reports whether state changed; the shell keeps a `dirty`
  flag.
- Preferred: miniquad's **blocking event loop** (`blocking_event_loop` in the
  platform config, with `schedule_update` on input), so the process sleeps
  while idle.
- Fallback, if the blocking loop misbehaves on the target's Wayland/XWayland
  setup: render the scene into a render target only when `dirty`, and on clean
  frames just blit that texture (one quad).
- Shapes outside the viewport are culled by AABB before drawing.

Which option is used is confirmed in [[T12 App shell and toolbar]] and recorded
in a follow-up ADR there.

## Alternatives considered

- **Always redraw at vsync** — simplest, but measurable idle CPU/GPU use.

## Consequences

- The shell must call `schedule_update`/mark dirty on every input it forwards.

## Rollback plan

Switching between the two strategies only touches `shell::app`.
