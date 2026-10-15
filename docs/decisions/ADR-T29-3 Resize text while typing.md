---
id: ADR-T29-3
title: Resize text while typing
status: accepted
kind: decision
date: 2026-10-14
task: T29
builds_on: [ADR-T29-1]
amends: [ADR-T29-2]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T29-3 Resize text while typing

## Context

[[ADR-T29-2 Text edit mode]] takes a new block's size from the stroke
width (`[` / `]`) when it is placed. In edit mode `[` and `]` type
brackets, so a block's size cannot change once it is open — the owner
asked for an easy way to grow the letters while typing.

## Decision

- In edit mode, `Ctrl+]` / `Ctrl+[` (exactly `Ctrl`, ADR-T11-1) step the
  open block's font size up / down through the on-screen text ladder
  `text_size_px(w)` for every `w` of `WIDTH_LADDER_PX` (14 … 76 px), at the
  current zoom. A size between two rungs goes to the next rung in that
  direction; at either end the key does nothing.
- The new size is part of the edit: it is committed with the text as the
  same single undo step (a size-only change of an existing block is a
  `Replace`) and thrown away by `Ctrl+Z`.
- The stroke width setting is not changed.

## Alternatives considered

- **Bare `+` / `-`** — they must type their characters.
- **Changing the stroke width too** — couples two settings the user did
  not ask to couple; the next block still follows `[` / `]`.
- **Free scaling (e.g. ×1.1 per press)** — sizes would drift off the
  ladder new blocks use.

## Consequences

`Ctrl+[` produces the control character ESC on X11; control characters
are never inserted (ADR-T29-2), so only the size changes.

## Rollback plan

Remove the two key arms and `TextEdit::step_size` with a `revert(T29)`
commit and a rollback ADR.
