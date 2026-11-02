---
id: T44
title: One keyboard per Wayland seat
status: review
wave: 29
branch: fix/wayland-seat-devices
depends_on: [T28]
adrs: ["[[ADR-T44-1 Wayland seat devices follow the announced capabilities]]"]
feature: "[[Native Wayland (experimental)]]"
tutorial: ""
tags: [task]
---

# T44 One keyboard per Wayland seat

## Goal

With `DRAW_BACKEND=wayland`, every key press reaches the editor once, for
the whole life of the window, however often the compositor re-announces
the seat's capabilities. Today a typed letter can show up three times
(`aaa`), and one `Backspace` then removes all three.

## Known facts (2026-11-02, owner's Hyprland 0.56.2, Flatpak 1.4.3)

- Seen by the owner with `DRAW_BACKEND=wayland`: every typed character
  appeared three times in a row, first in text inside shapes, later in a
  plain text block. One `Backspace` removed all three, so the text really
  held three characters and `Backspace` also arrived three times: every
  keyboard event was delivered three times.
- Everything from `Collector::char_event` to `TextEdit::insert` inserts
  once per `char_event`, and the input subscriber is registered once
  (`src/shell/app.rs`). The duplication is below draw.
- `seat_handle_capabilities`
  (`vendor/miniquad/src/native/linux_wayland.rs:560`, upstream miniquad
  0.4.11 code) asks for a new `wl_keyboard` (and `wl_pointer`,
  `wl_touch`) and adds the listener **every** time `wl_seat.capabilities`
  arrives, without checking for or releasing the previous one. The
  protocol lets the compositor send `capabilities` again whenever the
  seat's devices change. After two re-sends there are three keyboards on
  the same listener, and each key comes in three times. A pointer
  re-created in the same way would duplicate button presses.
- **Not reproduced.** A `WAYLAND_DEBUG=client` capture of a long session
  showed `wl_seat.capabilities` only once, with many tablet tool
  `tool_added`/`removed` pairs, and nothing was duplicated. Virtual
  keyboards (`wtype`) did not make Hyprland re-send it either. What made
  it re-send it in the owner's session is unknown (device re-plug,
  suspend/resume, output hotplug and config reload are candidates). This
  fix is the protocol-correct handling and the leading explanation, not a
  confirmed one: if triplicated input comes back after it, the
  explanation is wrong.

## Proposal (settle in the spec step)

1. Make capability handling idempotent: create the keyboard, pointer
   or touch device only when the capability is announced and none exists
   yet; when a capability disappears, release (or destroy) the device and
   clear the handle. Mark it `draw patch (ADR-T35-1)` and list it in
   `vendor/miniquad/PATCHED.md`.
2. Testable core: a pure helper in the vendored crate (same pattern as
   `pen::pointer_shadowed_by_pen`, T40), e.g. "given whether a device
   exists and whether the capability is announced: create, release or
   keep", tested from draw. The FFI callback only applies its answer.
3. Leaving a released keyboard must also stop its key repeat
   (`KeyboardContext::repeated_key`, timer), as `keyboard_handle_leave`
   does.
4. Needs an ADR, or a short amendment to ADR-T35-1, recording that the
   vendored backend now also carries a seat fix that is not about pens.

## Spec

Decision: [[ADR-T44-1 Wayland seat devices follow the announced capabilities]].
Proposals 1 to 4 are taken; the ADR amends ADR-T35-1.

`miniquad::seat::device_change(exists, announced)` says what the Wayland
backend does with one seat device (keyboard, pointer or touch) when
`wl_seat.capabilities` arrives: `Create` it when the capability is
announced and no device exists, `Release` it when the capability is gone
and a device exists, `Keep` otherwise. `miniquad::seat::has_release_request(version)`
says whether the bound `wl_seat` version has the devices' `release`
request (3 and later); older seats only destroy the proxy.

`seat_handle_capabilities` applies `device_change` to each device. A
released keyboard also stops its key repeat; a released pointer also
drops its cursor-shape device and any pointer lock; a released touch
forgets its touch points. A pointer created after start-up gets a
cursor-shape device again.

- **AC-1** — a capability announced with no device creates one:
  `seat_device_change_announced_without_device_creates`.
- **AC-2** — a capability announced again with a device keeps it, so a
  re-sent `capabilities` adds no second keyboard:
  `seat_device_change_announced_with_device_keeps`.
- **AC-3** — a capability gone with a device releases it; gone with no
  device does nothing:
  `seat_device_change_withdrawn_with_device_releases`,
  `seat_device_change_withdrawn_without_device_keeps`.
- **AC-4** — replaying `capabilities(keyboard)` three times creates
  exactly one keyboard: `seat_capabilities_sent_three_times_create_one_device`.
- **AC-5** — replaying keyboard announced, gone, announced creates,
  releases and creates again, so a re-plugged keyboard is read:
  `seat_capability_regained_creates_a_new_device`.
- **AC-6** — `release` is used from seat version 3 on, and only destroy
  below it: `seat_release_request_from_version_3`.
- **AC-7** — owner check on Hyprland with `DRAW_BACKEND=wayland`: typing
  in a text block and in a shape gives each character once, `Backspace`
  removes one, the mouse cursor shape and pen still work (see *Log*).

## Out of scope

- X11 (one keyboard, no such callback).
- Finding the exact Hyprland trigger.
- Upstreaming the fix to miniquad.

## Files owned

- `vendor/miniquad/src/native/linux_wayland.rs`,
  `vendor/miniquad/src/seat.rs` (new), `vendor/miniquad/src/lib.rs`,
  `vendor/miniquad/PATCHED.md`
- `src/shell/input_map.rs` (tests)
- `docs/features/Native Wayland (experimental).md`
- new ADR `ADR-T44-*`

## Subtasks (one commit each)

- [x] spec — `docs(T44): specify one keyboard per Wayland seat acceptance criteria`
- [x] tests — `test(T44): add failing tests for one device per Wayland seat`
- [x] models — `feat(T44): add seat device_change and has_release_request signatures`
- [x] behaviour — `fix(T44): keep one keyboard, pointer and touch per Wayland seat`
- [x] quality — skipped, nothing to change (see *Log*)
- [x] docs — `docs(T44): document one device per Wayland seat`

## Learning path

No new tutorial; the vendored-backend tutorial of T35 covers the ground.

## Log

- 2026-11-02 — note written from the owner's report and code reading;
  not reproduced (see *Known facts*).
- 2026-11-02 — implemented on `fix/wayland-seat-devices`. AC-1 to AC-6
  pass. `scripts/check.sh` was green right after the behaviour commit, so
  the quality step had nothing to commit. Smoke run with
  `DRAW_BACKEND=wayland WAYLAND_DEBUG=client` on Hyprland 0.56.2: one
  `capabilities(3)`, one `get_pointer`, one cursor-shape device (the
  start-up `init_pointer_context` returns early) and one `get_keyboard`.
  The release path could not be driven live, because the compositor
  trigger is still unknown.
- AC-7 (owner check) pending: type in a text block and in a shape with
  `DRAW_BACKEND=wayland`, check the cursor shape and the pen.
