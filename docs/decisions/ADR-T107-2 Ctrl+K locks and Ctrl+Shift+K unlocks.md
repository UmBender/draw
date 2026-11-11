---
id: ADR-T107-2
title: Ctrl+K locks and Ctrl+Shift+K unlocks
status: accepted
kind: decision
date: 2026-11-10
task: T107
builds_on: [ADR-T107-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T107-2 Ctrl+K locks and Ctrl+Shift+K unlocks

## Context

Locking ([[ADR-T107-1 Lock flag kept by the document]]) needs two keys.
`Ctrl+K` and `Ctrl+Shift+K` are unbound, and `K` has no bare binding.

## Decision

- `Ctrl+K` → `Command::LockSelection`, "Lock selection": locks every
  selected shape and clears the selection, as one undo step; nothing
  selected records nothing.
- `Ctrl+Shift+K` → `Command::UnlockAll`, "Unlock every shape": one undo
  step; nothing locked records nothing.
- Both cancel the gesture in progress first, like the other selection
  commands, and both commit an open text block first, like every
  command.

## Alternatives considered

- **`Ctrl+L`** (common elsewhere) — already "text block to graph".
- **One toggle key** — cannot unlock: a locked shape cannot be selected.

## Consequences

- The keymap table, the cheat sheet and the README list both chords.
- Command bursts in the fuzz harness reach both keys through `BINDINGS`.
