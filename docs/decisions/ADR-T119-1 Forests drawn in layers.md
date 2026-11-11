---
id: ADR-T119-1
title: Forests drawn in layers
status: accepted
kind: decision
date: 2026-11-11
task: T119
builds_on: ["[[ADR-T87-1 Graph from an edge list key and layout rules]]", "[[ADR-T104-2 Segment tree nodes as boxes]]"]
amends: ["[[ADR-T87-1 Graph from an edge list key and layout rules]]"]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T119-1 Forests drawn in layers

## Context

[[ADR-T87-1 Graph from an edge list key and layout rules]] draws a graph in layers only when it is one tree (connected, `n − 1` edges); anything else goes on a circle. A problem's input is often a forest, e.g. `dgraph:` with two separate trees, and the owner reported that such input lands on a circle ("bug in the tree generation", 2026-11-11), where a node with children no longer reads as a tree at all.

## Decision

- A graph with no cycle is a **forest** and is drawn in layers. A cycle is `edges + components ≠ nodes` after self-loops and repeated edges are dropped; arrow directions still do not count, so `1>2 2>1` stays a cycle and goes on the circle.
- Every component is drawn as a tree from its first-named node, in the order the components were first named, side by side on the same top row, left to right.
- Components share the leaf-column counter, with half a column of extra space between one component and the next, so their circles never overlap and the groups can be told apart.
- A graph that is one tree is drawn exactly as before (same positions), and the segment tree drawing of [[ADR-T104-2 Segment tree nodes as boxes]], which calls the same layout, is unchanged.

## Alternatives considered

- **Keep the circle for forests** — the behaviour reported as a bug.
- **Stack components vertically** — wastes the width of a wide window and the layer axis would no longer mean depth.
- **A layout per component placed on a grid** — more code for no gain over one row.

## Consequences

- Every acyclic input is drawn as trees; only cycles (and opposite arrows) use the circle.
- A forest is wider than one tree with the same node count.

## Rollback plan

Restore the `edges + 1 == nodes` check in `EdgeList::layers` and the `forest_goes_on_a_circle` test; this ADR is then reverted by a new ADR.
