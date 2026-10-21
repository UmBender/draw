---
id: ADR-T38-1
title: Left-hand keys for pen, line and smart snap
status: accepted
kind: decision
date: 2026-10-21
task: T38
builds_on: ["[[ADR-T11-1 Exact modifier matching]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T38-1 Left-hand keys for pen, line and smart snap

## Context

While drawing, the right hand holds the mouse or the pen and the left hand
rests on the left half of the keyboard. Pen (`P`), line (`L`) and smart
snap (`M`) are under the right hand, so switching to them means putting
the pen down or crossing the left hand over. The free bare letters on the
left (QWERTY / ABNT2) are `Q`, `W`, `D`, `X` and `Z`.

## Decision

- Bare `D` selects the pen (draw; home row, middle finger), bare `W` the
  line tool (next to `E` / `R`, near `A` for arrows), bare `X` toggles
  smart snap.
- `P`, `L` and `M` are freed, not kept as aliases: one key per command,
  as in the rest of the table, and the docs and toolbar show one key.
- `Ctrl+D` (duplicate) and `Ctrl+X` (cut) are unchanged; modifiers still
  match exactly (ADR-T11-1), so the bare keys do not clash with them.
- Hand (`H`) and numbering (`N`, `Shift+N`) stay: `Space` + drag already
  pans, and the owner did not take the proposed moves.
- The toolbar shows the new keys.

## Alternatives considered

- **Keep `P` / `L` / `M` as aliases** — eases the switch but leaves two
  keys per command in the docs and toolbar; the owner asked to change
  them.
- **Pen `Q`, line `W`** — `Q` is a pinky stretch; the pen is the most used
  tool and gets the home row.
- **Move hand and numbering too (`Q`, `Z`)** — offered; not taken.

## Consequences

Users of earlier releases must relearn three keys; the CHANGELOG says so.
`P`, `L`, `M` are free for later bindings. Tutorials of earlier steps keep
the keys of their time.

## Rollback plan

Restore the three rows of `BINDINGS`, the toolbar labels and the docs
with a `revert(T38)` commit and a rollback ADR.
