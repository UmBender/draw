---
id: T14
title: Performance and release validation
status: in-progress
wave: 10
branch: task/T14-release
depends_on: [T12, T13, T15, T16, T17, T18, T19, T20]
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-0007 Anti-tremor pipeline]]", "[[ADR-T14-1 Performance budgets]]"]
feature: "[[Contest cheat sheet]]"
tutorial: "[[14 Profiling and shipping a release build]]"
tags: [task]
---

# T14 Performance and release validation

## Goal

Prove the app is fast and solid on the target laptop, tune smoothing for the
user's hand, and write the contest cheat sheet.

## Spec

Budgets, benchmark document and how to run: [[ADR-T14-1 Performance budgets]].
Perf tests are `#[ignore]`d and run in release mode on the target:
`cargo test --release --test perf -- --ignored --nocapture --test-threads=1`.
The benchmark document has 10 000 shapes, an even mix of strokes, lines,
arrows, filled + numbered rectangles and ellipses, and 8 × 8 grids with
indices and filled cells (T16–T22 features included).

- **AC-1** — Perf budgets on the target, median of 101 runs:
  - hit-test of one point that misses everything (whole document scanned)
    < 1 ms — `tests/perf.rs::perf_hit_test_miss_with_10k_shapes_is_under_1ms`;
  - eraser outline scan at random points < 1 ms —
    `perf_eraser_scan_with_10k_shapes_is_under_1ms`;
  - culling pass (`cull_rect` + `is_visible` over every shape) < 1 ms —
    `perf_culling_pass_with_10k_shapes_is_under_1ms`;
  - 1 000-point stroke finish (High smoothing, RDP) < 1 ms —
    `perf_stroke_finish_with_1000_points_is_under_1ms`; RDP on 1 000 raw
    points < 1 ms — `perf_rdp_on_1000_raw_points_is_under_1ms`;
  - one snapped box-drag move (grid + smart snap, targets rebuilt from the
    document) < 4 ms — `perf_snap_drag_with_10k_shapes_is_under_4ms`.
- **AC-2** — `cargo build --release` succeeds with the profile from
  [[ADR-0014 Minimal dependencies and no unsafe]] (`lto = "fat"`,
  `codegen-units = 1`, `panic = "abort"`, `strip = true`). Binary size and
  startup time (launch → window shown) recorded in the *Log*.
- **AC-3** — Manual checklist on the target (KDE Wayland, i5-8250U), results
  in the *Log*: every [[Keymap]] binding; every tool (pen, line, arrow, rect,
  ellipse, grid, select, eraser both modes, bucket on shapes and cells, pan);
  helpers (smart snap incl. outline snap, grid snap, numbering, grid size
  flyout, grid indices); zoom limits (0.05× and 20×) without artefacts; MSAA
  edges look smooth; a 5 000-shape session stays smooth; idle CPU ≈ 0 %.
- **AC-4** — Smoothing levels tried by the user on the target; changed
  values go into `src/core/smoothing.rs` and an ADR `ADR-T14-2` amending
  [[ADR-0007 Anti-tremor pipeline]]. If no value changes, the *Log* says so
  and no ADR is written. Existing `core::smoothing` tests must stay green.
- **AC-5** — `features/Contest cheat sheet.md`: one printable page listing
  every binding of `core::keymap::BINDINGS`, grouped by purpose.

## Out of scope

New features.

## Files owned

`tests/perf.rs`, `src/core/smoothing.rs` (constants only, if tuned),
`docs/features/Contest cheat sheet.md`, new ADRs `ADR-T14-*`.

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] behaviour (tuning) · [ ] quality · [ ] docs

## Learning path

Step 14 — requires steps 12–13 (validates the features of T15–T19 too).

## Log

- 2026-10-01 — Re-planned to wave 10 so release validation covers the
  helper features (T15–T19); AC-1/AC-3 must include grids, labels, snapping
  and MSAA.
