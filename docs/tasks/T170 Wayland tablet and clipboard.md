---
id: T170
title: Wayland tablet and clipboard
status: todo
wave: 72
branch: task/T170-wayland-tablet-and-clipboard
depends_on: [T169]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T170 Wayland tablet and clipboard

## Goal

Slice 3 of [[T143 Wayland backend with window, keyboard, pointer, tablet and clipboard]]: `zwp_tablet_v2` pen input (pressure, tilt, eraser, proximity) and the clipboard (text and `image/png`) through `wl_data_device`.

## Known facts (2026-11-15)

- Split out of T143 by the owner. Builds on [[T169 Wayland keyboard, pointer and seat]].
- Patches to port (`vendor/miniquad/PATCHED.md`): tablet pressure (T35), the pointer dropped while a tool is in proximity ([[ADR-T40-1 Tablet stream wins over copied wl_pointer input]], `pointer_shadowed_by_pen`), barrel-button mapping (`fix/pen-barrel-takeover`), tip bounce (`fix/pen-tip-bounce`).
- `tablet-unstable-v2.xml` is added to the generator's inputs ([[T168 Wayland connection, window and protocol states]]).
- Headless sway has no virtual tablet: tablet tests decode recorded event sequences; the owner checks a real tablet by hand at review.
- Clipboard transfers pass file descriptors (pipes): an `unsafe` primitive under [[ADR-T127-2 The unsafe policy]]. `wl-copy`/`wl-paste` drive the integration tests.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Tablet: tools, proximity, tip, pressure, tilt, eraser, barrel buttons, frames to pen samples; pointer shadowed while a tool is in proximity. Tests on recorded sequences; regressions for T35, T40 and the pen fixes.
- **AC-2** — Clipboard: read and write text, write `image/png` (T115), large transfers. Integration tests with `wl-copy`/`wl-paste`.
- **AC-3** — Decoders fuzzed (proptest).
- **AC-4** — Hand check with a tablet (owner, at review), recorded in the Log.

## Out of scope

- `main` and e2e ([[T171 Wayland platform in main and the e2e suite]]); primary selection.

## Files owned

- `crates/draw-sys/src/wayland/**`, `crates/draw-sys/protocols/**`, `crates/draw-sys/tests/wayland*.rs`, `crates/draw-sys/tests/common/sway.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T170): specify the slice`
- [ ] tests — `test(T170): add failing tests for the tablet and clipboard`
- [ ] models — `feat(T170): add the tablet and transfer types`
- [ ] behaviour — `feat(T170): implement the tablet and clipboard`
- [ ] quality — `chore(T170): pass clippy and rustfmt`
- [ ] tablet by hand — owner, recorded in the Log

## Log

- 2026-11-15 — split from T143 by the owner.
