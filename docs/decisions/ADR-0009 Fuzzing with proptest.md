---
id: ADR-0009
title: Fuzzing with proptest
status: accepted
kind: decision
date: 2026-10-01
task: T00
builds_on: [ADR-0003, ADR-0008]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-0009 Fuzzing with proptest

## Context

The user wants a fuzz test. The machine has only the stable toolchain;
`cargo-fuzz` (libFuzzer) needs nightly and an extra install.

## Decision

Fuzz the headless `Editor` with **proptest**, on stable, in `tests/fuzz.rs`:

- A strategy generates random sequences of `InputEvent`s: pointer down/move/up
  with any button, keys with any modifiers, scrolls, tool and colour changes,
  undo/redo/copy/paste bursts. Coordinates include edge values: `0`, huge
  magnitudes, `NaN`, `±∞`.
- After every event the fuzzer checks the invariants listed in
  [[Architecture]] (no panic, finite coordinates, zoom bounds, valid
  selection), and at the end checks undo-all/redo-all symmetry.
- Normal `cargo test` runs a quick pass (default case count). Long runs:
  `scripts/check.sh --fuzz` (`PROPTEST_CASES=20000`).
- Failures are shrunk by proptest to a minimal event sequence; the regression
  file (`tests/fuzz.proptest-regressions`) is committed so it is re-run forever.

## Alternatives considered

- **cargo-fuzz / libFuzzer** — coverage-guided and stronger at finding deep
  bugs, but needs nightly. Can be added later via `arbitrary` behind a
  separate `fuzz/` crate as an amending ADR without touching this harness.
- **Hand-written random loop** — no shrinking, so failures are hard to read.

## Consequences

- proptest is a dev-dependency only; the release binary is unaffected.
- Every module that sanitizes input (camera, smoothing, shapes) must handle
  non-finite floats explicitly.

## Rollback plan

Remove `tests/fuzz.rs` and the dev-dependency; no runtime code depends on it.
