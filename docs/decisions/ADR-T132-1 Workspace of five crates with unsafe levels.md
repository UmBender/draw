---
id: ADR-T132-1
title: Workspace of five crates with unsafe levels
status: accepted
kind: decision
date: 2026-11-12
task: T132
builds_on: [ADR-T127-1, ADR-T127-2, ADR-T127-3]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, 2.0, unsafe, workspace]
---

# ADR-T132-1 Workspace of five crates with unsafe levels

## Context

[[ADR-T127-2 The unsafe policy]] allows `unsafe` only in `draw-sys` and in
`draw-gfx/gpu/gl.rs`, and wants the compiler to enforce it per crate.
Today everything is one crate. The sources are 57 000 lines of `core` that
spell their own paths as `crate::core::…` about 400 times.

## Decision

- The root package `draw` stays the binary and a facade, and the root
  `Cargo.toml` gains a `[workspace]` with the members `crates/draw-types`,
  `crates/draw-core`, `crates/draw-gfx` and `crates/draw-sys`. The vendored
  miniquad stays a `[patch]` outside it.
- `draw-types` holds `geom` (`Vec2`, `Rect` and friends, the units that
  [[T133 Typed units and ids]] types). `draw-core` holds what was
  `src/core` and `src/theme.rs`, at the same module paths (`draw_core::core`,
  `draw_core::theme`), and `draw` re-exports them, so `draw::core::…` and
  `draw::theme` keep working. `src/shell` stays in `draw` until the tasks
  that fill `draw-gfx` and `draw-sys`; those two start as crates with their
  lint level and no code.
- Moved files change only where a path leaves the old crate (`include_str!`,
  `CARGO_MANIFEST_DIR`, the colour guard's search root, `core/mod.rs`
  re-exporting `geom`).
- Every crate root declares its level with an attribute, because a
  workspace lint table cannot differ per crate:
  `forbid(unsafe_code)` in `draw-types`, `draw-core` and `draw`;
  `deny(unsafe_code)` in `draw-gfx` and `draw-sys`.
- **The list of files that may `#![allow(unsafe_code)]`** (checked by
  `tests/unsafe_levels.rs`): every file under `crates/draw-sys/src/`, and
  `crates/draw-gfx/src/gpu/gl.rs`. Nothing else.
- **The dependency list** of [[ADR-T127-3 The dependency list]] is checked by
  `tests/deps.rs`: `draw-types` none; `draw-core` `fontdue`, `png`,
  `ttf-parser`; `draw-gfx` and `draw-sys` none yet; `draw` `macroquad` and
  the four `draw-*` crates; dev dependency `proptest` only. A path
  dependency is allowed only between the `draw-*` crates.
- Shared lints live in `[workspace.lints]`; every member has
  `[lints] workspace = true`.

## Alternatives considered

- **Make `draw-core`'s root the old `core` module** — every
  `crate::core::` path (about 400) would change.
- **Move `src/shell` now** — it mixes platform code and rendering; the
  tasks after this one split it with tests.
- **Keep `theme` in `draw-types`** — it is constants, not types, and the
  colour guard scans `src` for it.

## Consequences

- The compiler refuses `unsafe` outside `draw-sys` and one file of `draw-gfx`.
- The first `unsafe` primitive adds its file to the list by a new ADR.
- `geom` is no longer counted in the `core` coverage module.

## Rollback plan

Return the crates to `src/` with `git revert` of the split commit and a
new ADR; the module paths did not change, so no caller needs an edit.
