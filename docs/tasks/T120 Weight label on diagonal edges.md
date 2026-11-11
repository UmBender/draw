---
id: T120
title: Weight label on diagonal edges
status: review
wave: 60
branch: fix/weight-label-tie
depends_on: []
adrs: []
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task, bug]
---

# T120 Weight label on diagonal edges

## Goal

The weight written beside an edge never lands on a node, including on an edge that runs at exactly 45°.

## Known facts (2026-11-11)

- Reported by the owner with a `dgraph:` input on the circle layout: the weight `1` of the edge `1→5` was drawn over node `3`.
- `weight_text` (`src/core/tools/layout.rs`) puts the label on the "upper right" side of the edge by flipping the normal when `normal.x - normal.y < 0`. On an edge running from the upper right to the lower left (a `/` diagonal) the two normals are the lower right and the upper left and that difference is `0`, so the side is decided by float noise: sometimes the label is above the edge, sometimes below, over whatever node is there.
- Reproduced at text size 24 (two steps up the width ladder), with the block anywhere.

## Spec

- **AC-1** — On an exact `/` diagonal the label is always on the upper side, for any position and size of the drawing; on every other edge the side is unchanged.
  Tests: `weight_on_a_slash_diagonal_is_always_above`, `weight_side_is_stable_under_float_noise`, `weight_side_of_other_edges_is_unchanged`.
- **AC-2** — The owner's case: after `Ctrl+L` on the saved scene no weight overlaps a node.
  Test: `regression_T120_weight_label_diagonal` (snapshot `tests/snapshots/regressions/T120-weight-label-diagonal.snap`, built by `regression_fixture_T120_is_the_built_scene`).

## Out of scope

- Labels overlapping other edges; the circle layout itself.

## Files owned

- `src/core/tools/layout.rs`
- `tests/snapshots.rs`, `tests/snapshots/regressions/`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T120): specify the weight label fix`
- [x] regression — `test(T120): add regression test for the weight label on a diagonal`
- [x] behaviour — `fix(T120): put the weight above a slash diagonal`
- [x] quality — skipped: the fix commit passes clippy and rustfmt
- [x] docs — none: no user-visible rule changes beyond the fix

## Log

- 2026-11-11 — written from the owner's bug report.
- 2026-11-11 — fixed; `scripts/check.sh` green. The bug needs the width ladder two steps up (text size 24) to show with the owner's input; the regression scene sets it. A tie within `1e-4` now takes the upper side.
