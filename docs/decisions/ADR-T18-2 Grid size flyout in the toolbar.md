---
id: ADR-T18-2
title: Grid size flyout in the toolbar
status: accepted
kind: decision
date: 2026-10-01
task: T18
builds_on: [ADR-T16-2, ADR-T16-3, ADR-T18-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T18-2 Grid size flyout in the toolbar

## Context

The grid's columns and rows were only visible in the preview of a drag and
only changeable with the arrow keys. The owner asked for a toolbar counter
for both. The toolbar is a single 40 px column that already ends at 766 px
in the default 1280 × 800 window; two more full buttons (68 px) would not
fit.

## Decision

- While the grid tool is the active tool and the toolbar is visible, a
  **flyout** panel is drawn to the right of the strip, its top aligned with
  the `G` button. It has two rows, `cols` and `rows`, each
  `label  [-]  value  [+]`.
- The `-`/`+` buttons are toolbar buttons (`ButtonKind::GridCols(±1)`,
  `ButtonKind::GridRows(±1)`) that run the existing `Command::GridCols` /
  `Command::GridRows`; clamping and redraw stay in the editor (T16).
- `toolbar::route` takes whether the flyout is shown. A press inside the
  flyout panel never reaches the canvas: on a button it applies the
  command, elsewhere it is swallowed — like the strip.
- Labels are ASCII (`cols`, `rows`, `-`, `+`): macroquad's default font
  has no arrow glyphs.

## Alternatives considered

- **Counters in the helper group** — no vertical room; a split 32 px cell
  (top half columns, bottom half rows, right click = −1) is hard to discover.
- **Display-only cell** — does not let the mouse change the values.
- **Always-visible flyout** — covers canvas for every tool; the values
  matter only for the grid tool.

## Consequences

- The flyout covers a small canvas area (about 150 × 60 px) at the left
  edge while the grid tool is active; a drag cannot start there.
- `shell::app::dispatch` passes `editor.tool() == Tool::Grid`.

## Rollback plan

Revert the T18 toolbar commits: drop the flyout functions and the two
`ButtonKind` variants, restore `route`'s signature and the `dispatch` call.
