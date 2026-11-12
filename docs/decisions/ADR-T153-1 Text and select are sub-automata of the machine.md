---
id: ADR-T153-1
title: Text and select are sub-automata of the machine
status: accepted
kind: decision
date: 2026-11-12
task: T153
builds_on: [ADR-T134-1, ADR-T136-1]
amends: [ADR-T136-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T153-1 Text and select are sub-automata of the machine

## Context

[[ADR-T136-1 One gesture variant per tool in the machine]] named one gesture
variant per tool but left the states a tool keeps inside itself in
`core/tools/**` (UNATTENDED CHOICE: "the tool sub-automata are not rewritten
now"). The owner reviewed that choice and asked for the rewrite where it
pays: the goal of [[T136 Automaton gestures and text editing per tool]] is that
a half-finished gesture cannot meet a command that does not expect it.

Reading the tools: pen, line, arrow, rectangle, ellipse, grid, bucket and
eraser are a press, a drag and a release; `Stage` already covers them. The
text tool is a struct of several `Option`s and flags (`target`, `host`,
`resized`), so combinations the code never means can be built. Select
already has a private `enum Gesture` (marquee, move, resize) in `select.rs`.

## Decision

- The sub-states of **text** and **select** become enums in `machine/`, with
  one variant per legal state and the data only that state needs. A
  combination that is not a variant cannot be built.
- Every transition is a `match` on the state and the input that lists all
  variants, with no wildcard arm (the T131 lint).
- The other tools keep `Stage` and their small `Drag` or `Mode` types; they are
  not rewritten.
- Dispatch is by `enum`, never `dyn`. The pointer path allocates nothing it
  did not before.
- The snapshot stays readable: a record of an older version is read and
  mapped to a state. If a written record has to change, the format version
  goes up with a migration, decided in the task spec.
- The model test of T138 grows a text and a select part, so a sequence of inputs
  is checked against a reference for both.

## Alternatives considered

- **Keep ADR-T136-1 as it is** — leaves the unreachable-state risk in text
  and gives the task goal only by name.
- **Rewrite all eleven tools** — about 7 500 lines of tested code, most of it
  already safe by construction; the regression risk is not paid back.

## Consequences

Text and select edits cannot reach an undefined state, and the machine
documents them. The change touches the snapshot code and `editor.rs`, so
the e2e and fuzz suites must stay green.

## Rollback plan

`git revert` the T153 behaviour commit; the tools return to
[[ADR-T136-1 One gesture variant per tool in the machine]].
