---
id: T13
title: Fuzz harness
status: done
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
  apply `Cancel`, redo any leftover redo entries, snapshot shapes (ids + shapes), undo until `!can_undo()` →
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

- [x] spec · [x] tests (strategies + invariant checks) · [x] fixes (one commit per bug) — none needed · [x] quality · [x] docs

## Learning path

Step 13 — requires steps 9–11.

## Log

- **Spec** — named a test per AC; AC-3 now redoes leftover redo entries
  before the snapshot (found by the first fuzz run, see below). Added a
  reachability check (`sessions_reach_deep_editor_states`) and
  `session_strategy_respects_length_bounds` beyond the spec.
- **Tests** — `tests/fuzz.rs`. The harness is the deliverable, so the commit
  went green once the harness itself was correct; there was no production code
  to be red against.
- **Models / Behaviour** — skipped: the task adds no types or runtime code.
- **Fixes (AC-5)** — none. 20 000 cases (`check.sh --fuzz`) plus two
  independent 60 000-case runs of the editor properties found no failure in
  `src/core`. The only failure was in the harness (redo leftovers at session
  end); its shrunk case is kept in `tests/fuzz.proptest-regressions`. No
  `src/core` file was touched.
- **Tuning** — left-button and no-modifier bias in `arb_gesture()` raised
  shape-creating sessions from 72/200 to 149/200.
- **Quality** — clippy `struct_excessive_bools` in the test; pinned the
  regression file with `FileFailurePersistence::WithSource`.
- **Result** — `scripts/check.sh --fuzz` green (20 000 cases, ~30 s release).
