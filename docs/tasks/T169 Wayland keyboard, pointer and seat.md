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

Decisions: [[ADR-T169-1 Wayland input decoding and keymap reading]]. Tests are in `crates/draw-sys/tests/wayland_input.rs`.

- **AC-1** — `wl_seat` keeps at most one keyboard and pointer and follows capability changes. Tests: `seat_devices_first_capabilities_create_keyboard_and_pointer`, `seat_devices_capabilities_resent_keep_devices`, `seat_devices_lost_capability_releases_device`, `seat_devices_never_create_touch`, integration `wayland_seat_has_one_keyboard_and_one_pointer`.
- **AC-2** — Keyboard: keymap via xkbcommon, `RawKey`/`RawMods`, repeat from `repeat_info`, no doubled keys after a long idle. Tests: `keymap_usable_*`, `xkb_garbage_keymap_is_none`, `raw_key_*`, `repeats_due_*`, `regression_T45_capabilities_resent_keeps_one_keyboard`, `regression_T45_stalled_loop_drops_surplus_repeats`, `regression_T79_no_keymap_is_unusable`, integration `wayland_keyboard_typed_text_arrives`, `wayland_keyboard_modifiers_arrive`, `wayland_keyboard_focus_is_reported`.
- **AC-3** — Pointer: motion, buttons, wheel (discrete and smooth). Tests: `pointer_position_*`, `raw_button_*`, `wheel_*`, integration `wayland_pointer_motion_arrives`, `wayland_pointer_buttons_arrive`, `wayland_pointer_wheel_arrives`. The cursor shape stays the compositor's default (ADR-T169-1).
- **AC-4** — Every decoder of bytes from the server (keymap, keysym, fixed point, repeat) never panics on arbitrary input: `fuzz_keymap_decoder_never_panics`, `fuzz_pure_decoders_never_panic` (xorshift, no new dependency, ADR-T169-1).

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
