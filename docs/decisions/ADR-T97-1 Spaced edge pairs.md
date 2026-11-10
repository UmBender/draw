---
id: ADR-T97-1
title: Spaced edge pairs
status: accepted
kind: decision
date: 2026-11-10
task: T97
builds_on: [ADR-T87-1, ADR-T96-1]
amends: [ADR-T87-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T97-1 Spaced edge pairs

## Context

[[ADR-T87-1 Graph from an edge list key and layout rules]] reads a pair only as one token, `u-v` or `u>v`. Typing `1 > 3` is natural but is refused, because the line splits into three tokens and `>` is not a node name. A `-` is also the sign of a negative weight, as in `1 2 -5`.

## Decision

- **Joining.** Before a line is read, its whitespace-separated tokens are joined across separators:
  - a token that ends with `-` or `>` (a lone `-` or `>` included) is joined to the token after it;
  - a token that starts with `>`, or is a lone `-`, is joined to the token before it.
- **Signs.** A token that starts with `-` but holds more, such as `-5`, is never joined to the token before it. `1 2 -5` stays a plain edge weighted `-5`.
- **Reading.** A line where any join happened is read as a list of pairs, every joined token a pair (`u-v` or `u>v`), as T87 reads a line whose first token holds a separator. A token that is not a pair refuses the line, so `1 >`, `> 2` and `5 1 > 3` name their line in the log.
- **Lines without joins** are read exactly as before.
- Pairs keep their written meaning under every keyword of [[ADR-T96-1 Keywords for graph input]]: a spaced `u - v` is undirected under `dgraph:`.

## Consequences

- `1 > 3`, `1> 3`, `1 >3` and `1 - 3` all draw the same edge as the unspaced pair.
- `1 -3` is not joined, since `-3` reads as a sign, and is refused as before.
- Pairs still carry no weight, spaced or not.
