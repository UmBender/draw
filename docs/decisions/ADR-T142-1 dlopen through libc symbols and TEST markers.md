---
id: ADR-T142-1
title: dlopen through libc symbols and TEST markers on every unsafe block
status: accepted
kind: decision
date: 2026-11-13
task: T142
builds_on: [ADR-T132-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T142-1 dlopen through libc symbols and TEST markers on every unsafe block

## Context

`draw-sys` loads libwayland, libxcb, libEGL and libxkbcommon at run time.
No new dependency is allowed, so the `libc` and `libloading` crates are out.
Every `unsafe` primitive needs a test; the plan wants a script to enforce it.

## Decision

- `loader.rs` declares `dlopen`, `dlsym`, `dlclose` and `dlerror` in one
  `unsafe extern "C"` block. They live in glibc (2.34 and later), which
  `std` already links.
- `Library::open` is safe. `Library::symbol::<F>` is `unsafe`, because the
  caller states the type of the function; it checks only that `F` has the
  size of a pointer. `Symbol<'lib, F>` borrows the library, so it cannot
  outlive it.
- Every `unsafe` block, `unsafe fn`, `unsafe impl` and `unsafe extern` in
  `crates/*/src` carries a `// TEST: <fn_name>` comment (same line or the
  8 lines above). The named function must exist in the repository.
  `scripts/unsafe-tests.sh` runs the check (`every_unsafe_block_has_a_test_reference`).

## Alternatives considered

- **`libloading` crate** — a new dependency, and it hides the contract this
  task exists to write down.
- **Parse with `syn`** — a new dependency; a line scan is enough for a marker.

## Consequences

Linux with glibc only (the project target). A musl build would need the
same symbols, which musl also provides. The line scan can miss an `unsafe`
split across lines in odd formatting; rustfmt keeps them on one line.

## Rollback plan

Revert the T142 commits; later backends then declare their own loading.
