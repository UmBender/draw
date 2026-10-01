---
title: Project layout, lints and the TDD loop
step: 0
requires: []
feature:
code: ["Cargo.toml", "src/lib.rs", "src/main.rs", "src/shell/app.rs", "scripts/check.sh", "tests/skeleton.rs"]
tags: [tutorial]
---

# Project layout, lints and the TDD loop

> **Learning path step 0.** Requires: nothing · Next: [[01 2D vectors, AABBs and point-segment distance]], [[02 Palettes and design tokens]]

## Why this matters here

Everything later depends on three things set up in this step: a layout that
keeps logic testable without a window, a lint policy that catches mistakes
before they reach a contest, and one command that tells you whether the
project is healthy.

## The concept

### Library + binary

A Cargo package can have both `src/lib.rs` (a library crate) and `src/main.rs`
(a binary crate). The binary uses the library like any external user would:
`draw::shell::app::run()`. Integration tests in `tests/` also link only the
library. So anything worth testing must live in the library, and `main.rs`
stays a one-liner.

```
src/
├── main.rs          binary: open window, call shell::app::run
├── lib.rs           library root: pub mod core; pub mod shell;
├── core/            pure logic — no macroquad
│   ├── geom.rs …    one file per concern
│   └── tools/       one file per tool
└── shell/           macroquad: app loop, input_map, render, toolbar
```

The `core`/`shell` split ([[ADR-0003 Headless core and thin shell]]) means
`cargo test` never needs a GPU.

### Lints in `Cargo.toml`

Since Rust 1.74, lint levels can live in a `[lints]` table, so `cargo build`,
`cargo clippy`, your editor and every agent use the same rules:

```toml
[lints.clippy]
all = { level = "deny", priority = -1 }      # group, applied first
pedantic = { level = "warn", priority = -1 }
cast_precision_loss = "allow"                # individual overrides win
```

`priority = -1` makes the groups apply first so individual lints can override
them. Then `-D warnings` in the check script promotes every warning to an
error. See [[ADR-0010 Clippy lint policy]].

When a lint has to be silenced, say why — Rust supports a `reason`:

```rust
#[allow(unused_imports, reason = "the imports are the test")]
```

### The TDD loop, one commit per step

```
spec ─► test (red) ─► models (compiles, still red) ─► behaviour (green) ─► quality ─► docs
```

T00 itself followed it: `tests/skeleton.rs` was committed while the `draw`
library did not exist yet (23 compile errors — red), then stubs made it compile
with `window_conf` still a `todo!()` (one test failing), then the real
implementation turned it green. See [[Workflow]].

## How draw implements it

- `tests/skeleton.rs::all_modules_are_reachable` imports every module path. If a
  later change deletes or renames a module, the build breaks immediately.
- `tests/skeleton.rs::window_conf_has_expected_title_and_size` checks the window
  config returned by `shell::app::window_conf`.
- `src/main.rs` uses `macroquad::Window::from_config(conf, future)` instead of
  the `#[macroquad::main]` attribute, so the config is an ordinary, testable
  function.
- `scripts/check.sh` runs fmt → clippy → tests → docs with `set -euo pipefail`,
  stopping at the first failure.
- Release profile (`lto = "fat"`, `codegen-units = 1`, `panic = "abort"`) gives a
  smaller, faster binary; `[profile.dev.package."*"] opt-level = 3` keeps
  dependencies fast even in debug builds.

## Try it

1. Run `scripts/check.sh` — all green.
2. Add `pub   fn  x( ){}` to `src/core/geom.rs` and run it again: the *format*
   gate fails with exit code 1. Revert.
3. Comment out `pub mod pen;` in `src/core/tools/mod.rs` and run `cargo test`:
   the skeleton test no longer compiles. Revert.
4. `cargo run` — an empty black window titled "draw".

## Further reading

- The Cargo Book — [Cargo targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html), [`[lints]`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section), [profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Clippy lint list](https://rust-lang.github.io/rust-clippy/)
- rust-skills rules `proj-lib-main-split`, `lint-*`, `perf-release-profile`, `test-integration-dir`
