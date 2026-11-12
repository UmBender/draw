---
id: T132
title: Workspace split with unsafe levels
status: in-progress
wave: 65
branch: task/T132-workspace-split-with-unsafe-levels
depends_on: [T131]
adrs: [ADR-T132-1]
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

Decision: [[ADR-T132-1 Workspace of five crates with unsafe levels]] (layout, level of each crate, the list of files that may allow `unsafe`, the dependency list per crate).

- **AC-1** — The cargo workspace builds with the same binary, tests and packaging as before; `core` and `shell` keep their module paths through re-exports until the later tasks move them.
  `scripts/check.sh`, `scripts/e2e.sh` unchanged and green. Tests: `workspace_lists_the_five_crates` (in `tests/unsafe_levels.rs`).
- **AC-2** — `draw-types`, `draw-core` and `draw` declare `forbid(unsafe_code)`; `draw-sys` and `draw-gfx` declare `deny`, with `allow` only in the files the ADR lists.
  Tests in `tests/unsafe_levels.rs`: `crate_roots_declare_their_level`, `allow_unsafe_only_in_listed_files`, `scanner_finds_allow_attribute`, `scanner_ignores_comments_and_strings`.
- **AC-3** — The dependency list of `ADR-T127-3` is checked: `tests/deps.rs` fails when a crate adds a dependency not in it.
  Tests: `deps_match_the_adr`, `reader_finds_every_dependency_table`, `path_dependencies_stay_between_draw_crates`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `Cargo.toml` and the new crate manifests, `tests/unsafe_levels.rs`, `tests/deps.rs`, `scripts/*.sh`, `packaging/`
- moves of `src/` files (no change of content except the paths that leave the old crate, see the ADR), `crates/**`, `src/lib.rs`, `src/main.rs`
- `docs/decisions/ADR-T132-1*`, `docs/process/Quality Gates.md`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T132): specify the workspace split`
- [ ] tests — `test(T132): add failing tests for the unsafe levels`
- [ ] behaviour — `refactor(T132): split into crates`
- [ ] quality — `chore(T132): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
