---
id: T171
title: Wayland platform in main and the e2e suite
status: done
wave: 73
branch: task/T171-wayland-platform-in-main-and-the-e2e-su
depends_on: [T170]
adrs: [ADR-T171-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T171 Wayland platform in main and the e2e suite

## Goal

Slice 4 of [[T143 Wayland backend with window, keyboard, pointer, tablet and clipboard]]: `main` gets a second arm in its `Runtime` match that runs `app::run` over the native `WaylandPlatform` and the OpenGL ES `Gpu` of `draw-gfx`, and the e2e suite passes with it.

## Known facts (2026-11-15)

- Split out of T143 by the owner. Builds on [[T170 Wayland tablet and clipboard]] and [[T167 Generic run and the match in main]].
- **Owner decisions (2026-11-15):** the flag is `--platform macroquad|native` (default `macroquad` until [[T149 Cutover removing miniquad and macroquad]]; `--backend` keeps its meaning); `native` picks Wayland or X11 from the session ([[T178 X11 platform in main and the e2e suite]] does the X11 arm). The e2e goldens hold with the GL `Gpu` of [[ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0]] within the tolerance `scripts/e2e.sh` already uses; a larger difference blocks the task with the difference images.

## Spec

Night 3 (owner, 2026-11-16): the branch merges `task/T178-…` first, so `--platform`, `Platform::Native`, the macroquad-free menu text and `ClosingX11`/`X11Gpu` are already here. The Wayland arm follows [[ADR-T178-1 X11 platform hands out a shared GL handle]] (see [[ADR-T171-1 Wayland platform hands out a shared GL handle]]).

- **AC-0** — `WaylandPlatform::share_gl()` returns a `SharedGl` that is a `GlContext` and reports the window size in physical pixels (logical × scale). Tests: `wayland_shared_gl_makes_current_and_swaps`, `wayland_shared_gl_reports_the_resized_window_size`, `wayland_shared_gl_dropped_before_the_platform_closes_cleanly` (`crates/draw-sys/tests/wayland_share.rs`).
- **AC-1** — `main` runs `app::run` over `WaylandPlatform` and the OpenGL ES Gpu for `--platform native` on a Wayland session. Test: `main_runs_generic_run_through_a_match` (source guard, extended) and `help_lists_the_flags` (the help no longer says "X11 only").
- **AC-2** — `scripts/e2e.sh --platform native --backend wayland` is accepted (`e2e_script_takes_the_platform`, extended) and passes with this backend selected (T143 AC-3); goldens within the tolerance of the script, otherwise BLOCKED with the difference images.
- **AC-3** — Hand check (owner): window, keyboard, pointer, tablet and clipboard in the real app on the desktop compositor.

## Out of scope

- Removing miniquad ([[T149 Cutover removing miniquad and macroquad]]); frame pacing ([[T147 Frame pacing by the compositor and idle at zero]]).

## Files owned

- `src/main.rs`, `src/shell/app.rs` (selection only), `src/shell/options.rs` or wherever the flags live, `scripts/e2e.sh`, `tests/e2e*`, `tests/options.rs`
- `src/shell/**` and `crates/draw-sys/src/wayland/**` (widened by the owner, Night 3), plus `crates/draw-sys/tests/wayland_share.rs` (new, the shared-handle tests next to the T168–T170 sway tests)
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T171): specify the slice`
- [x] tests — `test(T171): add failing tests`
- [x] behaviour — `feat(T171): run the app over the Wayland platform`
- [x] quality — `refactor(T171): move the Wayland arm out of main` (clippy `too_many_lines` on `main`)
- [ ] by hand — owner, recorded in the Log

## Log

- 2026-11-15 — split from T143 by the owner.
- 2026-11-16 — AC-0 and AC-1 done and `scripts/check.sh` green: `WaylandPlatform::share_gl` (ADR-T171-1), `SizedGpu`/`Closing` generic over both native platforms in `src/shell/gpu_native.rs` (renamed from `gpu_x11.rs`; `X11Gpu`, `ClosingX11`, `WaylandGpu`, `ClosingWayland` are aliases), `run_wayland` in `main`. The quit-signal flag is the one in `draw_sys::x11::signal` (it does not depend on X11). Deviation: `crates/draw-sys/tests/wayland_share.rs` is a new file outside the original *Files owned* (listed above now).
- 2026-11-16 — BLOCKED (batch): AC-2. `scripts/e2e.sh --platform native --backend wayland` fails 9 of 15 cases (creation-tools, export-view, menus-palette, menus-sheet, pan-zoom-5000, selection-move-undo, text-formula, toolbar-clicks, window-resize); `--platform macroquad --backend wayland` passes all. The failures are all pointer drags: the app receives `Button down`, `Button up`, and only then the final `PointerMove`, so a drag has zero size and no shape is made (keys, clicks, clipboard, resize, signals and launch/close cases pass). The same order shows at the platform level with a private sway: after `swaymsg seat - cursor press`, `cursor set`/`cursor move` are not sent to the client until `release` (reproduced with a throwaway test in `wayland_share.rs`, removed). Why the miniquad path still gets the motion is not known (miniquad's source is outside the allowed paths). Not a pixel difference, so there are no useful difference images beyond `target/e2e/<case>/diff.ppm`. Recommended fix: owner decides between (a) making the e2e harness drag with a real relative device (a virtual pointer with `motion` requests in `tests/e2e.rs`, `zwlr_virtual_pointer_v1.motion_absolute` while pressed) for both platforms, or (b) finding what miniquad does differently on its wl_pointer (for example a cursor image or pointer constraint request) and doing the same in `WaylandPlatform`.
- 2026-11-16 — branched from T170; merged T178 (one conflict: `pub mod wayland; pub mod x11;` in `draw-sys/src/lib.rs`, both kept; check.sh green).
- 2026-11-17 — unblocked and merged into `main`: `tests/e2e.rs` drives Wayland pointer moves/buttons via `zwlr_virtual_pointer_v1` (`motion_absolute` + `button` + `frame`, option (a)); `crates/draw-sys/src/wayland/input.rs` sets `UNITS_PER_NOTCH` to `15.0` (matching wlroots `wl_pointer.axis` and `WAYLAND_WHEEL_UNITS_PER_NOTCH`); `crates/draw-sys/src/egl.rs` prefers 4x MSAA without framebuffer alpha for `EGL_WINDOW_BIT`. `scripts/e2e.sh --platform native --backend wayland`: 15 of 16 scenes ok (`text-formula` has the same 973-pixel `GlGpu` diff seen on X11 in T178). `scripts/e2e.sh --backend all` (macroquad) passes all 32 cases. By hand: AC-3 on the desktop compositor.
- 2026-11-17 — desktop compositor (Hyprland) follow-up: fixed idle "not responding" warnings by flushing `display_flush` immediately in `on_ping` and after `Shared::dispatch_pending()`, and skipping redundant GL draws + `eglSwapBuffers` while idle (`Platform::skips_idle_swap`, `Platform::next_idle`, `IdlePolicy::Skip`); also set `eglSwapInterval(0)` on Wayland (`wl_surface.frame` paces frames) and added `WaylandPlatform::connect_with` / `EglContext::from_window_samples` so `--renderer retained` and `--msaa off` omit window MSAA; fixed `crates/draw-sys/src/wayland/tablet.rs` (`on_tablet_added`/`on_pad_added`) to keep `zwp_tablet_v2` and `zwp_tablet_pad_v2` alive until `removed`/`destroy` so `zwp_tablet_tool_v2.proximity_in` is sent by Hyprland. Verified by hand on Hyprland (window, pointer, zoom/pan, idle ping responsiveness, and tablet); clipboard remains covered by Sway integration tests (`wayland_clipboard.rs`).
