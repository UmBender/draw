---
id: T144
title: X11 backend with window, keyboard, pointer, XInput2 pressure and selections
status: todo
wave: 70
branch: task/T144-x11-backend-window-keyboard-pointer-xinp
depends_on: [T142, T145]
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

- [ ] spec — `docs(T144): specify the X11 backend`
- [ ] tests — `test(T144): add failing tests for the X11 platform`
- [ ] models — `feat(T144): add the protocol types`
- [ ] behaviour — `feat(T144): implement the X11 platform`
- [ ] quality — `chore(T144): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
