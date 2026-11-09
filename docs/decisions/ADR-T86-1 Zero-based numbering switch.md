---
id: ADR-T86-1
title: Zero-based numbering switch
status: accepted
kind: decision
date: 2026-11-09
task: T86
builds_on: ["[[ADR-T16-2 Helper key bindings]]", "[[ADR-T16-3 Helper settings and hooks]]", "[[ADR-T19-1 Numbering counter and undo]]"]
amends: ["[[ADR-T52-1 Text in shapes always on, numbered shapes hold no text]]", "[[ADR-T74-1 Line-based editor snapshot format]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T86-1 Zero-based numbering switch

## Context

Contest code is often 0-indexed, but node numbers always start at 1
([[T86 Zero-based numbering]]). The first number is a constant that
`roll_back`, `Shift+N` and the snapshot checks all rely on. Only 1 and 0
are wanted as first numbers. `Ctrl+N` is free.

## Decision

- **Setting.** `Helpers` gets a `zero_based: bool`, off by default;
  `Helpers::first_number()` gives 0 or `FIRST_NUMBER`. A bool rather than
  a stored number, so no other start value can exist.
- **Key.** `Ctrl+N` (`Command::ToggleZeroBased`) flips the switch and
  restarts the counter at the new first number. Like the other helper
  commands it does not cancel a gesture and is not an undo step. `Shift+N`
  restarts at the current first number.
- **Undo.** `numbering::roll_back` takes the first number as its floor.
  Labels are never rewritten: after switching, shapes keep the numbers
  they already have.
- **Labels.** A label is any `u32`, 0 included, whatever the current
  setting: switching back to 1 leaves nodes labelled 0 in the document.
  The snapshot reader stops rejecting label 0.
- **Counter check.** Reading a snapshot checks
  `next_number >= first_number()`.
- **Snapshots.** The seventh `helpers` token, reserved since ADR-T52-1,
  now holds `zero_based`. Every text written so far has `0` there, which
  still means the default, so `VERSION` stays `1` and every fixture stays
  byte-identical. This is the one exception to the *Versioning* rule of
  ADR-T74-1: a reserved token taking a meaning that its old value keeps.

## Alternatives considered

- **`first_number: u32` in `Helpers`** — allows values the feature does
  not have, and every reader must validate them.
- **A new token and a `VERSION` bump** — regenerates every fixture and
  e2e expectation for a slot that is already free.
- **Renumber existing labels when switching** — labels are not editable
  (out of scope), and it would change the document without an undo step.

## Consequences

- A snapshot with the seventh token at `1` now loads as zero-based; no
  writer produced that before this ADR.
- The reserved token is used up: the next new helper needs a `VERSION`
  bump.
- The shell has no toolbar button for the switch; it is keyboard only.

## Rollback plan

A rollback ADR and `revert(T86)` commits remove `Command::ToggleZeroBased`
and `Helpers::zero_based`, and turn the token back to reserved; the
snapshot reader keeps ignoring it.
