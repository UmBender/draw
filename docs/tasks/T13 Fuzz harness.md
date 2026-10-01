---
id: T13
title: Fuzz harness
status: in-progress
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

All tests live in `tests/fuzz.rs` (integration test, public API only).

- **AC-1** — Strategy `arb_event()` covers every `InputEvent` variant, every key,
  every modifier combination; coordinates from a mix of normal range, huge values,
  `0`, `NaN`, `±∞`. Strategy `arb_session()` = 1..500 events, biased towards
  realistic gestures (down → moves → up) and command bursts (undo/redo/copy/paste).
  *Test:* `strategies_generate_all_variants` — samples 20 000 events and asserts
  every variant, every `Key`, every `PointerButton`, all 8 modifier combinations and
  each special coordinate class (`0`, `NaN`, `±∞`, `|x| > 1e30`) appear.
- **AC-2** — `editor_never_panics_and_keeps_invariants` (proptest over
  `arb_session()`): after every event — all shape coordinates finite
  (`Shape::is_finite`), zoom in `[ZOOM_MIN, ZOOM_MAX]`, camera offset finite,
  selection ⊆ document ids with no duplicates, no panic.
- **AC-3** — `undo_all_then_redo_all_is_symmetric` (proptest): at session end,
  apply `Cancel`, snapshot shapes (ids + shapes), undo until `!can_undo()` →
  document empty *if* fewer than `HISTORY_LIMIT` undos were possible (history not
  capped); redo the same number of times → `!can_redo()` and shapes equal the
  snapshot.
- **AC-4** — `smoother_never_panics_on_arbitrary_points` (proptest): any
  `SmoothingParams` (incl. non-finite), any scale, any points → no panic, `points()`
  and `finish()` are finite. `simplify_rdp_never_panics_on_arbitrary_points`: output
  finite and a subsequence of the finite input.
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
