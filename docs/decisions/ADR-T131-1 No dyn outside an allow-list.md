---
id: ADR-T131-1
title: No dyn outside an allow-list
status: accepted
kind: decision
date: 2026-11-12
task: T131
builds_on: [ADR-0010, ADR-0014]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T131-1 No dyn outside an allow-list

## Context

The owner decided on 2026-11-12 to minimise or remove dynamic dispatch and to
catch bugs at compile time. Today `src/` has a few `dyn` uses and the vendored
miniquad has many (its `EventHandler`, `RenderingBackend` and `Clipboard`
traits are objects).

## Decision

- `tests/no_dyn.rs` fails on the word `dyn` in code (not in comments or
  strings) under `src/` and `vendor/`, except in the files listed below.
- An entry names a file, the reason and the task that removes it. A file that
  no longer contains `dyn`, or does not exist, must be removed from the list
  (the guard checks it).
- `Cargo.toml` denies `clippy::wildcard_enum_match_arm` and
  `clippy::undocumented_unsafe_blocks` and sets `unsafe_op_in_unsafe_fn` to
  deny. A match on an enum lists its variants.
- Two exceptions to the wildcard rule, each a scoped `#[allow(…, reason)]`:
  key tables over our `Key` (editor sheet, palette and text keys) and over
  miniquad's `KeyCode` (about 120 variants, in `src/shell/input_map.rs`),
  where "every other key does nothing" is the meaning. Test code in `src/`
  is exempt through `#![cfg_attr(test, allow(…))]` in `src/lib.rs`, so a
  test can end a match with `other => panic!(…)`.

## Allow-list

- `src/shell/export.rs` — `Error::source` returns `&(dyn Error + 'static)`; the std signature. Stays.
- `src/shell/crash.rs` — the panic hook payload is `&(dyn Any + Send)`; the std signature. Stays.
- `vendor/miniquad/src/fs.rs` — boxed callback. Removed by T149.
- `vendor/miniquad/src/lib.rs` — `Context` and `EventHandler` objects. Removed by T149.
- `vendor/miniquad/src/native.rs` — boxed clipboard and request callback. Removed by T149.
- `vendor/miniquad/src/native/linux_wayland.rs` — boxed `EventHandler`. Removed by T149.
- `vendor/miniquad/src/native/linux_x11.rs` — boxed `EventHandler`. Removed by T149.
- `vendor/miniquad/src/native/windows.rs` — boxed `EventHandler`. Removed by T149.
- `vendor/miniquad/src/native/macos.rs` — boxed `EventHandler`. Removed by T149.
- `vendor/miniquad/src/native/ios.rs` — boxed `EventHandler`. Removed by T149.
- `vendor/miniquad/src/native/android.rs` — boxed `EventHandler`. Removed by T149.
- `vendor/miniquad/src/native/wasm.rs` — boxed `EventHandler`. Removed by T149.

## Alternatives considered

- **Forbid `dyn` everywhere** — impossible for std signatures and the vendored
  window layer until T149.
- **A clippy lint only** — clippy has no stable lint for `dyn`.

## Consequences

New `dyn` needs an ADR amendment. Closing T149 empties the vendor entries.

## Rollback plan

Delete `tests/no_dyn.rs` and the two lint lines; supersede this ADR.
