---
id: T104
title: Segment tree drawing
status: todo
wave: 58
branch: task/T104-segment-tree
depends_on: [T102]
adrs: []
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

The spec step names the test functions of each AC.

- **AC-1** (owner) — A first line `segtree: n`, or `segtree:` followed by a line `n`, with `1 ≤ n ≤ 100`, builds the tree over positions `f … f+n−1`, `f` being the current first number (1, or 0 after `Ctrl+N`).
- **AC-2** — Node `[l, r]` with `l < r` has children `[l, m]` and `[m+1, r]`, `m = ⌊(l+r)/2⌋`; leaves are `[i, i]`. That is `2n − 1` nodes.
- **AC-3** (owner) — Every node is a circle holding `$[l, r]$`, typeset; circles share one size that fits the widest label; the tree is drawn in layers from `[f, f+n−1]`, children left to right.
- **AC-4** — `n` outside `1…100`, not a number, or anything after it refuses the text and names the line in the log. One undo step; the result is selected, as other `Ctrl+L` results.
- **AC-5** — Fuzzed with random `n`, including the bounds.

## Out of scope

- Other data-structure templates (Fenwick, sparse table).
- Values inside nodes, lazy tags, half-open ranges.

## Files owned

- `src/core/tools/layout.rs`, `src/core/editor.rs`
- `docs/features/Numbered nodes.md`, `docs/features/Contest cheat sheet.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T104-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T104): specify segment tree drawing`
- [ ] tests — `test(T104): add failing tests for segment tree drawing`
- [ ] models — `feat(T104): …`
- [ ] behaviour — `feat(T104): …`
- [ ] quality — `chore(T104): pass clippy and rustfmt`
- [ ] docs — `docs(T104): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written in the batch plan with the owner; owner decisions are in *Spec* and in `.batch/plan.md`.
