---
id: ADR-0003
title: Headless core and thin shell
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0002]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0003 Headless core and thin shell

## Context

TDD and fuzzing need logic that runs without a window or GPU. Parallel
implementation needs modules with clear boundaries.

## Decision

- Library + binary split: `src/lib.rs` holds everything; `src/main.rs` only
  calls `draw::shell::app::run()`.
- `core` module: pure Rust, **no dependencies**, own `Vec2`/`Aabb` types.
  Takes `InputEvent`s, exposes state. All editing logic lives here.
- `shell` module: the only code that imports macroquad. Converts input,
  renders state, draws the toolbar. No editing logic.
- [[T00 Bootstrap]] creates every module file up front so later tasks only fill
  files they own (see [[Parallel Execution]]).

## Alternatives considered

- **Cargo workspace with two crates** — cleaner boundary but more ceremony;
  a module boundary plus a review rule is enough at this size.
- **Using macroquad's `Vec2` (glam) in core** — couples core to the
  renderer's version of glam.

## Consequences

- `shell::render` converts `core::geom::Vec2` → macroquad `Vec2` (trivial).
- The shell is checked by hand ([[T14 Performance and release validation]]),
  not by unit tests.

## Rollback plan

Converting to a workspace later is mechanical (move `core` into its own crate).
