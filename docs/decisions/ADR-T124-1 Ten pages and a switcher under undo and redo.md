---
id: ADR-T124-1
title: Ten pages and a switcher under undo and redo
status: accepted
kind: decision
date: 2026-11-12
task: T124
builds_on: ["[[ADR-T116-1 Nine pages swapped into the editor]]", "[[ADR-T123-1 Menus use the bundled font on whole pixels]]"]
amends: ["[[ADR-T116-1 Nine pages swapped into the editor]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T124-1 Ten pages and a switcher under undo and redo

## Context

[[ADR-T116-1 Nine pages swapped into the editor]] gave nine pages, reached by `Alt+1`…`Alt+9`, and showed the page number top-right when it was not page 1. The owner wants a visible switcher like the workspaces of a status bar, numbered 1 to 10, and the number that appears on a switch removed.

## Decision

- **Ten pages.** `PAGE_COUNT` is 10 and `Alt+0` goes to page 10. The snapshot and file formats are unchanged: a page is read when its index is below `PAGE_COUNT`, so files written with nine pages or fewer still load. Version 1.6.0 is not released, so no shipped build has to read ten pages.
- **Switcher.** A grid of two columns by five rows of half-width cells, numbers 1…10 row by row, in the toolbar strip under the history buttons after a group gap. Cells that would cross the bottom of the window are left out (the strip is 40 px wide; a 1280×800 window cannot hold them, a 1080 px one can).
- **Look.** Current page: filled in the accent colour. A page with shapes: the toolbar text colour and a bar under the number. An empty page: dimmed text.
- **Click.** A press on a cell runs `GoToPage`, as the key does, and never reaches the canvas.
- **Indicator removed.** The top-right number of ADR-T116-1 is deleted; the switcher replaces it. With the toolbar hidden (`Tab`) no page is shown.

## Alternatives considered

- **A horizontal bar on top of the canvas** — closer to a status bar, but a new overlay that covers drawing space.
- **Single column of ten buttons** — does not fit any common window.
- **Keep nine pages** — the owner asked for ten.

## Consequences

- The toolbar shows which pages hold drawings.
- On short windows the lower cells are missing; the keys still work.

## Rollback plan

Set `PAGE_COUNT` back to 9, drop `Alt+0` and the cells, and restore `draw_page_label`; this ADR is then reverted by a new ADR.
