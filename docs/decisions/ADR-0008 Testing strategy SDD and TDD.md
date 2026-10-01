---
id: ADR-0008
title: Testing strategy SDD and TDD
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0003]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0008 Testing strategy SDD and TDD

## Context

The tool must not break during contests. The user asked for spec-driven and
test-driven development, with tests always written before code.

## Decision

- **SDD**: each task note has a *Spec* of acceptance criteria `AC-n`; every
  criterion names its test. Spec is committed before tests.
- **TDD**: tests are committed before models and behaviour (see [[Workflow]]).
- Test layers:
  - **Unit tests** in `#[cfg(test)] mod tests` next to the code, named
    `<subject>_<condition>_<expected>`, Arrange–Act–Assert.
  - **Property tests** (`proptest`) for math and invariants: camera round-trip,
    RDP keeps endpoints and stays within ε, undo/redo symmetry.
  - **Fuzz tests** for the whole `Editor` (see [[ADR-0009 Fuzzing with proptest]]).
  - **Shell**: manual checklist in [[T14 Performance and release validation]].
- Floats are compared with an explicit tolerance helper in `core::geom`, never `==`.

## Alternatives considered

- **Snapshot/golden image tests of rendering** — brittle across GPU drivers;
  not worth it for a thin shell.

## Consequences

- Red commits exist on task branches by design; `main` is always green.

## Rollback plan

N/A — process decision; changes go in an amending ADR.
