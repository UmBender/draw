---
id: T14
title: Performance and release validation
status: todo
wave: 10
branch: task/T14-release
depends_on: [T12, T13, T15, T16, T17, T18, T19, T20]
adrs: ["[[ADR-0006 Redraw on demand]]", "[[ADR-0007 Anti-tremor pipeline]]"]
feature: "[[Contest cheat sheet]]"
tutorial: "[[14 Profiling and shipping a release build]]"
tags: [task]
---

# T14 Performance and release validation

## Goal

Prove the app is fast and solid on the target laptop, tune smoothing for the
user's hand, and write the contest cheat sheet.

## Spec

- **AC-1** — Perf tests (`#[ignore]`, run with `cargo test --release -- --ignored perf`):
  10 000 shapes → hit-test of one point < 1 ms; culling pass < 1 ms; 1 000-point
  stroke finish (RDP) < 1 ms. *Tests:* `tests/perf.rs::*`.
- **AC-2** — Release binary builds with the profile from ADR-0014; record size and
  startup time in the log.
- **AC-3** — Manual checklist on the target (Wayland, i5-8th gen): every
  [[Keymap]] binding, every tool, zoom limits, 5 000-shape session stays smooth,
  idle CPU ≈ 0 %. Results in the log.
- **AC-4** — Smoothing levels tuned with the user; changed values go into an ADR
  amending [[ADR-0007 Anti-tremor pipeline]].
- **AC-5** — `features/Contest cheat sheet.md`: one-page printable shortcut list.

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
