---
id: T44
title: One keyboard per Wayland seat
status: todo
wave: 29
branch: fix/wayland-seat-devices
depends_on: [T28]
adrs: []
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

## Out of scope

- X11 (one keyboard, no such callback).
- Finding the exact Hyprland trigger.
- Upstreaming the fix to miniquad.

## Files owned

- `vendor/miniquad/src/native/linux_wayland.rs`, a new helper module or
  `vendor/miniquad/src/pen.rs`-style file in the vendored crate,
  `vendor/miniquad/src/lib.rs`, `vendor/miniquad/PATCHED.md`
- draw tests calling the helper (e.g. `src/shell/input_map.rs` tests)
- `docs/features/Native Wayland (experimental).md`
- new ADR `ADR-T44-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T44): …`
- [ ] tests — `test(T44): …`
- [ ] models — `feat(T44): …`
- [ ] behaviour — `fix(T44): …`
- [ ] quality — `chore(T44): …`
- [ ] docs — `docs(T44): …`

## Learning path

No new tutorial; the vendored-backend tutorial of T35 covers the ground.

## Log

- 2026-11-02 — note written from the owner's report and code reading;
  not reproduced (see *Known facts*).
