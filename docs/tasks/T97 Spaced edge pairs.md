---
id: T97
title: Spaced edge pairs
status: in-progress
wave: 55
branch: task/T97-spaced-edge-pairs
depends_on: [T96]
adrs: [ADR-T97-1]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T97 Spaced edge pairs

## Goal

An edge written with spaces around its separator, `1 > 3` or `1 - 3`, or with a space on one side, `1> 3` or `1 >3`, is read by `Ctrl+L` as the pair `1>3` or `1-3`.

## Known facts (2026-11-10)

- [[T87 Graph or tree from an edge list]] reads a pair only as one token, `u-v` or `u>v`. `1 > 3` splits into three tokens, is read as `u v w` with `>` as `v`, and is refused.
- A weight is a finite decimal kept as written, so `-5` in `1 2 -5` is a weight, never a separator ([[ADR-T87-1 Graph from an edge list key and layout rules]]).
- [[T96 Keywords for graph input]] is not merged yet. This task builds on its branch because both change `EdgeList::parse`.

## Spec

Rules: [[ADR-T97-1 Spaced edge pairs]]. Tests live in `src/core/tools/layout.rs`.

- **AC-1** — `u > v` and `u - v`, with spaces on both sides, are the pairs `u>v` and `u-v`.
  Tests: `parse_reads_a_spaced_directed_pair`, `parse_reads_a_spaced_undirected_pair`.
- **AC-2** — A separator touching only one of its nodes, `u> v`, `u >v` or `u- v`, joins them too; `u -v` does not, since `-v` reads as a sign. Several spaced pairs fit on one line.
  Tests: `parse_reads_separators_touching_one_node`, `parse_reads_several_spaced_pairs_on_a_line`.
- **AC-3** — Lines with no spaced separator read as before: `1 2 -5` keeps the weight `-5`, `1-2 2>3` stays two pairs.
  Tests: `parse_keeps_a_negative_weight_apart_from_separators`, `parse_reads_directed_pairs`, `parse_reads_plain_lines_and_weights`.
- **AC-4** — A separator with a missing node (`1 >`, `> 2`) or a joined line holding a token that is not a pair (`5 1 > 3`) refuses the text and names the line.
  Test: `parse_refuses_a_dangling_separator`.
- **AC-5** — Under `dgraph:`, a spaced `u - v` stays undirected, as a written pair does.
  Test: `parse_dgraph_keyword_keeps_spaced_pairs`.

The fuzzer's edge lines include spaced pairs (`arb_edge_text` in `tests/fuzz.rs`).

## Out of scope

- Weights on pairs, spaced or not (`1 > 3 7`).
- Other separators such as `->` or `--`.

## Files owned

- `src/core/tools/layout.rs`
- `docs/features/Numbered nodes.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T97-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T97): specify spaced edge pairs`
- [ ] tests — `test(T97): add failing tests for spaced edge pairs`
- [ ] models — `feat(T97): …`
- [ ] behaviour — `feat(T97): join spaced separators into pairs`
- [ ] quality — `chore(T97): pass clippy and rustfmt`
- [ ] docs — `docs(T97): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written from the owner's request that `1 > 3` with spaces draw an arrow, as `1>3` does.
