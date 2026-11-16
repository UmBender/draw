---
id: T176
title: X11 keyboard, pointer and XInput2 pressure
status: done
wave: 71
branch: task/T176-x11-keyboard-pointer-and-xinput2-pressu
depends_on: [T175]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T176 X11 keyboard, pointer and XInput2 pressure

## Goal

Slice 2 of T144: the keyboard (layouts through xkbcommon-x11 or Xlib's keysyms, repeat detection), the pointer with buttons and wheel, and pen pressure through XInput2, as `RawEvent`s and pen samples.

## Known facts (2026-11-15)

- Split out of [[T144 X11 backend with window, keyboard, pointer, XInput2 pressure and selections]] by the owner before the batch (as large as T143, T156 and T164).
- **Owner decision:** Xlib, not pure xcb: the GLX context of [[ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0]] takes an Xlib `Display*`. XInput2 through `libXi`, as the vendored miniquad does. All libraries are loaded with `dlopen` through the `loader` of [[ADR-T142-1 dlopen through libc symbols and TEST markers]]; no new crate dependency ([[ADR-T127-3 The dependency list]]); each `unsafe` primitive follows [[ADR-T127-2 The unsafe policy]].
- Integration tests start their own Xvfb with the helper `crates/draw-sys/tests/common/xvfb.rs` (T145) and fail, never skip, when it is missing.
- Patches to port (`vendor/miniquad/PATCHED.md`): `PenTracker` (the `Abs Pressure` valuator of `XI_RawMotion`), samples pushed before core motion, press and release.
- Regressions to keep: the X11 bugs of T28, T35 and T45 (`fix/wayland-idle-doubled-keys` has an X11 counterpart in autorepeat detection).
- Xvfb has no tablet: pressure tests decode recorded XI2 events; the owner checks a real tablet by hand at review.

## Spec

Design: pure decoders next to the T175 ones (`x11/keys.rs`, `x11/pen.rs`, new variants of `x11/event.rs`); the only `unsafe` stays in `x11/xlib.rs` (Xlib `XkbKeycodeToKeysym`, `XkbSetDetectableAutoRepeat`, `XGetEventData`; `libXi` for `XIQueryVersion`, `XISelectEvents`, `XIQueryDevice`). Keys use the active layout through `XkbKeycodeToKeysym` (group from the event state). Autorepeat: detectable autorepeat is switched on, so a held key sends repeated presses with no release; a pure `KeyTracker` marks a press of a key already down as `repeat`. Raw pressure comes from `XI_RawMotion` selected on the root window; samples go out before the core motion, press and release (the T35 patch). Without `libXi` or XInput2 2.0 the platform runs without pen support. Tests use a seeded xorshift fuzz, as T175 does (`proptest` is not a dev-dependency of this crate), so AC-4 follows that. No new ADR: the choices follow ADR-T127-2, ADR-T127-3 and ADR-T142-1.

- **AC-1** — Keyboard: `RawKey`/`RawMods` with the active layout, autorepeat detected. Pure: `keys_keysym_letters_map_to_uppercase_letters`, `keys_keysym_special_keys_map_to_named_keys`, `keys_unknown_keysym_is_other_with_evdev_code`, `keys_char_from_keysym_latin1_and_unicode`, `keys_char_from_keysym_rejects_controls`, `keys_mods_from_state_reads_shift_control_alt`, `keys_level_uses_shift_lock_and_altgr`, `key_tracker_second_press_is_repeat`, `key_tracker_press_after_release_is_not_repeat`, `key_tracker_clear_forgets_held_keys`. Integration (`xdotool`): `x11_key_by_xdotool_reports_key_and_char`, `x11_ctrl_key_reports_mods_and_no_char`, `x11_held_key_reports_repeat_until_release` (regression T45 / `fix/wayland-idle-doubled-keys`: one press, then repeats), `x11_focus_events_reach_poll`. Regression T28: `x11_regression_t28_key_release_after_focus_loss_clears_state`.
- **AC-2** — Pointer: motion, buttons, wheel (buttons 4–7). Pure: `keys_button_maps_left_middle_right_other`, `keys_wheel_buttons_map_to_deltas`. Integration: `x11_pointer_motion_by_xdotool_reports_position`, `x11_button_press_and_release_report_button_and_position`, `x11_wheel_buttons_report_wheel_and_no_button`.
- **AC-3** — XInput2 pressure: device query, the `Abs Pressure` valuator, normalised samples; eraser and barrel buttons reach the shell as `Middle`/`Right`/`Other` buttons as today. Pure on recorded events: `pen_valuator_reads_packed_values_by_mask`, `pen_valuator_unset_bit_is_none`, `pen_normalise_clamps_to_unit_range`, `pen_normalise_degenerate_range_is_zero`, `pen_tracker_keeps_last_pressure_when_valuator_missing`, `pen_tracker_device_without_axis_clears_pressure`, `pen_tracker_new_pen_starts_from_zero`, `pen_sample_is_none_without_pressure`; decoder of the XI generic event: `event_decode_generic_reads_extension_and_evtype`. Integration: `x11_xi2_raw_motion_from_xtest_is_not_a_pen` (Xvfb raw motion arrives, its device has no pressure axis, so no `Tablet`). Regression T35: `regression_t35_pressure_sample_precedes_press_and_release` (recorded sequence through the tracker).
- **AC-4** — Decoders fuzzed: `fuzz_event_decode_random_bytes_never_panics` (key, button, motion, focus, generic types forced), `fuzz_keys_random_keysyms_never_panic`, `fuzz_pen_valuator_random_masks_never_panic`.
- **AC-5** — Hand check with a tablet (owner, at review), recorded in the Log.

## Out of scope

- Selections and the clean exit ([[T177 X11 selections and clean exit]]); `main` and e2e ([[T178 X11 platform in main and the e2e suite]]); IME.

## Files owned

- `crates/draw-sys/src/x11/**`, the `pub mod x11` line of `crates/draw-sys/src/lib.rs`
- `crates/draw-sys/tests/x11*.rs`, `crates/draw-sys/tests/common/xvfb.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T176): specify the slice`
- [x] tests — `test(T176): add failing tests`
- [x] models — `feat(T176): add the types`
- [x] behaviour — `feat(T176): implement the keyboard, pointer and pressure`
- [x] quality — `chore(T176): pass clippy and rustfmt`
- [ ] tablet by hand — owner, recorded in the Log

## Log

- 2026-11-15 — split from T144 by the owner.
- 2026-11-15 — implemented in the batch run; `scripts/check.sh` green. Pure decoders in `x11/keys.rs` and `x11/pen.rs`, new `XEvent` variants, `unsafe` only in `x11/xlib.rs`. The integration tests run on a private Xvfb with `xdotool`.
- 2026-11-15 — deviations: the T175 test `decode_short_or_unknown_events_are_other` used event type 2 as "unknown"; type 2 is now a key press, so it uses type 12 (`x11_pure.rs`). The fuzz is a seeded xorshift, not proptest (not a dev-dependency). The note's "X11 bugs of T28" were not identifiable (T28 is a Wayland task): the focus-loss test `x11_regression_t28_key_release_after_focus_loss_clears_state` covers the lost-release case instead. No feature note or tutorial: the task frontmatter has none. No ADR.
- 2026-11-15 — AC-5 (tablet by hand) is left to the owner: with a real tablet, check pressure, the tip lift and the eraser/barrel buttons.
