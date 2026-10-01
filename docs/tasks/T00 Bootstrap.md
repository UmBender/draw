---
id: T00
title: Bootstrap
status: in-progress
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
files it owns. Running the binary opens an empty window (plain black: theme
colours belong to `core::palette`, filled by [[T02 Palette and theme tokens]] and
wired in by [[T12 App shell and toolbar]]).

## Spec

- **AC-1** — `cargo build` succeeds; `src/main.rs` only builds the window from
  `shell::app::window_conf()` and awaits `shell::app::run()`. `window_conf()` sets
  title `"draw"`, resizable, 1280×800, high-DPI on.
  *Test:* `tests/skeleton.rs::window_conf_has_expected_title_and_size`.
- **AC-2** — Every module in [[Architecture]]'s module map exists as a file with
  a `//!` module doc and is declared `pub` in its parent:
  `core::{geom, palette, camera, smoothing, shape, document, history, input, command, editor, clipboard, keymap}`,
  `core::tools::{navigate, pen, shape_tool, eraser, bucket, select}`,
  `shell::{render, input_map, toolbar, app}`.
  *Test:* `tests/skeleton.rs::all_modules_are_reachable` (one `use draw::… as _;` per module).
- **AC-3** — `Cargo.toml`: edition 2024, `rust-version = "1.85"`, `[lints]` exactly as in
  [[ADR-0010 Clippy lint policy]], profiles from [[ADR-0014 Minimal dependencies and no unsafe]],
  dependencies exactly `macroquad = "0.4"` and dev `proptest = "1"`; `Cargo.lock` committed.
  *Test:* `scripts/check.sh` (clippy gate) + review.
- **AC-4** — `scripts/check.sh` runs, in order and stopping at the first failure:
  `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`.
  `--fuzz` additionally runs `PROPTEST_CASES=20000 cargo test --release --test fuzz`
  when `tests/fuzz.rs` exists. *Test:* run green; break formatting locally → non-zero exit.
- **AC-5** — `rustfmt.toml` with `edition = "2024"`, `max_width = 100`. *Test:* fmt gate.
- **AC-6** — `.gitignore` ignores `/target` and Obsidian per-user state
  (`docs/.obsidian/workspace*.json`, `docs/.obsidian/cache`, `plugins/`, `.trash/`).
  Already done in the vault bootstrap commit on `main`; nothing left for this task.

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
