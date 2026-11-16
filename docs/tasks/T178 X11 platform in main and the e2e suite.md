---
id: T178
title: X11 platform in main and the e2e suite
status: in-progress
wave: 73
branch: task/T178-x11-platform-in-main-and-the-e2e-suite
depends_on: [T177]
adrs: [ADR-T178-1]
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

Design: [[ADR-T178-1 X11 platform hands out a shared GL handle]].

- **AC-0** — the platform hands out a shared GL handle the Gpu can own. Tests (`crates/draw-sys/tests/x11_share.rs`): `x11_shared_gl_makes_current_and_swaps`, `x11_shared_gl_reports_the_resized_window_size`, `x11_shared_gl_outlives_the_platform_without_using_a_closed_display`.
- **AC-1** — `main` runs `app::run::<X11Platform, X11Gpu>` for `--platform native` on X11. Tests: `platform_takes_macroquad_and_native`, `platform_refuses_an_unknown_name`, `defaults_without_arguments` (platform `macroquad`) in `tests/options.rs`; `main_runs_generic_run_through_a_match` (source guard, now also `X11Platform::open` and `X11Gpu`) in `src/main.rs`; `help_lists_the_flags` (`--platform`).
- **AC-2** — `scripts/e2e.sh --backend x11` passes with `--platform native` (T144 AC-3): the script takes `--platform macroquad|native` (default `macroquad`) and passes it to the app through `DRAW_E2E_PLATFORM`; run by hand with `scripts/e2e.sh --backend x11 --platform native`. Test: `app_args_pass_the_platform` in `tests/e2e.rs`.
- **AC-3** — Hand check (owner): window, keyboard, pointer, tablet and clipboard in the real app on X11.

## Out of scope

- Removing miniquad ([[T149 Cutover removing miniquad and macroquad]]); frame pacing ([[T147 Frame pacing by the compositor and idle at zero]]).

## Files owned

- `src/main.rs`, `src/shell/app.rs` (selection only), `src/shell/options.rs` or wherever the flags live, `scripts/e2e.sh`, `tests/e2e*`, `tests/options.rs`
- widened by the owner (2026-11-15): `crates/draw-sys/src/x11/**`, `crates/draw-sys/tests/x11*`
- new: `src/shell/gpu_x11.rs` and its line in `src/shell/mod.rs`
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
- 2026-11-15 — unblocked by the owner: Files owned widened to `crates/draw-sys/src/x11/**` and `crates/draw-sys/tests/x11*`; design in ADR-T178-1.
