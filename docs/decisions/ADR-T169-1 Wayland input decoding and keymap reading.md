---
id: ADR-T169-1
title: Wayland input decoding, keymap reading and test fuzz without a new dependency
status: accepted
kind: decision
date: 2026-11-16
task: T169
builds_on: [ADR-T44-1, ADR-T79-1, ADR-T127-2, ADR-T142-1]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---
# ADR-T169-1 Wayland input decoding, keymap reading and test fuzz without a new dependency

## Context
T169 adds `wl_seat`, `wl_keyboard` and `wl_pointer` to the Wayland backend. The task note suggested mapping the keymap file descriptor with `mmap`. `draw-sys` has no `proptest` dev-dependency and `Cargo.toml` is not owned by this task. The compositor draws the default cursor for a client that never sets one, and `wp_cursor_shape_v1` is not among the checked-in protocol files.

## Decision
- **Keymap bytes are read, not mapped.** The keymap fd is wrapped in an `OwnedFd` (closed on every path), read with `FileExt::read_at` at offset 0 for `size` bytes (capped at 16 MiB), cut at the first NUL and given one NUL for `xkb_keymap_new_from_string`. This needs one `unsafe` call (`OwnedFd::from_raw_fd`) and no `mmap`/`munmap`. A format other than `XKB_V1`, size 0, a failed read, a null keymap or a null state leave the last good pair in place (ADR-T79-1).
- **Pure decisions** live in `wayland::input` and are tested without a compositor: `keymap_usable`, `repeats_due` (capped per poll by `MAX_REPEATS_PER_POLL`), `raw_key_from_keysym`, `raw_button_from_evdev`, `pointer_position`, `wheel_of_frame`, and `SeatDevices` (the T44/T45 rule: one keyboard and one pointer, a capability announced again keeps its device).
- **Key repeat is client side**, from `repeat_info`, computed at each poll from the held key's press time and the number of repeats already sent. A stalled loop drops the surplus repeats instead of replaying them (T45).
- **Wheel**: one `RawEvent::Wheel` per `wl_pointer.frame`. `axis_value120` wins over `axis` when both arrive in the frame (10 units per notch, like the smooth value); the vertical sign is flipped (Wayland scrolls down positive).
- **Cursor**: no `set_cursor`; the compositor's default cursor shows. A cursor shape request needs `wp_cursor_shape_v1`, left for T171 if the owner wants it.
- **Fuzz** (AC-4): a deterministic xorshift generator in the test, not `proptest` (a dev-dependency would need the owner of `Cargo.toml`).
- The protocols `wl_seat`, `wl_pointer`, `wl_keyboard` join the generated set.

## Alternatives considered
- **`mmap` the fd** — more `unsafe` (two primitives, a mapping to release) for a few KiB the compositor already shares.
- **Add `proptest` to `draw-sys`** — outside the files owned.

## Consequences
The safe keymap reader is the only fd handling. Repeat needs the loop to poll often; the app polls every frame.

## Rollback plan
Revert the T169 commits.
