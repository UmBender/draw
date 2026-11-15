---
id: T178
title: X11 platform in main and the e2e suite
status: todo
wave: 73
branch: task/T178-x11-platform-in-main-and-the-e2e-suite
depends_on: [T177]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T178 X11 platform in main and the e2e suite

## Goal

Slice 4 of T144: `--platform native` on an X11 session runs `app::run` over `X11Platform` and the OpenGL ES `Gpu` of `draw-gfx`, and the e2e suite passes with it.

## Known facts (2026-11-15)

- Split out of [[T144 X11 backend with window, keyboard, pointer, XInput2 pressure and selections]] by the owner before the batch (as large as T143, T156 and T164).
- **Owner decision:** Xlib, not pure xcb: the GLX context of [[ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0]] takes an Xlib `Display*`. XInput2 through `libXi`, as the vendored miniquad does. All libraries are loaded with `dlopen` through the `loader` of [[ADR-T142-1 dlopen through libc symbols and TEST markers]]; no new crate dependency ([[ADR-T127-3 The dependency list]]); each `unsafe` primitive follows [[ADR-T127-2 The unsafe policy]].
- Integration tests start their own Xvfb with the helper `crates/draw-sys/tests/common/xvfb.rs` (T145) and fail, never skip, when it is missing.
- **Owner decisions (2026-11-15):** the flag is `--platform macroquad|native` (default `macroquad` until [[T149 Cutover removing miniquad and macroquad]]); `native` picks Wayland or X11 from the session. If [[T171 Wayland platform in main and the e2e suite]] has not added the flag yet on this branch, this task adds it the same way; the integrator merges the two. The e2e goldens hold with the tolerance `scripts/e2e.sh` already uses; a larger difference blocks the task with the difference images.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `main` runs `app::run::<X11Platform, GlGpu>` for `--platform native` on X11.
- **AC-2** — `scripts/e2e.sh --backend x11` passes with `--platform native` (T144 AC-3).
- **AC-3** — Hand check (owner): window, keyboard, pointer, tablet and clipboard in the real app on X11.

## Out of scope

- Removing miniquad ([[T149 Cutover removing miniquad and macroquad]]); frame pacing ([[T147 Frame pacing by the compositor and idle at zero]]).

## Files owned

- `src/main.rs`, `src/shell/app.rs` (selection only), `src/shell/options.rs` or wherever the flags live, `scripts/e2e.sh`, `tests/e2e*`, `tests/options.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T178): specify the slice`
- [ ] tests — `test(T178): add failing tests`
- [ ] behaviour — `feat(T178): run the app over the X11 platform`
- [ ] quality — `chore(T178): pass clippy and rustfmt`
- [ ] by hand — owner, recorded in the Log

## Log

- 2026-11-15 — split from T144 by the owner.
- 2026-11-15 — BLOCKED (batch): `app::run::<X11Platform, GlGpu<_>>` cannot be built from `main` with the files this task owns. `GlGpu::new(context, size)` takes the `GlContext` by value, but `X11Platform` owns its `GlSurface` privately (`crates/draw-sys/src/x11/platform.rs`: field `gl`, only `gl(&self) -> &GlSurface`, not `Clone`) and also calls `swap_buffers` itself in `next_frame`; `main` has `forbid(unsafe_code)`, so it cannot alias the context. Recommended fix (needs `crates/draw-sys/src/x11/**`, a T177 file): either `X11Platform::open` returns the platform and a shareable `GlSurface` handle (`Rc` inside, `Clone`; window dropped after both), or add `take_gl(&mut self)` and make `next_frame` stop swapping when the Gpu does (`GlGpu::swap_buffers`); then T178 adds `--platform macroquad|native` and the `Runtime::X11` arm and the e2e run. Decide who swaps the buffers (Platform or Gpu) in an ADR first. Nothing else was started.
