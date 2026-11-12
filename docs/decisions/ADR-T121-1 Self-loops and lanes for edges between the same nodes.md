---
id: ADR-T121-1
title: Self-loops and lanes for edges between the same nodes
status: accepted
kind: decision
date: 2026-11-11
task: T121
builds_on: ["[[ADR-T87-1 Graph from an edge list key and layout rules]]", "[[ADR-T119-1 Forests drawn in layers]]"]
amends: ["[[ADR-T87-1 Graph from an edge list key and layout rules]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T121-1 Self-loops and lanes for edges between the same nodes

## Context

[[ADR-T87-1 Graph from an edge list key and layout rules]] skips a self-loop, and draws every edge as a straight line between its nodes. The owner reported two inputs that therefore read wrongly (2026-11-11): `dgraph:` with `1 1 12` shows no loop, and `1 3 4` together with `3 1 2` puts two arrows and two weights on one line.

## Decision

- A self-loop is **kept**, in `EdgeList::loops`, apart from `edges`: it is no edge for the tree test or the layers, so no layout changes. It is drawn for `graph:` and `dgraph:` alike (an undirected loop has no arrow head). A repeat with the same direction and weight is dropped, as before; a node named only by a loop still exists.
- A loop is a stroke on a circle of `0.6` node radii whose centre is `0.4` loop radii beyond the node's outline, so it starts and ends on that outline. It points into the widest angular gap between the other edges of its node and the loops already drawn on it, up if there is none. A directed loop ends in an arrow head (its last four of 24 segments).
- Its weight is placed beyond the far side of the loop, in line with the direction it points.
- Edges between the same two nodes (either direction) share the line as **lanes**: `k` lanes are spaced by `1.6 r / max(k − 1, 2)`, centred on the line, each parallel to it, in the order the edges were written. A lone edge has one lane at offset `0` and is drawn exactly as before.
- On a lane off the centre the weight is on the outer side of its lane; the centre lane keeps the upper-right rule of [[ADR-T87-1 Graph from an edge list key and layout rules]] and T120.

## Alternatives considered

- **Curved arcs for opposite edges** — the usual drawing, but there is no arc shape; a polyline per edge costs more and its arrow head needs a tangent.
- **Keep loops in `edges`** — the tree test, the layers and the circle would all have to learn to skip them.
- **Loops always on top** — collides with a parent above or any edge pointing up.

## Consequences

- A graph shows every edge it lists; a loop adds a stroke, and a weight beyond it.
- Lanes make the two weights of opposite edges readable.

## Rollback plan

Drop `EdgeList::loops` and the lane offsets in `EdgeList::shapes`; this ADR is then reverted by a new ADR.
