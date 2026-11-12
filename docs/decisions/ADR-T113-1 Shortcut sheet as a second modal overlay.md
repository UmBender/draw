---
id: ADR-T113-1
title: Shortcut sheet as a second modal overlay
status: accepted
kind: decision
date: 2026-11-11
task: T113
builds_on: ["[[ADR-T111-1 Command palette as a modal editor state]]"]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T113-1 Shortcut sheet as a second modal overlay

## Context

`?` opens a paged cheat sheet over the canvas. [[ADR-T111-1 Command palette as a modal editor state]]
suggested reusing the palette's plumbing.

## Decision

- The editor holds `sheet: Option<Sheet>` next to `palette`. They are never
  both `Some`: each opens only when the other is closed. Like the palette,
  the sheet is not in snapshots or the history, and while open `apply` and
  every pointer, scroll and paste event do nothing.
  (An enum of overlays was not needed: no code asks "which overlay".)
- `?` arrives as `InputEvent::Char('?')` (there is no `Slash` key). It opens
  the sheet when no text block is open and no overlay is open; in a text
  block it types a `?`. While the sheet is open `?` or `Esc` closes it,
  `←`/`→` flip pages (wrapping), `1`–`9` jump to the first page of that topic.
  Opening cancels a gesture in progress, like the palette.
- Topics (`keymap::Topic`): Tools, Style, Edit, View, Helpers, Gestures,
  Typing in a text block, Typing in a shape: the order of [[Contest cheat sheet]].
  `keymap::topic_of(Command)` is an exhaustive `match`, so a new command
  cannot be added without choosing its topic (no table to drift). `BINDINGS`
  keeps its 3-tuple shape (the palette and tests use it).
- Rows that are not bindings (gestures, typing) are one `const` table in
  `core::sheet`.
- Pages: `sheet::pages()` lists the topic pages; a topic with more than
  `ROWS_PER_PAGE` rows is split evenly ("Edit 1/2"). `ROWS_PER_PAGE` is 14:
  at the default window (1280×800) the header, tab line, 14 rows of 30 px
  and the footer take about 580 px.
- Drawing: the shell paints a dimming layer and a panel from the editor's
  sheet state, only while open; no frames are requested.
- New files `src/core/sheet.rs` and `src/shell/sheet.rs`; `src/core/mod.rs`
  and `src/shell/mod.rs` are added to the files owned
  (`UNATTENDED CHOICE:`).

## Alternatives considered

- **Add a topic field to every `BINDINGS` entry** — touches ~100 entries and
  every user of the tuple; the exhaustive match gives the same guarantee.
- **One overlay enum** — more churn in the palette code for no use.

## Consequences

Pure, fuzzable paging logic; the sheet always lists the real keys.

## Rollback plan

Remove `sheet.rs`, the editor field and the shell overlay.
