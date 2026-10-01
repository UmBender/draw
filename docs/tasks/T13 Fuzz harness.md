---
id: T13
title: Fuzz harness
status: todo
wave: 6
branch: task/T13-fuzz
depends_on: [T09, T10, T11]
adrs: ["[[ADR-0009 Fuzzing with proptest]]"]
feature:
tutorial: "[[13 Property-based testing and fuzzing]]"
tags: [task]
---

# T13 Fuzz harness

## Goal

A proptest-based fuzzer that drives the whole `Editor` with random input
sequences and checks the invariants in [[Architecture]].

## Spec

- **AC-1** — Strategy `arb_event()` covers every `InputEvent` variant, every key,
  every modifier combination; coordinates from a mix of normal range, huge values,
  `0`, `NaN`, `±∞`. Strategy `arb_session()` = 1..500 events, biased towards
  realistic gestures (down → moves → up). *Test:* `fuzz::strategies_generate_all_variants`.
- **AC-2** — `editor_never_panics_and_keeps_invariants`: after every event —
  all shape coordinates finite, zoom in range, selection ⊆ document ids, no panic.
- **AC-3** — `undo_all_then_redo_all_is_symmetric`: at session end, undo until
  `!can_undo()` → document empty *if* history was not capped; redo all → same
  shapes as before undo.
- **AC-4** — Smoothing fuzz: `smoother_never_panics_on_arbitrary_points` and output
  points are finite.
- **AC-5** — Any failure found while writing the harness is fixed in the owning
  module *in this branch* with a regression test there, and the shrunk case is kept in
  `tests/fuzz.proptest-regressions`. Each fix is its own `fix(T13): …` commit.
- **AC-6** — `scripts/check.sh --fuzz` runs 20 000 cases green.

## Out of scope

cargo-fuzz/libFuzzer (see [[ADR-0009 Fuzzing with proptest]]).

## Files owned

`tests/fuzz.rs`, `tests/fuzz.proptest-regressions`; bug fixes in any `src/core`
module (exception to *Files owned*, logged in the task note).

## Subtasks (one commit each)

- [ ] spec · [ ] tests (strategies + invariant checks) · [ ] fixes (one commit per bug) · [ ] quality · [ ] docs

## Learning path

Step 13 — requires steps 9–11.

## Log
