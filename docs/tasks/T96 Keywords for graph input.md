---
id: T96
title: Keywords for graph input
status: review
wave: 54
branch: task/T96-graph-keywords
depends_on: [T87]
adrs: [ADR-T96-1]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T96 Keywords for graph input

## Goal

A problem's graph input can be pasted as it is under one keyword line, `graph:`, `tree:`, `dgraph:`, `parents:` or `parents0:`, and `Ctrl+L` draws it, with no `>` to add and no count line to delete.

## Known facts (2026-11-09)

- [[T87 Graph or tree from an edge list]] reads `u v` lines as undirected edges, needs `u>v` for a directed one, and reads a leading `n m` line as an edge.
- Trees are often given as a parent list: `p2 … pn` with root 1, `p1 … p(n-1)` with root 0, or one entry per node with `-1` at the root.
- A tree is drawn in layers from the first node named (ADR-T87-1).

## Spec

Rules: [[ADR-T96-1 Keywords for graph input]]. Tests live in `src/core/tools/layout.rs` and `src/core/editor.rs`.

- **AC-1** — Without a keyword, text is read exactly as before. A first line that looks like a keyword but is not one is a bad line.
  Tests: `parse_without_a_keyword_keeps_the_t87_rules`, `parse_refuses_an_unknown_keyword`.
- **AC-2** — `graph:` and `tree:` read the lines after them as T87 edges.
  Tests: `parse_graph_keyword_reads_edges`, `parse_tree_keyword_reads_edges`.
- **AC-3** — `dgraph:` reads `u v` and `u v w` lines as directed edges from `u` to `v`, keeping the weight; `u-v` stays undirected.
  Tests: `parse_dgraph_keyword_makes_plain_edges_directed`, `parse_dgraph_keyword_keeps_written_pairs`.
- **AC-4** — After an edge keyword, a first line that is one number, or `n m` with `m` the number of lines after it and `n` at least every node name in them, is a count line and is skipped. Without a keyword nothing is skipped.
  Tests: `parse_skips_a_count_line_after_a_keyword`, `parse_keeps_a_first_edge_that_is_not_a_count_line`, `parse_never_skips_without_a_keyword`.
- **AC-5** — `parents:` reads `p2 … pn` with root 1 and `parents0:` reads `p1 … p(n-1)` with root 0. A list holding one `-1` is one entry per node, from 1 or from 0, with `-1` at the root. Entries may span lines, and a count line `n` is skipped when `n − 1` or `n` entries follow.
  Tests: `parse_parents_reads_p2_to_pn_with_root_1`, `parse_parents0_reads_p1_onwards_with_root_0`, `parse_parents_reads_a_full_list_with_minus_one_at_the_root`, `parse_parents_skips_a_count_line`, `parse_parents_reads_entries_across_lines`.
- **AC-6** — A parent list's tree is drawn from its root, and a bad entry, or more than one `-1`, refuses the text and names its line.
  Tests: `parse_parents_names_the_root_first`, `parents_tree_is_drawn_from_its_root`, `parse_parents_refuses_bad_entries`.
- **AC-7** — A keyword with nothing after it is no graph. `Ctrl+L` on a keyword list is one undo step.
  Tests: `parse_keyword_alone_is_no_graph`, `graph_from_text_reads_a_parent_list`, `graph_from_text_reads_a_directed_list`.

The fuzzer's edge texts include keyword lines (`arb_edge_text` in `tests/fuzz.rs`).

## Out of scope

- A root other than 1 or 0 for `p2 … pn` lists.
- Lists that mark the root with `0` instead of `-1`.
- Adjacency matrices and adjacency lists.
- Labels on graph nodes.

## Files owned

- `src/core/tools/layout.rs`, `src/core/editor.rs`
- `docs/features/Numbered nodes.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T96-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T96): specify keywords for graph input`
- [x] tests — `test(T96): add failing tests`
- [x] models — `feat(T96): add types`
- [x] behaviour — `feat(T96): implement`
- [x] quality — `chore(T96): pass clippy and rustfmt`
- [x] docs — `docs(T96): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's request: a first line such as `dgraph:` says how to read the input below it, so it can be pasted without adding `>`. Parent lists accept all the common forms. The count line is skipped automatically, the owner's goal being to paste input unchanged.
- 2026-11-09 — implemented on `task/T96-graph-keywords`. `check.sh --fuzz` green (1 697 library tests, 20 000 fuzz cases). The quality step changed nothing, so it has no commit.
- 2026-11-09 — the first count-line rule (`n m` with `m` lines after it) took real first edges such as `1 2` for counts in the tests' own examples, so `n` must also be at least every node name after it. Recorded in ADR-T96-1 and AC-4, in the behaviour commit.
- 2026-11-09 — keyword lines mixed into the fuzzer's edge lines dropped `graph_sessions_turn_text_blocks_into_graphs` to 32 of 200 sessions, under its 50. Keywords now go on the first line only and parent lists are generated on their own; the threshold is unchanged.
- 2026-11-09 — left for the owner: paste a tree as `n` plus edges under `tree:`, a directed problem's input under `dgraph:`, and a parent list under `parents:`, and judge the drawings.
