---
id: ADR-T146-1
title: draw-gfx may depend on draw-core
status: accepted
kind: decision
date: 2026-11-15
task: T146
builds_on: [ADR-T132-1]
amends: [ADR-T132-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr, 2.0, workspace]
---

# ADR-T146-1 draw-gfx may depend on draw-core

## Context

[[ADR-T132-1 Workspace of five crates with unsafe levels]] lets `draw-gfx`
depend on no crate of the workspace but `draw-types` and `draw-sys`. The
retained renderer of [[T146 Retained renderer in draw-gfx]] builds its chunks
from the document's shapes, which live in `draw-core`.

## Decision

- `draw-gfx` may depend on `draw-core` (and so on `draw-types`).
- `draw-core` never depends on `draw-gfx`: the dependency goes one way.
- `tests/deps.rs` lists `draw-core` in the `draw-gfx` row. Nothing else
  of ADR-T132-1 changes.

## Alternatives considered

- **Copy the shape types into `draw-gfx`** — two models to keep equal.
- **Move the chunks into `draw-core`** — the core would hold GPU-facing
  geometry and the cache that the tests compare with a recomputed one.

## Consequences

- The chunk cache reads shapes directly; no conversion layer.
- A cycle between the two crates is refused by cargo and by `tests/deps.rs`.

## Rollback plan

Remove `draw-core` from the row and from `crates/draw-gfx/Cargo.toml`, with
a new ADR; the chunks would then need their own shape view.
