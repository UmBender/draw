---
id: T190
title: Windows keyboard and pointer
status: todo
wave: 79
branch: task/T190-windows-keyboard-and-pointer
depends_on: [T188]
adrs: []
feature: "[[Windows]]"
tutorial: ""
tags: [task, windows]
---

# T190 Windows keyboard and pointer

## Goal

The Win32 platform reports keys, typed text, mouse buttons, motion and
the wheel as `RawEvent`s, with every message decoded by pure functions
that Linux CI tests too.

## Known facts (2026-11-24)

- Decisions: [[Windows port proposal]] §3.5.
- `RawKey` has `Letter(char)`, `Digit`, named keys and `Other(u32)`;
  `RawMods` has shift, ctrl and alt; `Key` and `Char` events carry
  `repeat` (`crates/draw-sys/src/platform.rs`).
- Keys: `WM_KEYDOWN`/`WM_KEYUP` and `WM_SYSKEYDOWN`/`WM_SYSKEYUP`;
  bit 30 of `lParam` is the repeat flag. Letters come from the layout's
  virtual-key codes. Text comes from `WM_CHAR`, with UTF-16 surrogate
  pairs joined and dead keys through `TranslateMessage`.
- AltGr arrives as a synthetic left Ctrl with the same message time as
  the right Alt that follows: typing with AltGr must never fire a Ctrl
  shortcut (Brazilian ABNT2 and other layouts).
- Pointer: `EnableMouseInPointer(TRUE)` (set in T188) sends the mouse as
  `WM_POINTER*`; pen samples are [[T191 Windows pen and clipboard]].
- Wheel: `WM_MOUSEWHEEL`/`WM_MOUSEHWHEEL`, 120 units per notch;
  precision touchpads send fractions. `src/shell/backend.rs` holds the
  per-backend wheel units.
- Windows tests run only on the Windows CI job (proposal, question 7).

## Spec

Draft; the spec step names the final tests.

- **AC-1** — Virtual keys map to `RawKey` (letters, digits, named keys,
  modifiers), with `repeat` from bit 30.
  Tests (pure): `vk_letters_and_digits_map_to_raw_keys`,
  `vk_named_keys_map_to_raw_keys`, `repeat_bit_sets_repeat`.
- **AC-2** — Text: `WM_CHAR` becomes `Char`, surrogate pairs joined, a
  lone surrogate dropped.
  Tests (pure): `utf16_surrogate_pairs_join`, `lone_surrogate_is_dropped`.
- **AC-3** — AltGr: the synthetic left Ctrl before right Alt is not a
  Ctrl press, a real Ctrl+Alt still is.
  Tests (pure): `altgr_pair_is_not_ctrl`, `real_ctrl_alt_is_ctrl_and_alt`.
- **AC-4** — Pointer motion and buttons in physical pixels; the wheel in
  notches, fractions added up.
  Tests (pure): `pointer_coordinates_are_physical`,
  `wheel_delta_120_is_one_notch`, `touchpad_wheel_fractions_add_up`;
  `wheel_unit_for_windows` (`backend.rs`).
- **AC-5** — Injected input reaches `poll`.
  Tests (Windows): `win32_sendinput_key_reaches_poll`,
  `win32_sendinput_text_reaches_char_events`,
  `win32_mouse_buttons_and_motion_reach_poll`,
  `win32_wheel_reaches_poll`.

Manual check (the owner's friend): shortcuts and typing on their own
keyboard layout, including AltGr characters inside a text block.

## Out of scope

- Pen and clipboard ([[T191 Windows pen and clipboard]]).
- IME composition (draw's text is ASCII and TeX).

## Files owned

- `crates/draw-sys/src/win32/decode.rs`, `crates/draw-sys/src/win32/input.rs` (new),
  `crates/draw-sys/src/win32/window.rs` (message dispatch),
  `crates/draw-sys/src/win32/platform.rs`
- `crates/draw-sys/tests/win32_input.rs`, `crates/draw-sys/tests/win32_pure.rs`
- `src/shell/backend.rs` (the Windows wheel unit)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T190): specify Windows keyboard and pointer`
- [ ] tests — `test(T190): add failing tests for Windows input`
- [ ] models — `feat(T190): add the Windows input decoders`
- [ ] behaviour — `feat(T190): implement Windows keyboard and pointer`
- [ ] quality — `chore(T190): pass clippy and rustfmt`

## Log

- 2026-11-24 — written from the Windows port proposal.
