---
id: T104
title: Segment tree drawing
status: review
wave: 58
branch: task/T104-segment-tree
depends_on: [T102]
adrs: [ADR-T104-1]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T104 Segment tree drawing

## Goal

A text block `segtree: 8` turns with `Ctrl+L` into a segment tree over 8 positions, each node a circle labelled with its range `[l, r]`.

## Known facts (2026-11-10)

- `Ctrl+L` reads graph keywords ([[ADR-T96-1 Keywords for graph input]]); a tree is drawn in layers from its root (ADR-T87-1).
- Circles hold `$…$` text and grow to fit the widest name ([[ADR-T98-1 Math names and weights in graphs]]).
- `Ctrl+N` sets the first number to 1 or 0 (ADR-T86-1); `EdgeList::parse` is pure and does not see it today.
- Branches from T102 only because both edit `src/core/tools/layout.rs`.

## Spec

Rules: [[ADR-T104-1 Segment tree keyword]]. Unit tests in `src/core/tools/layout.rs` (section `T104 segment tree`) and `src/core/editor.rs` (section `T104 segment tree`).

- **AC-1** (owner) — A first line `segtree: n`, or `segtree:` followed by a line `n`, with `1 ≤ n ≤ 100`, builds the tree over positions `f … f+n−1`, `f` being the current first number (1, or 0 after `Ctrl+N`).
  Tests: `parse_segtree_reads_the_size_on_the_keyword_line`, `parse_segtree_reads_the_size_on_the_next_line`, `parse_segtree_starts_at_the_first_number`, `parse_segtree_accepts_the_bounds`, `parse_segtree_alone_is_no_graph`, `parse_without_segtree_ignores_the_first_number`; editor `segtree_from_text_starts_at_one`, `segtree_from_text_starts_at_zero_after_ctrl_n`.
- **AC-2** — Node `[l, r]` with `l < r` has children `[l, m]` and `[m+1, r]`, `m = ⌊(l+r)/2⌋`; leaves are `[i, i]`. That is `2n − 1` nodes.
  Tests: `parse_segtree_splits_at_the_floor_of_the_middle`, `parse_segtree_has_2n_minus_1_nodes`.
- **AC-3** (owner) — Every node is a circle holding `$[l, r]$`, typeset; circles share one size that fits the widest label; the tree is drawn in layers from `[f, f+n−1]`, children left to right.
  Tests: `shapes_segtree_circles_hold_their_ranges_at_one_size`, `shapes_segtree_is_layered_from_the_root_children_left_to_right`.
- **AC-4** — `n` outside `1…100`, not a number, or anything after it refuses the text and names the line in the log. One undo step; the result is selected, as other `Ctrl+L` results.
  Tests: `parse_segtree_refuses_a_bad_size_naming_its_line`, `tree_size_error_names_the_line_not_its_text`; editor `segtree_from_text_is_one_undo_step_and_selected`, `segtree_from_text_bad_size_changes_nothing_and_leaves_a_notice`.
- **AC-5** — Fuzzed with random `n`, including the bounds.
  Tests: proptest `segtree_parse_and_shapes_hold_for_any_size` (layout.rs, `n` in `0…=120`, both first numbers); `segtree:` texts with sizes 1, 100, 0, 101 and junk in `arb_edge_text` (`tests/fuzz.rs`).

## Out of scope

- Other data-structure templates (Fenwick, sparse table).
- Values inside nodes, lazy tags, half-open ranges.

## Files owned

- `src/core/tools/layout.rs`, `src/core/editor.rs`
- `docs/features/Numbered nodes.md`, `docs/features/Contest cheat sheet.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T104-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T104): specify segment tree drawing`
- [x] tests — `test(T104): add failing tests for segment tree drawing`
- [x] models — `feat(T104): add the tree size error and parse_numbered`
- [x] behaviour — `feat(T104): draw segment trees from segtree: blocks`
- [x] quality — `chore(T104): pass clippy and rustfmt`
- [x] docs — `docs(T104): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
- 2026-11-10 — spec: tests named per AC, rules in ADR-T104-1. UNATTENDED CHOICE: `segtree:` with no size is no graph (as any keyword alone, ADR-T96-1), not a refusal; bad sizes use a new `GraphError::NotATreeSize` so the log does not say "not an edge"; `EdgeList::parse_numbered(text, first)` carries the first number, `parse` keeps its meaning.
- 2026-11-10 — tests: red only for the missing `parse_numbered` and `NotATreeSize` (9 compile errors); the fuzzed graph session check stayed green with the new `segtree:` texts.
- 2026-11-10 — behaviour: `parse` now calls `parse_numbered(text, 1)`; the edge building moved into `EdgeList::from_raws`, whose root is a `NodeName`. `segment_tree_edges` writes the edges in preorder (left subtree before the right child), so the existing layered layout draws children left to right with no layout change.
- 2026-11-10 — quality: clippy asked for `u32::midpoint` (also used for the split in the code) and `assert_eq!` on an empty slice. `scripts/check.sh --fuzz` then failed `sessions_reach_deep_editor_states` at 19/200 sessions that could redo (threshold 20): the new `segtree:` choice shifted the deterministic stream. UNATTENDED CHOICE: fixed the generator, not the threshold — `arb_session` now uses `arb_graph_session_maybe_undone`, which presses `Ctrl+Z` after half of the fuzzed graph sessions; redo reaches 68/200. `arb_graph_session` itself is unchanged, so its own check still counts graphs left at the end.
- 2026-11-10 — `scripts/e2e.sh` not run: no key or existing drawing changed, `segtree:` is a new text only, and the command is not in this task's allowlist. Manual checks left for the owner: `segtree: 8` with first number 1 and after `Ctrl+N`; `segtree: 100` and judge the size of the drawing.
- 2026-11-10 — docs: feature note *Numbered nodes* and the *Contest cheat sheet* recipe updated; status `review`.
