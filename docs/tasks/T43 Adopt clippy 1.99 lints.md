---
id: T43
title: Adopt clippy 1.99 lints
status: in-progress
wave: 28
branch: chore/clippy-1-99
depends_on: []
adrs: []
feature:
tutorial:
tags: [task]
---

# T43 Adopt clippy 1.99 lints

## Goal

`scripts/check.sh` passes again on rustc 1.99.1. The toolchain update
(2026-10-13) brought new clippy lints that fail the `-D warnings` gate
on code that has not changed. After this task the code satisfies them,
with no change in behaviour.

## Known facts (2026-10-27, rustc 1.99.1)

`cargo clippy --all-targets -- -D warnings` reports 46 errors outside
the T42 files (T42 fixes the 2 in `src/shell/input_map.rs` and
`src/shell/app.rs`):

- `assert_is_empty` (42, all in tests): `assert!(x.is_empty())` and
  `assert!(!x.is_empty())`.
- `manual_midpoint` (3): `(a + b) * 0.5` in `src/core/geom.rs`,
  `src/shell/render.rs`, `src/shell/toolbar.rs`.
- `double_must_use` (1): `src/core/document.rs`, a `#[must_use]` on a
  function whose return type is already `#[must_use]`.

## Spec

No new behaviour, so no new tests: the existing suite is the spec.

- **AC-1** — `cargo clippy --all-targets -- -D warnings` reports no
  error in the files owned. Together with T42 it is clean:
  `scripts/check.sh` passes on a test merge of both branches.
- **AC-2** — every existing test still passes, with the same number of
  tests (1033 with T42).
- **AC-3** — empty checks become `assert_eq!(x, [])` /
  `assert_eq!(s, "")`; non-empty checks become `assert_ne!(s, "")` for
  strings and `assert_ne!(x.len(), 0)` for collections. Midpoints use
  `f32::midpoint`. The lints are fixed, not allowed: no new `#[allow]`
  and no change to `[lints.clippy]`.

## Out of scope

- Pinning the toolchain (`rust-toolchain.toml`).
- Any other refactor in the touched files.
- `src/shell/input_map.rs` and `src/shell/app.rs` (owned by T42).

## Files owned

- `src/core/clipboard.rs`, `src/core/document.rs`, `src/core/editor.rs`,
  `src/core/geom.rs`, `src/core/shape.rs`, `src/core/smoothing.rs`,
  `src/core/snap.rs`, `src/core/tex.rs`
- `src/core/tools/bucket.rs`, `src/core/tools/eraser.rs`,
  `src/core/tools/grid.rs`, `src/core/tools/pen.rs`,
  `src/core/tools/select.rs`, `src/core/tools/shape_tool.rs`
- `src/shell/preflight.rs`, `src/shell/render.rs`,
  `src/shell/toolbar.rs`

## Subtasks (one commit each)

- [x] spec — `docs(T43): specify the clippy 1.99 acceptance criteria`
- [ ] tests — `test(T43): show values in empty and non-empty asserts`
- [ ] behaviour — `refactor(T43): use f32::midpoint and drop a double must_use`
- [ ] docs — `docs(T43): record the clippy 1.99 cleanup`

Models and quality have no separate commit: there are no new types, and
the refactor commit is the quality step.

## Learning path

No tutorial: a lint cleanup teaches nothing new.

## Log

- 2026-10-27 — Created while finishing T42, whose quality step found the
  gate broken by the toolchain update.
