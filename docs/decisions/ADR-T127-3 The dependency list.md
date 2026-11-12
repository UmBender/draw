---
id: ADR-T127-3
title: The dependency list
status: accepted
kind: decision
date: 2026-11-12
task: T127
builds_on: [ADR-T127-1, ADR-T32-1, ADR-T92-1, ADR-T114-1]
amends: [ADR-0014]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, 2.0, dependencies]
---

# ADR-T127-3 The dependency list

## Context

[[ADR-0014 Minimal dependencies and no unsafe]] says "runtime dependencies:
macroquad only". `Cargo.toml` already lists `macroquad`, `fontdue`, `png`
and `ttf-parser` (added by ADR-T32-1, ADR-T92-1 and ADR-T114-1), and
`proptest` for tests. Owner decision, 2026-11-12: "**All the quality
proposals are accepted**", which includes item 5, a new ADR that lists the
real set.

## Decision

- Runtime dependencies today: `macroquad` (leaves with
  [[ADR-T127-4 An own backend replaces miniquad and macroquad]]),
  `fontdue`, `png`, `ttf-parser`.
- Runtime dependencies in 2.0.0: `fontdue`, `png`, `ttf-parser`; nothing
  else. libwayland, libxcb, EGL and xkbcommon are loaded with `dlopen` and
  are not crates.
- Dev dependencies: `proptest`. Tools that are not crates in the build
  (`cargo-llvm-cov`, Miri, sanitizers, `loom` only where a lock-free
  structure exists) are named in the task that uses them; `loom` would be
  its own dev-dependency ADR.
- Each addition needs an ADR; `Cargo.lock` stays committed and versions
  pinned.

## Alternatives considered

- **Bindings crates** (`wayland-client`, `x11rb`, `glow`, `winit`) — the
  owner chose an own backend, and each adds a tree to audit.
- **Leave ADR-0014 as is** — it no longer describes the tree.

## Consequences

- The list is true again and the next addition is visible in review.

## Rollback plan

Re-add the crate with its own ADR; this ADR's list is amended, not edited.
