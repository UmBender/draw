---
id: T121
title: Self-loops and opposite edges in graphs
status: done
wave: 60
branch: fix/graph-loops-and-lanes
depends_on: []
adrs: ["[[ADR-T121-1 Self-loops and lanes for edges between the same nodes]]"]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task, bug]
---

# T121 Self-loops and opposite edges in graphs

## Goal

A graph input draws every edge it lists: a self-loop (`1 1 12`) appears as a loop on its node, and edges between the same two nodes (`1 3 4` with `3 1 2`) are drawn side by side with their weights apart.

## Known facts (2026-11-11)

- Reported by the owner with a `dgraph:` input: `1 1 12` drew nothing, and `1 3 4` with `3 1 2` drew one line with the weights `4` and `2` on top of each other.
- `EdgeList::from_raws` drops every `from == to` edge ([[ADR-T87-1 Graph from an edge list key and layout rules]], AC-6 of T87).
- Edges between the same two nodes were all drawn on the same line, and `weight_text` puts every weight on the same side of that line.

## Spec

- **AC-1** — A self-loop is kept in a new list `EdgeList::loops` (never in `edges`, so the layout and the tree test do not change). Repeats with the same direction and weight are still dropped.
  Tests: `parse_keeps_a_self_loop_apart_from_the_edges`, `parse_draws_a_repeated_self_loop_once`.
- **AC-2** — A loop is drawn outside its node, on the side with the most room between the other edges of that node (up if it has none), as a stroke that starts and ends on the circle; an arrow head ends it when the loop is directed. Its weight sits beyond the far side of the loop.
  Tests: `loop_is_drawn_outside_its_node`, `loop_leaves_the_widest_gap_between_edges`, `directed_loop_ends_in_an_arrow`, `loop_weight_is_beyond_the_loop`, `loops_of_one_node_do_not_overlap` (they take the next widest gap).
- **AC-3** — Edges between the same two nodes are drawn on parallel lanes, centred on the line between the nodes; each weight is on the outer side of its lane. A lone edge is drawn exactly as before.
  Tests: `opposite_edges_are_drawn_on_two_lanes`, `weights_of_opposite_edges_do_not_overlap`, `lone_edge_is_not_moved`, `lanes_end_on_the_circles`.
- **AC-4** — No panic and every shape finite, whatever the input.
  Test: `loops_and_lanes_layout_never_panics_and_stays_finite` (proptest).
- **AC-5** — The owner's case: after `Ctrl+L` on the saved scene there is a loop, seven weights and none overlaps another or a node.
  Test: `regression_T121_dgraph_loops_and_opposite_edges` (snapshot `tests/snapshots/regressions/T121-dgraph-loops-and-opposite-edges.snap`, built by `regression_fixture_T121_is_the_built_scene`).

## Out of scope

- Curved arcs for lanes; labels overlapping edges of other node pairs; the layouts themselves.

## Files owned

- `src/core/tools/layout.rs`
- `tests/snapshots.rs`, `tests/snapshots/regressions/`
- `docs/features/Numbered nodes.md`
- this note, and new `ADR-T121-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T121): specify self-loops and lanes`
- [x] regression — `test(T121): add regression test for loops and opposite edges`
- [x] tests — `test(T121): add failing tests for loops and lanes`
- [x] behaviour — `fix(T121): draw self-loops and lanes`
- [x] quality — `chore(T121): pass clippy and rustfmt`
- [x] docs — `docs(T121): update the feature note`

## Log

- 2026-11-11 — written from the owner's bug report.
- 2026-11-11 — fixed; `scripts/check.sh --fuzz` green. Loops of one node take successive widest gaps instead of nesting (nested weights overlapped). The old editor test that expected a loop to be skipped was updated (ADR-T121-1 amends it).
