---
id: T98
title: Math names and weights in graphs
status: in-progress
wave: 56
branch: task/T98-math-graph-labels
depends_on: [T97]
adrs: [ADR-T98-1]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T98 Math names and weights in graphs

## Goal

`Ctrl+L` draws a graph whose node names or edge weights are `$…$` formulas, such as `$s$ $t$ $\infty$` or `$v_1$ > $v_2$`, with each formula typeset inside its circle or beside its edge.

## Known facts (2026-11-10)

- [[ADR-T87-1 Graph from an edge list key and layout rules]] limits node names to ASCII digits fitting `u32`, and weights to finite numbers. `$x$ 2` is refused by a test.
- A circle can already show text instead of a number, `$…$` included, fitted into its inscribed rectangle and shrunk when it does not fit ([[ADR-T52-1 Text in shapes always on, numbered shapes hold no text]]).
- Weights are already drawn as ordinary text blocks, which typeset formulas.
- Lines are split on whitespace, and `-` and `>` are separators ([[ADR-T97-1 Spaced edge pairs]]). A formula may hold both, and spaces.
- Count lines and parent lists compare node numbers ([[ADR-T96-1 Keywords for graph input]]).

## Spec

Rules: [[ADR-T98-1 Math names and weights in graphs]]. Tests live in `src/core/tools/layout.rs` and `src/core/editor.rs`.

- **AC-1** — A `$…$` token is one node name in `u v`, `u v w` lines and in pairs, spaces, `-` and `>` inside it included. Number and math names mix in one graph, and `1` and `$1$` are different nodes.
  Tests: `parse_reads_math_node_names`, `parse_keeps_spaces_and_separators_inside_math`, `parse_reads_math_names_in_pairs`, `parse_mixes_number_and_math_names`.
- **AC-2** — A weight may be a `$…$` formula as well as a finite number.
  Test: `parse_reads_math_weights`.
- **AC-3** — An unclosed `$`, an empty formula `$$`, a token mixing math and plain text (`$a$b`) and any other non-number name are refused, naming the line.
  Test: `parse_refuses_bad_math`.
- **AC-4** — A math node is a circle holding its formula as text, never a label. Every circle is wide enough for the widest name to be typeset at full size, so long names keep the nodes apart.
  Tests: `shapes_write_math_names_inside_circles`, `circles_grow_to_fit_the_widest_math_name`.
- **AC-5** — Count lines and parent lists stay numeric: math names never count as the largest node of an `n m` line, and a math entry in a parent list is refused.
  Tests: `parse_count_line_ignores_math_names`, `parse_parents_refuses_math_entries`.
- **AC-6** — `Ctrl+L` on a math list is one undo step that draws text-filled circles and a formula weight.
  Test: `graph_from_text_reads_math_names_and_weights`.

The fuzzer's edge lines include math names and weights (`arb_edge_text` in `tests/fuzz.rs`).

## Out of scope

- Plain-word names such as `a` or `src`, and plain-word weights such as `inf`: they would make typos read as edges.
- Math entries in parent lists.
- A circle size per node: all circles share the largest size.

## Files owned

- `src/core/tools/layout.rs`, `src/core/editor.rs`
- `docs/features/Numbered nodes.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T98-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T98): specify math names and weights in graphs`
- [ ] tests — `test(T98): add failing tests for math names and weights`
- [ ] models — `feat(T98): add math node names`
- [ ] behaviour — `feat(T98): read and draw math names and weights`
- [ ] quality — `chore(T98): pass clippy and rustfmt`
- [ ] docs — `docs(T98): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written from the owner's question why graphs refuse LaTeX nodes and edges. The limit was a T87 scope choice, not a rendering one. Scope agreed: math names and weights in edge lines and pairs; parent lists and count lines stay numeric.
