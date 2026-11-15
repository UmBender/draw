---
id: T144
title: X11 backend with window, keyboard, pointer, XInput2 pressure and selections
status: done
wave: 70
branch: task/T144-x11-backend-window-keyboard-pointer-xinp
depends_on: [T175, T176, T177, T178]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T144 X11 backend with window, keyboard, pointer, XInput2 pressure and selections

## Goal

A native X11 platform that keeps every X11 user working.

## Known facts (2026-11-12)

- Owner: X11 users must keep working. The vendored miniquad's X11 pressure code and the selection handling (including the PNG selection of [[T115 Copy the selection as an image]]) must be reproduced.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Window creation and resize, keyboard with repeat detection and layouts, pointer, XInput2 pen pressure, clipboard and primary selections (text and `image/png`) with `INCR`.
  Integration tests against a private Xvfb; regression tests for the X11 bugs found in T28, T35, T45 and T115.
- **AC-2** — A window-manager close, `Ctrl-C` and the clean exit of [[T48 Clean exit on Ctrl-C]] behave as before.
- **AC-3** — The e2e suite passes on X11 with this backend selected.
  `scripts/e2e.sh --backend x11`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-sys/src/x11/**`
- this note

## Subtasks (one commit each)

- [x] spec — moved to the slices
- [x] tests — moved to the slices
- [x] models — moved to the slices
- [x] behaviour — moved to the slices
- [x] quality — moved to the slices

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-15 — closed by the owner before the batch (as large as T143): split into [[T175 X11 connection, window and GLX context]], [[T176 X11 keyboard, pointer and XInput2 pressure]], [[T177 X11 selections and clean exit]] (AC-1, AC-2) and [[T178 X11 platform in main and the e2e suite]] (AC-3). Xlib rather than pure xcb, because GLX takes an Xlib `Display*`.
