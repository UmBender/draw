---
id: ADR-T125-1
title: Six pages, one button per row
status: accepted
kind: decision
date: 2026-11-12
task: T125
builds_on: ["[[ADR-T124-1 Ten pages and a switcher under undo and redo]]"]
amends: ["[[ADR-T124-1 Ten pages and a switcher under undo and redo]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T125-1 Six pages, one button per row

## Context

[[ADR-T124-1 Ten pages and a switcher under undo and redo]] gave ten pages and a switcher of small cells in two columns, which needs a window about 890 px tall. The owner asked for at most six pages, one per line like the palette colours.

## Decision

- `PAGE_COUNT` is 6. `Alt+1`…`Alt+6` go to the pages; `Alt+7`…`Alt+9` and `Alt+0` are unbound again. Files and snapshots are unchanged in format; one written with a page above 6 is refused like any page out of range (none was released).
- The switcher is one 32 px square button per page in a single column under the history buttons, after a group gap, with the same looks as ADR-T124-1 (current filled, with shapes in full colour and a bar, empty dimmed). Buttons that would cross the bottom of the window are left out; six need about 1000 px.

## Alternatives considered

- **Keep two columns** — the owner asked for one per line.

## Consequences

- The switcher matches the colour buttons; a window under about 1000 px shows only the pages that fit, and the keys still work.

## Rollback plan

Restore `PAGE_COUNT = 10`, the bindings and the two-column cells; a new ADR reverts this one.
