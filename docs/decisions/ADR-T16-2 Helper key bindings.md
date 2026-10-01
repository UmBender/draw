---
id: ADR-T16-2
title: Helper key bindings
status: accepted
kind: decision
date: 2026-10-01
task: T16
builds_on: [ADR-T11-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T16-2 Helper key bindings

## Context

The helper features (grid tool, smart snap, grid snap, numbering) need keys.
[[Keymap]] already uses most mnemonic letters; `Shift` is also the drawing
constraint modifier. The grid tool needs to change its column/row counts
while the mouse button is held.

## Decision

New rows in `keymap::BINDINGS` (exact modifier matching,
[[ADR-T11-1 Exact modifier matching]]):

| Chord | Command |
|-------|---------|
| `G` | `SetTool(Tool::Grid)` |
| `M` | `ToggleSmartSnap` (m for *magnet*) |
| `Shift+G` | `ToggleGridSnap` |
| `N` | `ToggleNumbering` |
| `Shift+N` | `ResetNumbering` |
| `→` / `←` | `GridCols(+1)` / `GridCols(-1)` |
| `↓` / `↑` | `GridRows(+1)` / `GridRows(-1)` |

These commands never cancel the gesture in progress, so arrow keys resize a
grid live during the drag; the dimensions persist for the next grid and are
clamped to `1..=GRID_MAX_CELLS`. `KeyChord::shift` is added for the first
`Shift`-only bindings.

## Alternatives considered

- **Number keys for grid size** — `1`–`6` are colours.
- **`S` for snap** — taken by anti-tremor.
- **Arrow keys only while the grid tool is active** — would need tool-aware
  key routing; harmless global meaning is simpler and lets users preset the
  size before drawing.

## Consequences

- `Shift+G`/`Shift+N` fire only with exactly `Shift` held; holding `Shift`
  for a square while tapping `G` does nothing (as before for every tool key).
- Arrow keys are unavailable for nudging selections; a future nudge feature
  needs a new ADR.

## Rollback plan

Remove or change rows in `BINDINGS` and the `documented()` table in
`keymap::tests`, update [[Keymap]], and add a superseding ADR.
