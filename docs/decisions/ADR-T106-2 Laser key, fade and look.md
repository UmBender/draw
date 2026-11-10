---
id: ADR-T106-2
title: Laser key, fade and look
status: accepted
kind: decision
date: 2026-11-10
task: T106
builds_on: [ADR-T106-1, ADR-T11-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T106-2 Laser key, fade and look

## Context

New keys need an ADR (batch rules). [[T106 Fading laser pointer]] leaves
the fade time, the look and the buffer size to the spec step; the owner
chose the key.

## Decision

- **Key** (owner): hold `Q`, the left hand, while the right hand moves
  the pen or mouse. Any `Q` without `Ctrl` or `Alt` counts (`Shift` may
  be held for something else); `Ctrl+Q` stays unbound. In text edit mode
  `Q` types `q`. `Q` is a held key like `Space`, not a keymap binding, so
  `keymap::BINDINGS` does not list it; the README and the cheat sheet do.
- **Fade**: each point fades linearly from full opacity to nothing over
  `laser::FADE_SECS` = 1 s after it was added, then is dropped.
- **Look**: a stroke `laser::WIDTH_PX` = 6 px wide in screen pixels (the
  same at every zoom), with round joints, in `theme::RED`. A piece
  between two points takes the opacity of the older point, so the tail
  fades first. A lone point shows as a dot.
- **Buffer**: at most `laser::MAX_POINTS` = 1024 points; the oldest go
  first. A pen at 200 Hz fills about 200 per second of fade.

## Alternatives considered

- **`L`** (the proposal) — the right hand would have to leave the mouse.
- **An eased fade** — linear is enough at 1 s and is simpler to test.
- **A colour of its own in `theme`** — `theme.rs` is not owned by this
  task; red is the obvious laser colour and is already in the palette.

## Consequences

- Changing the fade, width or colour is one constant each.

## Rollback plan

Change the constants; a different key is a new ADR.
