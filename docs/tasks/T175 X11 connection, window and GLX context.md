---
id: T175
title: X11 connection, window and GLX context
status: done
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

Layout: `crates/draw-sys/src/x11/` has `event.rs` (safe decoder of the 192-byte `XEvent`), `dpi.rs` (pure DPI and scale), `xlib.rs` (the dlopen'd Xlib calls, the only `unsafe`) and `platform.rs` (`X11Platform`). Everything is tested from `crates/draw-sys/tests/x11.rs` (Xvfb) and `tests/x11_pure.rs` (no server).

- **AC-1** — Connect (`DISPLAY`), create and map a window with title, class and `WM_DELETE_WINDOW`; a GLX context on it renders and swaps; the EGL fallback works on the same window. Tests: `x11_window_has_title_and_class`, `x11_glx_context_clears_and_swaps`, `x11_egl_fallback_swaps_on_the_window`, `x11_missing_display_is_an_error_value`.
- **AC-2** — `ConfigureNotify` resize, DPI from `Xft.dpi` or the screen size, the WM close. Tests: `x11_resize_by_xdotool_reaches_window_metrics`, `x11_wm_close_sets_close_requested`, `dpi_from_xft_resource_reads_the_value`, `dpi_without_resource_falls_back_to_screen_size`, `dpi_scale_is_clamped_and_finite`, `metrics_divide_physical_size_by_scale`, `decode_configure_notify_reads_the_size`, `decode_client_message_reads_type_and_first_long`, `is_close_needs_protocols_and_delete_atoms`.
- **AC-3** — `X11Platform` implements `Platform` for the window part (`window()`, `poll` with resize, `schedule_update`, `next_frame`); keyboard, pointer, pen and clipboard return nothing yet, documented as T176/T177. Tests: `x11_platform_poll_reports_the_resize`, `x11_platform_input_and_clipboard_return_nothing`, `x11_platform_schedule_update_ends_the_wait`.
- **AC-4** — The X11 event decoder is fuzzed: `decode_random_events_never_panics`, `decode_random_configure_events_keep_the_fields`. Deviation: `proptest` is a dev-dependency of the root crate only and `crates/draw-sys/Cargo.toml` is not owned, so the fuzz uses a seeded xorshift generator over many thousands of events instead of proptest.

Design notes: the close is `X11Platform::close_requested()`, as in the Wayland backend, because `RawEvent` (`platform.rs`) is not owned by this task. The window is created on the visual of the GLX FB config that `GlxContext::from_window` will pick (same attribute list, same order), otherwise `glXCreateWindow` fails with `BadMatch`. `next_frame` swaps and waits at most 16 ms for events: idle at zero is [[T147 Frame pacing by the compositor and idle at zero]].

## Out of scope

- Keyboard, pointer, pen ([[T176 X11 keyboard, pointer and XInput2 pressure]]); selections and the clean exit ([[T177 X11 selections and clean exit]]); `main` and e2e ([[T178 X11 platform in main and the e2e suite]]).

## Files owned

- `crates/draw-sys/src/x11/**`, the `pub mod x11` line of `crates/draw-sys/src/lib.rs`
- `crates/draw-sys/tests/x11*.rs`, `crates/draw-sys/tests/common/xvfb.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T175): specify the slice`
- [x] tests — `test(T175): add failing tests`
- [x] models — `feat(T175): add the types`
- [x] behaviour — `feat(T175): implement the X11 window`
- [x] quality — `chore(T175): pass clippy and rustfmt`

## Log

- 2026-11-15 — split from T144 by the owner.
- 2026-11-15 — implemented (batch). The tests commit does not compile: the `x11` module is new (as T163/T165). The task has no `feature` or `tutorial`, so there is no docs commit besides this note. `scripts/check.sh` green.
- Deviations: (1) AC-4 fuzz uses a seeded xorshift, not proptest: `crates/draw-sys/Cargo.toml` is not owned. (2) The WM close is `X11Platform::close_requested()`, not a `RawEvent` (`platform.rs` not owned; same as Wayland). (3) No window manager runs under Xvfb and `xdotool windowquit` needs one (`_NET_CLOSE_WINDOW`), so the close test uses the new `x11::send_close_request`, which sends `WM_DELETE_WINDOW` as a WM does. (4) `common/xvfb.rs` now starts Xvfb with `-dpi 96`: the 64×64 screen gave a scale of 1.058.
- Found: `GlxContext` and `EglContext` `dlclose` their library handles on drop, before `XCloseDisplay`; libGL hooks the display close, so it segfaulted. `Connection` keeps `libGL` and `libEGL` loaded until after the display closes.
- DPI is read once at open (`Xft.dpi`, else screen size, else 96); following later `Xft.dpi` changes is not done. `next_frame` swaps and waits at most 16 ms for events; idle at zero is T147.
- Manual check for the owner: none.
