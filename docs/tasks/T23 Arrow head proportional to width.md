---
id: T23
title: Arrow head proportional to width
status: done
wave: 11
branch: fix/arrow-head-scale
depends_on: [T05]
adrs: ["[[ADR-0004 Vector object model]]", "[[ADR-0013 World-space widths and zoom limits]]", "[[ADR-0016 Arrow head proportional to width]]"]
feature: "[[Shapes]]"
tutorial: "[[05 Modelling shapes and hit-testing]]"
tags: [task]
---

# T23 Arrow head proportional to width

## Goal

An arrow drawn while zoomed in has a thin world width
([[ADR-0013 World-space widths and zoom limits]]), but its head stayed at the
8-unit floor of [[T05 Shape model]], so the head looked oversized next to the
shaft. The head must keep its proportion to the shaft at any zoom.

Reported by the owner on 2026-10-02. Branched from `main` after T22 was
integrated.

## Spec

Decision: [[ADR-0016 Arrow head proportional to width]].

- **AC-1** — Head length is `ARROW_HEAD_LENGTH_PER_WIDTH * width` with no
  absolute minimum; negative or NaN widths count as 0.
  `ARROW_HEAD_MIN_LENGTH` is removed. *Tests:*
  `shape::tests::arrow_head_scales_with_thin_width` (replaces
  `arrow_head_has_min_size`), `shape::tests::arrow_head_length_scales_with_width`.
- **AC-2** — Scaling an arrow and its width by `k` scales its head by `k`.
  *Tests:* `shape::tests::arrow_head_is_scale_invariant` (proptest).
- **AC-3** — Bounds and hit-testing follow the new head size. *Tests:*
  `shape::tests::bounds_arrow`, the arrow hit test (head sizes now derived
  from the width).

## Out of scope

A minimum head size in screen pixels, storing the head size on the shape
(both rejected in ADR-0016), changing `ARROW_HEAD_LENGTH_PER_WIDTH`.

## Files owned

`src/core/shape.rs`, `docs/features/Shapes.md`,
`docs/tutorials/05 Modelling shapes and hit-testing.md`, new `ADR-0016`.

## Subtasks (one commit each)

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs

## Learning path

No new step — tutorial 05's "Try it" now uses `ARROW_HEAD_LENGTH_PER_WIDTH`.

## Log

- 2026-10-02 — spec: ADR-0016 (commit `docs: ADR-0016 …`); written as a
  fix, so this note was added at integration.
- tests: red because `arrow_head_scales_with_thin_width` still sees the
  8-unit floor.
- models + behaviour: one commit — removing the constant and the `max` is
  the whole change.
- quality: `scripts/check.sh` green with no changes, so no quality commit.
- docs: tutorial 05 on the branch; `Shapes` feature note at integration.
  [[T05 Shape model]] still describes the old minimum as the original spec.
