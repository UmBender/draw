---
title: Property-based testing and fuzzing
step: 13
requires: ["[[09 Building drawing tools]]", "[[10 Selection, clipboard and fill]]", "[[11 Keymaps and gesture macros]]"]
feature:
code: ["tests/fuzz.rs", "tests/fuzz.proptest-regressions", "scripts/check.sh"]
tags: [tutorial]
---

# Property-based testing and fuzzing

> **Learning path step 13.** Requires: steps 9–11 · Next: step 14 (performance)

## Why this matters here

Unit tests check the cases you thought of. A drawing app gets the cases you
did not: a touchpad sending `NaN`, a window resized to zero, `Ctrl+Z` while a
drag is still going, a paste at a cursor a billion pixels away. In a contest
there is no time to restart a crashed scratchpad, so the editor must survive
*any* input sequence. A fuzzer finds those sequences for us.

## The concept

**Property-based testing** replaces "for this input, expect this output" with
"for *every* input, this property holds". A *strategy* describes how to
generate inputs; the framework generates hundreds of them and checks the
property for each.

When a case fails, the framework **shrinks** it: it keeps simplifying the input
(shorter vectors, smaller numbers) while it still fails. A failing 400-event
session usually shrinks to 2–5 events you can read.

**Fuzzing** is the same idea aimed at robustness: generate hostile input and
check *invariants* — properties that must hold after every step, whatever
happened before:

```
for event in random_session:
    editor.handle(event)          // must not panic
    assert invariants(editor)     // finite shapes, zoom range, valid selection
at the end:
    undo all → empty;  redo all → identical document
```

Coverage-guided fuzzers (libFuzzer) also watch which branches run and mutate
inputs to reach new ones. draw uses proptest instead, on stable Rust
([[ADR-0009 Fuzzing with proptest]]): no coverage feedback, so the
*strategies* must be biased to reach deep states by themselves.

## How draw implements it

Everything lives in `tests/fuzz.rs` and uses only the public API, like the
shell does.

**Strategies.** `arb_coord()` mixes on-screen values with `0`, `NaN`, `±∞`,
huge magnitudes and any bit pattern, weighted so most coordinates are normal.
`arb_event()` builds every `InputEvent` variant from it, with every `Key`,
`PointerButton` and all 8 modifier combinations.

A session of uniformly random events would almost never draw anything: a
`PointerDown` followed by a `KeyUp` and a `Resize` is not a stroke. So
`arb_session()` is built from *chunks*:

- `arb_gesture()` — down, a wobbly path of small steps (rarely a wild point),
  usually an up; mostly left button with no modifiers;
- `arb_command_burst()` — key chords picked straight from `keymap::BINDINGS`
  (tools, undo/redo, copy/paste, view);
- `arb_space_drag()` — `Space` held around a drag;
- single raw events and resizes.

Chunks are flattened and capped at 500 events.

**Is the harness any good?** A fuzzer that passes because it never reaches
interesting states proves nothing. `strategies_generate_all_variants` samples
20 000 events and checks every variant, key, button, modifier set and special
coordinate class appears. `sessions_reach_deep_editor_states` runs 200
sessions and requires that enough of them create shapes, select, zoom and
leave something to redo. Raising the left-button bias took shape-creating
sessions from 36 % to 75 %.

**Invariants.** `check_invariants` runs after every event:
`Shape::is_finite` for every shape, zoom in `[ZOOM_MIN, ZOOM_MAX]`, a finite
camera offset, and a selection whose ids exist and are unique.

**Undo symmetry.** `undo_all_then_redo_all_is_symmetric` cancels any gesture,
redoes leftover redo entries, snapshots `(id, shape)` pairs, undoes until
`!can_undo()` (expecting an empty document unless the history hit
`HISTORY_LIMIT`), redoes the same number of times and compares.

The first version of this test forgot the "redo leftovers" step. Proptest
shrank the failure to *draw a dot, press `Ctrl+Z`*: redoing "everything"
then went one step past the snapshot. The bug was in the test, not the editor,
but the shrunk case still lives in `tests/fuzz.proptest-regressions` and is
replayed on every run.

**Smoothing.** `smoother_never_panics_on_arbitrary_points` feeds `Smoother`
arbitrary parameters (negative, `NaN`, `∞`), scales and points;
`simplify_rdp_never_panics_on_arbitrary_points` checks the output is a finite,
in-order subsequence of the input.

**Running it.** `cargo test` runs 256 cases per property (about a second).
`scripts/check.sh --fuzz` runs `PROPTEST_CASES=20000` in release mode
(about 30 s). The `config()` helper keeps `PROPTEST_CASES` working and pins the
regression file next to `tests/fuzz.rs`.

## Try it

1. Break an invariant on purpose: in `Camera::zoom_at`
   (`src/core/camera.rs`) remove the clamp, run `cargo test --test fuzz`,
   and read the shrunk session. Look at
   `tests/fuzz.proptest-regressions` — a new `cc …` line appeared. Revert both.
2. Set the gesture's left-button weight in `arb_gesture()` to `0` and watch
   `sessions_reach_deep_editor_states` fail: the harness has gone shallow.
3. Add an invariant: "every selected shape is also returned by
   `Editor::selection_bounds`" — and run 20 000 cases.

## Further reading

- [proptest book](https://proptest-rs.github.io/proptest/) — strategies,
  shrinking, failure persistence.
- John Hughes, *QuickCheck Testing for Fun and Profit* — the origin of
  property-based testing.
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/) — cargo-fuzz and
  coverage-guided fuzzing, the alternative in ADR-0009.
