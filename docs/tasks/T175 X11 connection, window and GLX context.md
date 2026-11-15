---
id: T175
title: X11 connection, window and GLX context
status: todo
wave: 70
branch: task/T175-x11-connection-window-and-glx-context
depends_on: [T142, T145]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T175 X11 connection, window and GLX context

## Goal

Slice 1 of T144: `draw-sys` opens an X11 connection through Xlib, creates a window with a GLX context on it (EGL on X11 as the fallback T145 already has), follows resize, DPI and the window-manager close, and implements the window part of `Platform`.

## Known facts (2026-11-15)

- Split out of [[T144 X11 backend with window, keyboard, pointer, XInput2 pressure and selections]] by the owner before the batch (as large as T143, T156 and T164).
- **Owner decision:** Xlib, not pure xcb: the GLX context of [[ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0]] takes an Xlib `Display*`. XInput2 through `libXi`, as the vendored miniquad does. All libraries are loaded with `dlopen` through the `loader` of [[ADR-T142-1 dlopen through libc symbols and TEST markers]]; no new crate dependency ([[ADR-T127-3 The dependency list]]); each `unsafe` primitive follows [[ADR-T127-2 The unsafe policy]].
- Integration tests start their own Xvfb with the helper `crates/draw-sys/tests/common/xvfb.rs` (T145) and fail, never skip, when it is missing.
- `GlxContext::from_window` and `EglContext::from_window` (X11 platform) have no window test yet (T145 Log): this slice adds them.
- The app does not use the backend yet: [[T178 X11 platform in main and the e2e suite]].

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Connect (`DISPLAY`), create and map a window with title, class and `WM_DELETE_WINDOW`; a GLX context on it renders and swaps; the EGL fallback works on the same window. Integration tests against Xvfb.
- **AC-2** — `ConfigureNotify` resize, DPI from `Xft.dpi` or the screen size, the WM close as a `RawEvent`. Integration tests (`xdotool` for the close and resize).
- **AC-3** — `X11Platform` implements `Platform` for the window part (`window()`, `poll` with resize and close, `schedule_update`, `next_frame`); keyboard, pointer, pen and clipboard return nothing yet, documented as T176/T177.
- **AC-4** — The X11 event decoder is fuzzed (proptest).

## Out of scope

- Keyboard, pointer, pen ([[T176 X11 keyboard, pointer and XInput2 pressure]]); selections and the clean exit ([[T177 X11 selections and clean exit]]); `main` and e2e ([[T178 X11 platform in main and the e2e suite]]).

## Files owned

- `crates/draw-sys/src/x11/**`, the `pub mod x11` line of `crates/draw-sys/src/lib.rs`
- `crates/draw-sys/tests/x11*.rs`, `crates/draw-sys/tests/common/xvfb.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T175): specify the slice`
- [ ] tests — `test(T175): add failing tests`
- [ ] models — `feat(T175): add the types`
- [ ] behaviour — `feat(T175): implement the X11 window`
- [ ] quality — `chore(T175): pass clippy and rustfmt`

## Log

- 2026-11-15 — split from T144 by the owner.
