---
id: T177
title: X11 selections and clean exit
status: todo
wave: 72
branch: task/T177-x11-selections-and-clean-exit
depends_on: [T176]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T177 X11 selections and clean exit

## Goal

Slice 3 of T144: the clipboard and primary selections (text and `image/png`, with `INCR` for large transfers), and the window-manager close and `Ctrl-C` behaving as [[T48 Clean exit on Ctrl-C]] does today.

## Known facts (2026-11-15)

- Split out of [[T144 X11 backend with window, keyboard, pointer, XInput2 pressure and selections]] by the owner before the batch (as large as T143, T156 and T164).
- **Owner decision:** Xlib, not pure xcb: the GLX context of [[ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0]] takes an Xlib `Display*`. XInput2 through `libXi`, as the vendored miniquad does. All libraries are loaded with `dlopen` through the `loader` of [[ADR-T142-1 dlopen through libc symbols and TEST markers]]; no new crate dependency ([[ADR-T127-3 The dependency list]]); each `unsafe` primitive follows [[ADR-T127-2 The unsafe policy]].
- Integration tests start their own Xvfb with the helper `crates/draw-sys/tests/common/xvfb.rs` (T145) and fail, never skip, when it is missing.
- The PNG selection of [[T115 Copy the selection as an image]] must keep working.
- Regressions to keep: the X11 selection bugs of T115.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — Own and answer `CLIPBOARD` and `PRIMARY` (`UTF8_STRING`, `TARGETS`, `image/png`), read them from other clients, `INCR` both ways. Integration tests against Xvfb with `xclip` or a second connection in the test; regressions for T115.
- **AC-2** — WM close and `Ctrl-C` (SIGINT) exit cleanly as in T48. Integration test.
- **AC-3** — Selection decoders fuzzed (proptest).

## Out of scope

- `main` and e2e ([[T178 X11 platform in main and the e2e suite]]).

## Files owned

- `crates/draw-sys/src/x11/**`, the `pub mod x11` line of `crates/draw-sys/src/lib.rs`
- `crates/draw-sys/tests/x11*.rs`, `crates/draw-sys/tests/common/xvfb.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T177): specify the slice`
- [ ] tests — `test(T177): add failing tests`
- [ ] models — `feat(T177): add the types`
- [ ] behaviour — `feat(T177): implement the selections and the clean exit`
- [ ] quality — `chore(T177): pass clippy and rustfmt`

## Log

- 2026-11-15 — split from T144 by the owner.
