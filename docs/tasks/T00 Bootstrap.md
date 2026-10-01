---
id: T00
title: Bootstrap
status: ready
wave: 0
branch: task/T00-bootstrap
depends_on: []
adrs: ["[[ADR-0003 Headless core and thin shell]]", "[[ADR-0010 Clippy lint policy]]", "[[ADR-0014 Minimal dependencies and no unsafe]]"]
feature:
tutorial: "[[00 Project layout, lints and the TDD loop]]"
tags: [task]
---

# T00 Bootstrap

## Goal

A compiling crate with the final module skeleton, lint policy, profiles,
dependencies and the `scripts/check.sh` gate, so every later task only fills
files it owns. Running the binary opens an empty window in the background
colour.

## Spec

- **AC-1** — `cargo build` succeeds; `src/main.rs` only builds the window from
  `shell::app::window_conf()` and awaits `shell::app::run()`. *Test:* build in check.sh.
- **AC-2** — Every module in [[Architecture]]'s module map exists as a file with
  a `//!` module doc and is declared in its parent. *Test:* `tests/skeleton.rs::all_modules_are_reachable`
  (references each module path via `use draw::core::geom as _;` etc.).
- **AC-3** — `Cargo.toml` contains the `[lints]` from [[ADR-0010 Clippy lint policy]]
  and profiles from [[ADR-0014 Minimal dependencies and no unsafe]];
  dependencies are exactly `macroquad` and dev `proptest`. *Test:* check.sh clippy pass.
- **AC-4** — `scripts/check.sh` runs fmt-check, clippy `-D warnings`,
  tests and `cargo doc` with `-D warnings`; exits non-zero on any failure;
  `--fuzz` sets `PROPTEST_CASES=20000` and runs `--test fuzz` if it exists. *Test:* run it green.
- **AC-5** — `rustfmt.toml` exists (edition 2024 defaults, `max_width = 100`).
- **AC-6** — `.gitignore` ignores `/target` and Obsidian per-user state
  (`docs/.obsidian/workspace*.json`, `docs/.obsidian/cache`).

## Out of scope

Any logic. Stub modules contain only docs (and `app::run` = clear + `next_frame` loop).

## Files owned

`Cargo.toml`, `Cargo.lock`, `rustfmt.toml`, `.gitignore`, `scripts/check.sh`,
`src/main.rs`, `src/lib.rs`, `src/core/mod.rs`, `src/core/*.rs` (stubs),
`src/core/tools/*.rs` (stubs), `src/shell/mod.rs`, `src/shell/*.rs` (stubs),
`tests/skeleton.rs`.

## Subtasks (one commit each)

- [ ] spec — `docs(T00): specify bootstrap acceptance criteria`
- [ ] tests — `test(T00): add module skeleton reachability test`
- [ ] models — `feat(T00): add crate skeleton, lints and profiles`
- [ ] behaviour — `feat(T00): open empty window and add check script`
- [ ] quality — `chore(T00): pass fmt, clippy and doc gates`
- [ ] docs — `docs(T00): add bootstrap tutorial`

## Learning path

Step 0 — no prerequisites.

## Log
