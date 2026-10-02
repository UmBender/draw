---
title: Profiling and shipping a release build
step: 14
requires: ["[[12 The app loop, idle redraw and UI]]", "[[13 Property-based testing and fuzzing]]"]
feature: "[[Contest cheat sheet]]"
code: ["tests/perf.rs", "Cargo.toml", "src/core/smoothing.rs"]
tags: [tutorial]
---

# Profiling and shipping a release build

> **Learning path step 14.** Requires: steps 12–13 · Next: step 15
> (anti-aliasing). Validates the features of steps 15–22 too.

## Why this matters here

"Fast" is a claim, and claims need numbers. Before a contest you want to
know that a big sketch will not stutter, that the binary starts instantly,
and that the anti-tremor filter suits *your* hand. This step turns those
into measurements and one decision ([[ADR-T14-2 Low smoothing by default]]).

## The concept

**Budgets, not benchmarks.** A benchmark answers "how fast is it?"; a
budget answers "is it fast enough?". For an interactive app the reference
is the frame: at 60 Hz you have 16.7 ms to handle input *and* draw. Any
per-event operation that scales with the document gets a slice of that:

```
frame 16.7 ms
├── input handling   hit-test < 1 ms, snapping < 4 ms
├── culling          < 1 ms
└── draw + present   the rest
```

**Measure the median.** One timing is noise: the scheduler, a CPU frequency
change or a page fault can double it. Run the operation many times, sort the
durations and take the middle one: a hiccup moves the mean but not the
median.

**Measure the optimised build.** Debug builds can be 10–50× slower, so their
numbers say nothing about what you ship.

**Separate what can be automated from what cannot.** Timings of pure code
run headless in a test; whether a 5 000-shape session *feels* smooth, or
whether smoothing lags for your hand, needs a person and goes in a manual
checklist.

## How draw implements it

- `tests/perf.rs` builds a fixed 10 000-shape document with a seeded
  xorshift generator (`Rng`), so every run times the same data. It mixes all
  six shape kinds, including numbered boxes and grids with indices and
  filled cells, so newer features are measured too.
- `median_time(setup, op)` runs `op` 5 times to warm caches, then 101 timed
  times, keeping the input construction (`setup`) outside the clock.
  `std::hint::black_box` stops the optimiser from deleting work whose result
  is unused.
- Each test is `#[ignore]`d (too slow and machine-dependent for every
  `cargo test`) and returns early when `cfg!(debug_assertions)` is set. Run
  them with:

  ```sh
  cargo test --release --test perf -- --ignored --nocapture --test-threads=1
  ```

  One thread, so tests do not compete for cores: running them in parallel
  roughly doubled some medians.
- The results and budgets are in [[ADR-T14-1 Performance budgets]]. The
  snapped drag rebuilds `snap::Targets` from the whole document on every
  pointer move; at ≈ 1 ms it is the slowest per-event path, so it got its
  own 4 ms budget instead of a flaky 1 ms one.
- The release profile in `Cargo.toml` (`lto = "fat"`, `codegen-units = 1`,
  `panic = "abort"`, `strip = true`) gives a ≈ 950 KiB binary whose window
  appears in ≈ 0.1 s.

## What the manual check found

The headless numbers were all well inside budget, yet a 5 000-shape session
felt clunky. Pure-code timings do not include drawing: every change
re-tessellates and re-uploads the whole scene (see
[[Rendering performance options]]). This is the classic lesson of
profiling: **measure the whole loop, not only the parts that are easy to
test**. The next step there is a frame timer around the render and
`perf record`, then the two-layer cache from that note.

## Try it

1. Run the perf tests and note the medians. Then change `RUNS` to `1` and
   run them five times: see how much a single timing jumps around.
2. Run them without `--test-threads=1` and compare.
3. Double `SHAPES`: which operations scale linearly, and which budget breaks
   first?
4. Build with `cargo build --release` and with `cargo build`, and compare the
   binary sizes in `target/`.

## Further reading

- The Rust Performance Book — <https://nnethercote.github.io/perf-book/>
- `std::hint::black_box` documentation.
- Brendan Gregg, *Linux perf examples* — <https://www.brendangregg.com/perf.html>
