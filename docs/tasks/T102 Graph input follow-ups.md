---
id: T102
title: Graph input follow-ups
status: in-progress
wave: 57
branch: task/T102-graph-input-follow-ups
depends_on: []
adrs: [ADR-T102-1]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T102 Graph input follow-ups

## Goal

Weights follow pairs (`1>3 7`), `parents:` lists may mark the root with `0`, and `wtree:`, `wgraph:` and `wdgraph:` are accepted as keywords.

## Known facts (2026-11-10)

- A line of pairs refuses any token that is not a pair, so `1>3 7` and `1 > 3 7` are refused ([[ADR-T97-1 Spaced edge pairs]]).
- In `parents:` nodes start at 1, so `0` cannot be a node there; in `parents0:` it can ([[ADR-T96-1 Keywords for graph input]]).
- Weights may be numbers or `$…$` formulas ([[ADR-T98-1 Math names and weights in graphs]]).

## Spec

The spec step names the test functions of each AC.

Decisions: [[ADR-T102-1 Weights after pairs, root 0 and weighted keywords]] (amends ADR-T96-1 and ADR-T97-1).

- **AC-1** (owner) — In a line of pairs, a token that is a weight (finite number or `$…$`) right after a pair is that pair's weight: `1>3 7`, `1 > 3 $w$`, `1-2 5 2-3 9`. Two weights in a row, or a weight first, refuse the line.
  Tests (`src/core/tools/layout.rs`): `parse_reads_a_weight_after_a_pair`, `parse_reads_a_weight_after_a_spaced_pair`, `parse_reads_weights_after_several_pairs`, `parse_refuses_a_weight_without_its_pair`; (`src/core/editor.rs`): `graph_from_text_reads_weights_after_pairs`.
- **AC-2** — `-5` after a pair is a weight, never joined as a separator (ADR-T97-1 keeps a sign token apart).
  Test: `parse_reads_a_negative_weight_after_a_pair`.
- **AC-3** (owner) — In `parents:`, one entry `0` marks the root exactly as `-1` does (one entry per node); a second root marker (`0` or `-1`) refuses the text. In `parents0:` `0` stays node 0.
  Tests: `parse_parents_reads_zero_as_the_root`, `parse_parents_refuses_a_second_root_marker`, `parse_parents0_keeps_zero_as_a_node`.
- **AC-4** (owner) — `wtree:` and `wgraph:` read as `tree:` / `graph:`, and `wdgraph:` as `dgraph:`.
  Test: `parse_weighted_keywords_read_as_their_plain_keywords`.
- **AC-5** — Without these forms, every earlier test passes unchanged; the fuzzer's edge lines include weighted pairs and the new keywords.
  Tests: the existing suite (the one case `1 > 3 7` leaves `parse_refuses_a_dangling_separator`, since AC-1 now reads it); `tests/fuzz.rs` `arb_edge_text` gains weighted pair lines, `wtree:` / `wgraph:` / `wdgraph:` headers and `0` roots, checked by the existing session properties.

## Out of scope

- Weights on parent-list edges.
- Other keyword spellings.

## Files owned

- `src/core/tools/layout.rs`, `src/core/editor.rs`
- `docs/features/Numbered nodes.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T102-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T102): specify graph input follow-ups`
- [ ] tests — `test(T102): add failing tests for graph input follow-ups`
- [ ] models — `feat(T102): …`
- [ ] behaviour — `feat(T102): …`
- [ ] quality — `chore(T102): pass clippy and rustfmt`
- [ ] docs — `docs(T102): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
