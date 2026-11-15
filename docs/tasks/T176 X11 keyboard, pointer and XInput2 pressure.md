---
id: T176
title: X11 keyboard, pointer and XInput2 pressure
status: todo
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

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Keyboard: `RawKey`/`RawMods` with the active layout, autorepeat detected (detectable autorepeat or the release/press pair). Integration tests with `xdotool key`; regressions for T28 and T45.
- **AC-2** — Pointer: motion, buttons, wheel (buttons 4–7). Integration tests with `xdotool`.
- **AC-3** — XInput2 pressure: device query, the `Abs Pressure` valuator, normalised samples; eraser and barrel buttons as today. Tests on recorded events; regression for T35.
- **AC-4** — Decoders fuzzed (proptest).
- **AC-5** — Hand check with a tablet (owner, at review), recorded in the Log.

## Out of scope

- Selections and the clean exit ([[T177 X11 selections and clean exit]]); `main` and e2e ([[T178 X11 platform in main and the e2e suite]]); IME.

## Files owned

- `crates/draw-sys/src/x11/**`, the `pub mod x11` line of `crates/draw-sys/src/lib.rs`
- `crates/draw-sys/tests/x11*.rs`, `crates/draw-sys/tests/common/xvfb.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T176): specify the slice`
- [ ] tests — `test(T176): add failing tests`
- [ ] models — `feat(T176): add the types`
- [ ] behaviour — `feat(T176): implement the keyboard, pointer and pressure`
- [ ] quality — `chore(T176): pass clippy and rustfmt`
- [ ] tablet by hand — owner, recorded in the Log

## Log

- 2026-11-15 — split from T144 by the owner.
