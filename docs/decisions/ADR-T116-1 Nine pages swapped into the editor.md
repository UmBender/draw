---
id: ADR-T116-1
title: Nine pages swapped into the editor
status: accepted
kind: decision
date: 2026-11-11
task: T116
builds_on: []
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T116-1 Nine pages swapped into the editor

## Context

The editor holds one document, history, camera and selection. Pages need
each of those per page, while the tools, the clipboard, style and
helpers stay editor-wide. Every tool and command already works on
`doc`, `history`, `camera` and `selection`.

## Decision

- The current page's document, history, camera, selection and
  auto-number counter stay in the editor's existing fields, so no tool
  or command changes. The other eight pages wait in `pages`, a table of
  nine slots; the slot of the current page is empty.
- `Command::GoToPage(n)` (`n` in `0..9`, shown as 1…9) is bound to
  `Alt+1`…`Alt+9`. It cancels the gesture and closes an open text edit
  as `Esc` does (a commit, not an undo step), then swaps the current
  page's state with slot `n`. An empty slot is a fresh page, so pages
  are created lazily. Going to the current page does nothing.
- A switch bumps the document revision and the base revision, so the
  shell redraws its cached document layer once (ADR-T24-5). It records
  no history entry on either page.
- The clipboard, tool, style, smoothing and helper switches are shared;
  the number counter is per page, so numbering restarts on a new page.
- The page number shows top-right, small, only when the page is not 1
  (owner decision), in the toolbar's text font.
- Snapshots: `VERSION` 3 adds a `[pages]` section (`current`, `stored`)
  and one `[page]` section per non-empty stored page (index, camera,
  number, selection, then `[document]` and `[history]`). The `[editor]`
  sections keep describing the current page. Version 2 still loads, as
  one page. Recovery restores the page that was open.

## Alternatives considered

- **A `Vec<Page>` and indexing through `pages[current]` everywhere** —
  touches every tool and command, large risk for no gain.
- **Shared history with per-page entries** — owner chose a history per
  page; entries would need a page tag and undo would skip around.

## Consequences

Switching moves five values, no copy of shapes. Memory grows with the
pages used. Old v2 snapshots and regression fixtures keep loading.

## Rollback plan

Revert the T116 commits and add a rollback ADR; v3 snapshots with
extra pages would no longer load.
