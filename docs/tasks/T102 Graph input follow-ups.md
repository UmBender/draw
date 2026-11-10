---
id: T102
title: Graph input follow-ups
status: review
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

- [x] spec — `docs(T102): specify graph input follow-ups`
- [x] tests — `test(T102): add failing tests for graph input follow-ups`
- [x] models — skipped: no new types or signatures (see *Log*)
- [x] behaviour — `feat(T102): read weights after pairs, a 0 root and weighted keywords`
- [x] quality — skipped: `scripts/check.sh --fuzz` was green with no change (see *Log*)
- [x] docs — `docs(T102): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
- 2026-11-10 — spec: [[ADR-T102-1 Weights after pairs, root 0 and weighted keywords]] amends ADR-T96-1 and ADR-T97-1. Tests named per AC.
- 2026-11-10 — tests: 8 new behaviour tests failed for the right reason; the refusal guards (`parse_refuses_a_weight_without_its_pair`, `parse_parents0_keeps_zero_as_a_node`) passed already, as they pin down unchanged behaviour. `1 > 3 7` left `parse_refuses_a_dangling_separator`. Fuzz edge text gains weighted pairs, the `w` keywords and a `00` parent entry.
- 2026-11-10 — models: skipped, the change adds only private helpers (`pair_edges`, `is_weight`); no new types or public signatures.
- 2026-11-10 — behaviour: `joined_pairs` now returns `Cow` tokens so a weight stays borrowed from the line. UNATTENDED CHOICE: the T87 case `1-2 3` in `parse_refuses_what_is_not_an_edge` is now a weighted pair by the owner's AC-1 (same form as `1-2 5`), so it became `1-2 3 4` (two weights in a row, still refused). UNATTENDED CHOICE: any entry worth 0 (`0`, `00`) is the root marker under `parents:`, not only the literal `0`, so `00` cannot name a node that list cannot have.
- 2026-11-10 — quality: skipped, `scripts/check.sh --fuzz` green (fmt, clippy, tests, docs, fuzz) with no change. `scripts/e2e.sh` not run: no key or drawing change.
- 2026-11-10 — manual check left for the owner: paste `1>3 7` and a `parents:` list with a `0` root, press `Ctrl+L`.
