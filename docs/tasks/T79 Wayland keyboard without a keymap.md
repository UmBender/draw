---
id: T79
title: Wayland keyboard without a keymap
status: todo
wave: 35
branch: fix/wayland-no-keymap
depends_on: [T75]
adrs: []
feature: "[[Native Wayland]]"
tutorial: ""
tags: [task]
---

# T79 Wayland keyboard without a keymap

## Goal

A `wl_keyboard.keymap` event with no usable keymap no longer takes draw
down. draw keeps running, keeps the last keymap it had, and leaks no file
descriptor.

## Known facts (2026-11-08)

- **Symptom:** `keyboard_handle_keymap`
  (`vendor/miniquad/src/native/linux_wayland.rs`) maps the event's `fd`
  and does `assert!(map_shm != libc::MAP_FAILED)`. A `no_keymap` event has
  size 0, `mmap` fails, and the assert panics inside a Wayland callback.
- **How it was found:** [[T75 End-to-end test suite]], see
  [[ADR-T75-1 Headless sway session for end-to-end tests]]. Each `wtype`
  is its own virtual keyboard. When the active one exits, wlroots clears
  the seat's keyboard and sway adopts the next new device before its
  keymap arrives, so the client gets `no_keymap` with size 0.
- **The suite avoids it today,** so no test covers it. A real seat can
  still hit it: anything that creates and destroys virtual keyboards
  (`wtype`, on-screen keyboards, remote-desktop tools).
- **The handler ignores `format`.** The same mapping and parsing runs for
  any format, and only `XKB_V1` is a keymap it can read.
  `xkb_keymap_new_from_string` can also return null on a bad keymap, and
  the handler stores that without checking.
- Patches to the vendored crate are marked in the source and listed in
  `vendor/miniquad/PATCHED.md` ([[ADR-T35-1 Vendored miniquad with a pen tablet patch]]).

## Proposal (settle in the spec step)

If the format is not `XKB_V1`, the size is 0 or `mmap` fails: close the
`fd`, record a backend diagnostic event, keep the current keymap and
state, and return. Apply the same to a null result from
`xkb_keymap_new_from_string`. Keys pressed while no keymap has ever
arrived must not crash either; check that path in the spec step.

## Spec

Draft; the spec step finalises it.

- **AC-1** — draw survives a `no_keymap` event in a real session, and
  typing works once a keymap arrives. *Test:* e2e case
  `tests/e2e/regression-T79-no-keymap/`, reproducing the `wtype` sequence
  above.
- **AC-2** — A failed keymap keeps the previous one: keys typed afterwards
  still translate correctly. *Test:* the same case's final snapshot.
- **AC-3** — The `fd` is closed on every path. *Test:* review in the task
  log, unless the spec finds a cheap automatic check.

## Out of scope

- Upstreaming to miniquad.
- The X11 backend.
- Changing how the e2e harness drives `wtype`, except what the regression
  case needs.

## Files owned

- `vendor/miniquad/src/native/linux_wayland.rs`
- `vendor/miniquad/PATCHED.md`
- `tests/e2e/regression-T79-no-keymap/` (new), `tests/e2e.rs` (only what
  the case needs)
- `docs/decisions/` (an ADR for the vendored change)

## Subtasks (one commit each)

- [ ] spec — `docs(T79): specify handling of a missing keymap`
- [ ] regression — `test(T79): add regression test for a missing keymap`
- [ ] behaviour — `fix(T79): survive a Wayland keyboard without a keymap`
- [ ] quality — `chore(T79): pass clippy and rustfmt`
- [ ] docs — `docs(T79): record the fix and close the task`

## Learning path

None planned.

## Log

- 2026-11-08 — created from the bug logged in T75 and ADR-T75-1.
