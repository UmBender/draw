---
id: T87
title: Graph or tree from an edge list
status: in-progress
wave: 53
branch: task/T87-graph-from-edges
depends_on: [T84]
adrs: [ADR-T87-1]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T87 Graph or tree from an edge list

## Goal

A selected text block of edges becomes a drawn graph or tree in one key. (idea C2 of [[Feature proposal]])

## Known facts (2026-11-09)

- Labels are `Option<u32>`, so node names must be numbers.
- Arrows snap to outlines ([[T20 Outline snapping]]) but the result here is computed directly.
- `Ctrl+L` is unbound; `Ctrl+Shift+L` is the diagnostics dump.

## Spec

Rules and layout: [[ADR-T87-1 Graph from an edge list key and layout rules]]. Tests live in `src/core/tools/layout.rs`, `src/core/editor.rs` (`graph_from_text_*`) and `src/core/keymap.rs`.

- **AC-1** — With one text block selected, `Ctrl+L` replaces it with numbered circles and edges, one undo step.
  Tests: `ctrl_l_builds_a_graph_from_text`, `graph_from_text_replaces_the_block`, `graph_from_text_keeps_the_block_place_and_colour`, `graph_from_text_is_one_undo_step`, `graph_from_text_selects_the_result`, `graph_from_text_cancels_a_gesture`, `graph_from_text_in_open_edit_only_types`, `shapes_label_circles_with_node_names`, `shapes_write_node_zero_as_text`.
- **AC-2** — It accepts `1-2 2-3`, `1>2` for directed edges, and lines of `u v` or `u v w`; a weight is drawn as text at the edge's middle.
  Tests: `parse_reads_pairs`, `parse_reads_directed_pairs`, `parse_reads_plain_lines_and_weights`, `parse_names_nodes_in_first_named_order`, `parse_blank_text_is_no_graph`, `shapes_draw_arrows_only_for_directed_edges`, `shapes_put_a_weight_beside_the_middle_of_its_edge`.
- **AC-3** — If the edges form a tree it is drawn in layers from the first node named; anything else goes on a circle.
  Tests: `tree_is_drawn_in_layers_from_the_first_node_named`, `tree_parent_is_centred_over_its_children`, `tree_ignores_arrow_direction`, `cycle_goes_on_a_circle`, `forest_goes_on_a_circle`, `circle_radius_and_layer_gap_grow_with_the_node_count`, `nodes_never_overlap` (proptest).
- **AC-4** — Edges end on the circles' outlines, and all results are ordinary shapes every tool can edit.
  Tests: `edges_end_on_the_outlines`, `graph_from_text_shapes_are_ordinary`, `graph_from_text_survives_a_snapshot`.
- **AC-5** — More than 200 nodes, or text that does not parse: nothing changes and the reason goes to the diagnostics log.
  Tests: `parse_accepts_200_nodes`, `parse_refuses_more_than_200_nodes`, `parse_refuses_what_is_not_an_edge`, `parse_refuses_too_much_text`, `parse_error_names_the_line_not_its_text`, `graph_from_text_bad_text_changes_nothing_and_leaves_a_notice`, `graph_from_text_too_many_nodes_changes_nothing_and_leaves_a_notice`, `graph_from_text_without_one_text_block_does_nothing`. The shell writes `Editor::take_notice` with `diag::log` after every event (T84).
- **AC-6** — Self-loops and duplicate edges do not panic; a self-loop is skipped.
  Tests: `parse_skips_a_self_loop_but_keeps_its_node`, `parse_draws_a_duplicate_once`, `parse_keeps_edges_that_differ_in_direction_or_weight`, `parse_and_shapes_hold_for_any_text` (proptest), `graph_from_text_only_self_loops_draws_the_nodes`.

The fuzzer reaches `Ctrl+L` through the bound chords and through a session built for it (`arb_graph_session`, `graph_sessions_turn_text_blocks_into_graphs` in `tests/fuzz.rs`).

## Out of scope

- Segment tree templates.
- Force-directed layout.
- A leading `n m` line is read as an edge; there is no way to tell them apart.

## Files owned

- new `src/core/tools/layout.rs`, `src/core/tools/mod.rs`, `src/core/shape.rs`
- `src/core/numbering.rs`
- `docs/features/Numbered nodes.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- this note, and new `ADR-T87-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T87): specify graph or tree from an edge list`
- [ ] tests — `test(T87): add failing tests`
- [ ] models — `feat(T87): add types`
- [ ] behaviour — `feat(T87): implement`
- [ ] quality — `chore(T87): pass clippy and rustfmt`
- [ ] docs — `docs(T87): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
