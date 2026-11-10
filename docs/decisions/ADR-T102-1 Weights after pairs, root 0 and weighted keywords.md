---
id: ADR-T102-1
title: Weights after pairs, root 0 and weighted keywords
status: accepted
kind: decision
date: 2026-11-10
task: T102
builds_on: [ADR-T96-1, ADR-T97-1, ADR-T98-1]
amends: [ADR-T96-1, ADR-T97-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T102-1 Weights after pairs, root 0 and weighted keywords

## Context

[[ADR-T97-1 Spaced edge pairs]] refuses any token in a line of pairs that is not a pair, so a weighted pair such as `1>3 7` or `1 > 3 7` cannot be written. [[ADR-T96-1 Keywords for graph input]] marks the root of a full parent list with `-1` only, while many problems write `0` for "no parent" with nodes numbered from 1. Problem statements also say "weighted tree" or "weighted graph", and `wtree:` is a natural first line to type.

## Decision

- **Weights after pairs.** In a line read as a list of pairs (after the joining of ADR-T97-1), a token that is not a pair but is a weight, a finite number or one `$…$` formula as in [[ADR-T98-1 Math names and weights in graphs]], is the weight of the pair right before it. `1>3 7`, `1 > 3 $w$` and `1-2 5 2-3 9` are read. A weight first on the line, or a weight right after another weight, refuses the line.
- **Pair first.** A token is tried as a pair before it is tried as a weight. A number such as `-5` or `1e-5` is never a pair (its left side is not a node name), so it is a weight. The sign rule of ADR-T97-1 is kept: `-5` is not joined to the pair before it.
- **Root `0` in `parents:`.** In a `parents:` list an entry whose number is 0 (`0`, `00`, …) marks the root exactly as `-1` does: the list then has one entry per node, numbered from 1. Two root markers, any mix of `0` and `-1`, refuse the text and name the line of the second. In `parents0:` `0` stays node 0 and only `-1` marks the root.
- **Weighted keywords.** `wtree:` and `wgraph:` read as `tree:` and `graph:`, and `wdgraph:` as `dgraph:`. The `w` only documents the input; weights are read on every edge keyword as before.

## Alternatives considered

- **A weight token attaching to the pair after it** — `7 1>3` reads poorly and does not match how inputs write `u v w`.
- **Only the literal `0` as root** — `00` would then silently name node 0 under `parents:`, a node that list cannot have.
- **Making `wtree:` require weights** — refuses inputs that are fine, for no gain.

## Consequences

- `1 > 3 7`, refused under ADR-T97-1, now draws an edge weighted 7; the T97 test listing it as refused drops that one case.
- A `parents:` list that used `0` as a parent name (drawing a node 0) now reads `0` as its root marker.
- Weights on parent-list edges stay out of scope.

## Rollback plan

Revert the T102 commits and record it in a rollback ADR. Graphs already drawn stay as they are.
