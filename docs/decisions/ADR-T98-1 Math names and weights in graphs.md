---
id: ADR-T98-1
title: Math names and weights in graphs
status: accepted
kind: decision
date: 2026-11-10
task: T98
builds_on: [ADR-T87-1, ADR-T96-1, ADR-T97-1, ADR-T52-1]
amends: [ADR-T87-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T98-1 Math names and weights in graphs

## Context

[[ADR-T87-1 Graph from an edge list key and layout rules]] accepts only digit names and numeric weights. Contest notes often name nodes `$s$`, `$v_1$` or `$u'$` and weight edges `$\infty$` or `$c_i$`. Circles and text blocks already typeset formulas ([[ADR-T52-1 Text in shapes always on, numbered shapes hold no text]]).

## Decision

- **Tokens.** A line is split on whitespace outside `$…$`, so a formula with spaces stays one token. A line with an unclosed `$` is not an edge.
- **Names.** A node name is ASCII digits fitting `u32`, as before, or a math token: a token that starts and ends with `$`, holds no other `$` and is not blank inside. `1` and `$1$` are different nodes. Plain words stay refused, so a typo is still a bad line.
- **Separators.** A pair's separator is the first `-` or `>` outside `$…$`. [[ADR-T97-1 Spaced edge pairs]] joining is unchanged: a math token starts and ends with `$`, never with a separator.
- **Weights.** A weight is a finite number or a math token, kept as written.
- **Numbers only.** Count-line detection ([[ADR-T96-1 Keywords for graph input]]) compares only number names, and parent-list entries stay numbers.
- **Drawing.** A math node is a circle holding its formula as text, never a label, so it does not touch the numbering counter. Every circle of a graph shares one radius: the T87 radius, or the smallest one whose fitted text frame (inscribed square scaled by `SHAPE_TEXT_FILL`) holds the widest name's typeset box at full size, if that is larger. Spacing follows from the radius as before.

## Consequences

- `$s$ $t$ $\infty$`, `$a$ > $b$` and `1-$x$` draw as written.
- One long name enlarges every circle of its graph; per-node sizes would need a new layout and are left out.
- `EdgeList::names` returns `NodeName`s instead of `u32`s.
