---
id: ADR-T87-1
title: Graph from an edge list key and layout rules
status: accepted
kind: decision
date: 2026-11-09
task: T87
builds_on: [ADR-T84-1, ADR-T11-1, ADR-T16-1, ADR-T19-1, ADR-T20-1, ADR-T31-1, ADR-T47-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T87-1 Graph from an edge list key and layout rules

## Context

A selected text block holding edges should become a drawn graph or tree in
one key ([[T87 Graph or tree from an edge list]]). New keys need an ADR
([[Keymap]]). Labels are `Option<u32>`, and on the task's base a label below
1 is refused by the snapshot parser and the fuzzer; the switch to zero-based
numbering lives in a separate task. Contest graphs are often 0-indexed.

## Decision

- **Key.** `Ctrl+L` is bound to `Command::GraphFromText`, next to
  `Ctrl+T` (text block to grid). `Ctrl+Shift+L` stays the diagnostics dump.
- **Grammar**, one pure parser `tools::layout::EdgeList::parse`, per line:
  - a line whose first token holds `-` or `>` is a list of pairs, `u-v` for
    an undirected edge and `u>v` for a directed one, every token a pair;
  - any other line is `u v` or `u v w`;
  - node names are ASCII digits fitting `u32`; a weight is a finite decimal
    number, kept as written, so `-5` and `1.5` are weights;
  - blank lines are ignored.
- **Refusals**, whole and never cut: a line that is none of these, more than
  200 nodes, or more than 64 KiB of text (`PASTE_MAX_BYTES`, as for
  [[ADR-T84-1 Grid from text keys and table rules]]). The reason goes to the
  log through `Editor::take_notice`. It names the line number, not its text,
  because the log does not record what was typed
  ([[ADR-T47-1 Flight recorder and capped session log]]).
- **Self-loops and duplicates.** A self-loop is skipped, but its node still
  exists. An edge repeating the same two nodes, direction and weight is drawn
  once; undirected `1-2` and `2-1` are the same edge. Edges that differ in
  direction or weight are all drawn.
- **Tree or circle.** After those are dropped, a connected graph with
  `n − 1` edges is a tree, whatever the arrows' direction. It is drawn in
  layers from the first node named, children in the order they were named,
  each leaf in its own column and each parent centred over its children.
  Anything else goes on a circle in the order named, starting at the top and
  going clockwise.
- **Sizes follow the block's text size.** Node diameter is 2.5 em. The
  distance between neighbours, the *pitch*, is one diameter plus the larger
  of one diameter and the widest weight plus 1 em, plus two line widths. So
  edges never invert and weights have room.
- **Spacing grows with the node count.** The circle's radius is
  `pitch / (2 sin(π / n))`, so neighbours on it are one pitch apart. Layers
  are `pitch × (1 + √n / 4)` apart, so edges of wide trees are not flat.
- **Edges end on the outlines**, computed directly: each end is
  `radius + line width` from its node's centre on the line between the two
  centres, the same offset outline snapping gives
  ([[ADR-T20-1 Outline snapping]]).
- **Weights** are text blocks at the block's size, placed beside the middle
  of their edge, the box touching the edge's stroke on the upper right side,
  so that no line runs through the digits.
- **Node 0.** A node named 0 is a circle holding the text `0`
  ([[ADR-T31-1 Shape text prototype behind a switch]]), any other a circle
  labelled with its number. Both look alike, and the shape stays valid
  with or without zero-based numbering.
- **Result.** Edges, then circles, then weights replace the block at its
  place in the z-order, in its colour and the current line width, as one
  undo step. The new shapes are selected, so they can be moved at once. The
  top-left of the nodes' box is the block's top-left.
- **Numbering.** Labels come from the text. The counter does not move, as
  for pasted shapes ([[ADR-T19-1 Numbering counter and undo]]).

## Alternatives considered

- **A weight centred on the edge's middle** — the line would strike through
  the digits, and shape text has no background to hide it.
- **Refuse node 0 until zero-based numbering is merged** — 0-indexed input
  is too common in contests to reject.
- **Change the label invariant here** — it belongs to the zero-based task
  and would touch the snapshot format twice.
- **Drop a repeated edge whatever its weight** — a parallel edge of another
  weight would vanish without a word.
- **Accept `1->2`** — not asked for; `-` and `>` already say it.
- **Force-directed layout** — out of scope for the task.

## Consequences

- A leading `n m` line is read as an edge, as the task states.
- Parallel edges of different weights overlap.
- Undoing a graph that holds the number just below the counter also gives
  that number back, as for a paste (and redoing one that holds the counter's
  own number takes it), a known limit of the label count of ADR-T19-1.

## Rollback plan

Remove `Command::GraphFromText`, its binding and `tools::layout` with
`revert(T87)` commits and a rollback ADR. No stored format changes.
