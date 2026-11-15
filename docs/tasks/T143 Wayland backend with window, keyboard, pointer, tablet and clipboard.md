---
id: T143
title: Wayland backend with window, keyboard, pointer, tablet and clipboard
status: done
wave: 70
branch: task/T143-wayland-backend-window-keyboard-pointer-
depends_on: [T168, T169, T170, T171]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T143 Wayland backend with window, keyboard, pointer, tablet and clipboard

## Goal

A native Wayland platform that replaces the patched miniquad's Wayland code.

## Known facts (2026-11-12)

- The patches to port are listed in `vendor/miniquad/PATCHED.md` (tablet pressure, seat devices that follow capabilities, doubled keys after a long idle).
- Type states for the surface protocol: `Created -> Configured -> Mapped`.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Window creation and resize with scale, frame callbacks, keyboard through xkbcommon with repeat, pointer and `zwp_tablet_v2` pressure, clipboard text and PNG through `wl_data_device`.
  Integration tests against a headless sway; one regression test for each bug of T35, T40, T44, T45 that the new code could reintroduce.
- **AC-2** — Calling a request out of protocol order does not compile.
  Compile-fail tests.
- **AC-3** — The e2e suite passes on Wayland with this backend selected.
  `scripts/e2e.sh --backend wayland`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-sys/src/wayland/**`
- this note

## Subtasks (one commit each)

- [x] spec — moved to the slices
- [x] tests — moved to the slices
- [x] models — moved to the slices
- [x] behaviour — moved to the slices
- [x] quality — moved to the slices

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-15 — closed by the owner before the batch (as large as T156 and T164, which blocked): split into [[T168 Wayland connection, window and protocol states]] (generated protocol code checked in, window, EGL on it, frame callbacks, type states; AC-2), [[T169 Wayland keyboard, pointer and seat]], [[T170 Wayland tablet and clipboard]] (AC-1 with T169) and [[T171 Wayland platform in main and the e2e suite]] (AC-3).
