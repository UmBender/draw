---
id: ADR-0014
title: Minimal dependencies and no unsafe
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0002, ADR-0003]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0014 Minimal dependencies and no unsafe

## Context

Every dependency is something that can break, slow the build, or bloat the
binary. The tool must just work mid-contest.

## Decision

- Runtime dependencies: **macroquad only**.
- Dev dependencies: **proptest only**.
- `unsafe_code = "forbid"` crate-wide.
- `Cargo.lock` is committed; versions are pinned until an ADR upgrades them.
- Release profile from rust-skills `perf-release-profile`: `opt-level = 3`,
  `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`.
  Dev profile builds dependencies with `opt-level = 3` so debug runs are smooth.

## Alternatives considered

- **thiserror/anyhow** — core has almost no fallible operations; plain enums
  suffice.

## Consequences

- Adding any dependency requires an ADR.

## Rollback plan

N/A — each new dependency is its own amending ADR.
