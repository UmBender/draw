---
id: T132
title: Workspace split with unsafe levels
status: todo
wave: 65
branch: task/T132-workspace-split-with-unsafe-levels
depends_on: [T131]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T132 Workspace split with unsafe levels

## Goal

The code lives in crates whose `unsafe` level is enforced by the compiler: `draw-types`, `draw-core`, `draw-gfx`, `draw-sys` and `draw`.

## Known facts (2026-11-12)

- [[2.0 Architecture proposal]] §2. Only `draw-sys` (and one file of `draw-gfx`) may contain `unsafe`; the others are `forbid`.
- Today everything is one crate with `unsafe_code = "forbid"`; the vendored miniquad is outside it.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — The cargo workspace builds with the same binary, tests and packaging as before; `core` and `shell` keep their module paths through re-exports until the later tasks move them.
  `scripts/check.sh`, `scripts/e2e.sh` unchanged and green.
- **AC-2** — `draw-types`, `draw-core` and `draw` declare `forbid(unsafe_code)`; `draw-sys` and `draw-gfx` declare `deny`, with `allow` only in the files an ADR lists.
  Test: `tests/unsafe_levels.rs` reads each crate root and each `#[allow(unsafe_code)]` against the list.
- **AC-3** — The dependency list of `ADR-T127-3` is checked: `tests/deps.rs` fails when a crate adds a dependency not in it.
  Tests: `deps_match_the_adr`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `Cargo.toml` and the new crate manifests, `tests/unsafe_levels.rs`, `tests/deps.rs`, `scripts/*.sh`, `packaging/`
- moves of `src/` files (no change of content)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T132): specify the workspace split`
- [ ] tests — `test(T132): add failing tests for the unsafe levels`
- [ ] behaviour — `refactor(T132): split into crates`
- [ ] quality — `chore(T132): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
