---
id: T169
title: Wayland keyboard, pointer and seat
status: todo
wave: 71
branch: task/T169-wayland-keyboard-pointer-and-seat
depends_on: [T168]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T169 Wayland keyboard, pointer and seat

## Goal

Slice 2 of [[T143 Wayland backend with window, keyboard, pointer, tablet and clipboard]]: `wl_seat`, the keyboard through `xkbcommon` (loaded with `dlopen`) with key repeat, and the pointer with buttons, wheel and cursor, all as `RawEvent`s.

## Known facts (2026-11-15)

- Split out of T143 by the owner. Builds on [[T168 Wayland connection, window and protocol states]] (generator, window, sway test helper).
- Patches to port (`vendor/miniquad/PATCHED.md`): seat devices that follow the capabilities ([[ADR-T44-1 Wayland seat devices follow the announced capabilities]], `device_change`, `has_release_request` already in `draw-sys::platform`); doubled keys after a long idle (T45); the wheel fix (`fix/wayland-wheel`); the missing keymap fix (`fix/wayland-no-keymap`).
- The keymap arrives as a file descriptor to `mmap`: a shared-memory `unsafe` primitive under [[ADR-T127-2 The unsafe policy]].
- Headless sway can drive the keyboard with `wtype` and the pointer with `swaymsg seat - cursor …`.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `wl_seat` keeps at most one keyboard and pointer and follows capability changes. Tests on the decision functions and an integration test.
- **AC-2** — Keyboard: keymap via xkbcommon, `RawKey`/`RawMods`, repeat from `repeat_info`, no doubled keys after a long idle. Integration tests with `wtype`; `regression_T45_*`.
- **AC-3** — Pointer: motion, buttons, wheel (discrete and smooth), cursor shape. Integration tests; regressions for the Wayland pointer bugs of T35/T40 that apply.
- **AC-4** — Every decoder of bytes from the server (keymap, events) is fuzzed (proptest).

## Out of scope

- Tablet and clipboard ([[T170 Wayland tablet and clipboard]]); `main` and e2e ([[T171 Wayland platform in main and the e2e suite]]); IME.

## Files owned

- `crates/draw-sys/src/wayland/**`, `crates/draw-sys/protocols/**`, `crates/draw-sys/tests/wayland*.rs`, `crates/draw-sys/tests/common/sway.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T169): specify the slice`
- [ ] tests — `test(T169): add failing tests for the keyboard and pointer`
- [ ] models — `feat(T169): add the seat and keymap types`
- [ ] behaviour — `feat(T169): implement the keyboard, pointer and seat`
- [ ] quality — `chore(T169): pass clippy and rustfmt`

## Log

- 2026-11-15 — split from T143 by the owner.
