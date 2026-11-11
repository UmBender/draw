---
id: T119
title: Forest layout in layers
status: done
wave: 60
branch: task/T119-forest-layers
depends_on: []
adrs: ["[[ADR-T119-1 Forests drawn in layers]]"]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T119 Forest layout in layers

## Goal

`Ctrl+L` on an edge list that is a forest (several separate trees) draws every tree in layers, side by side, instead of putting all the nodes on a circle.

## Known facts (2026-11-11)

- Reported by the owner with a `dgraph:` input: `ℵ 2 π`, `1 3 4`, `5 7 2`, `3 4 1`, `6 3 2`, `1 5 1`. Two components (`ℵ→2`, and a six-node tree), `6` edges for `8` nodes, so `EdgeList::layers` returned `None` and the circle was used.
- `EdgeList::layers` (`src/core/tools/layout.rs`) is also used by the segment tree drawing (T104), which always gives one tree.

## Spec

- **AC-1** — A forest is drawn in layers: each component from its first-named node, components left to right in the order first named, all roots on the top row, with no two circles overlapping.
  Tests: `forest_is_drawn_in_layers_side_by_side`, `forest_components_keep_the_order_first_named`, `forest_circles_never_overlap`, `forest_arrows_do_not_change_the_layout`.
- **AC-2** — A graph with a cycle (including opposite arrows) still goes on the circle, and a single tree keeps its positions.
  Tests: `cycle_goes_on_a_circle`, `opposite_arrows_are_not_a_tree`, `tree_is_drawn_in_layers_from_the_first_node_named`, `single_tree_positions_do_not_change_with_the_forest_layout`.
- **AC-3** — The owner's example gives two trees and no stray circle layout.
  Test: `forest_of_the_reported_input_has_two_trees`.
- **AC-4** — Any acyclic edge list, in any order, gives finite layers, and the segment tree drawing is unchanged. Fuzzed with random edge lists.
  Tests: `forest_layout_never_panics_and_stays_finite` (proptest, in `layout.rs`), the existing T104 tests.

## Out of scope

- Weights, labels and the circle layout (the label placement is [[T120 Weight label on diagonal edges]]).

## Files owned

- `src/core/tools/layout.rs`
- `docs/features/Numbered nodes.md`
- this note, and new `ADR-T119-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T119): specify the forest layout`
- [x] tests — `test(T119): add failing tests for the forest layout`
- [ ] models — skipped: no new types, the change is in `EdgeList::layers`
- [x] behaviour — `feat(T119): draw forests in layers`
- [x] quality — skipped: the behaviour commit already passes clippy and rustfmt
- [x] docs — `docs(T119): update the feature note`

## Learning path

None: it extends an existing feature.

## Log

- 2026-11-11 — written from the owner's bug report.
- 2026-11-11 — implemented; `scripts/check.sh` green. Fuzzing lives in the `layout.rs` proptests, so `tests/fuzz.rs` was not touched. `forest_goes_on_a_circle` was replaced by the forest tests (it asserted the behaviour the ADR amends).
