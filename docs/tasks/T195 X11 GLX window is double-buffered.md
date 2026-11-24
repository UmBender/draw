---
id: T195
title: X11 GLX window is double-buffered
status: in-progress
wave: 77
branch: fix/x11-glx-double-buffer
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, bug, x11]
---

# T195 X11 GLX window is double-buffered

## Goal

On X11 with a hardware GPU, the draw window shows the canvas and the
toolbar and follows resizes: the GLX context draws into a
double-buffered config, so `glXSwapBuffers` presents each frame.

## Known facts (2026-11-24)

- Reported by the owner on 2.0.0 with `draw --backend x11` on KDE Plasma
  Wayland (Xwayland, Intel UHD 620, Mesa 26.2.7): the window stays black
  (or shows what is behind it), and after maximising, the new area is not
  drawn. The diagnostics log has no error; the first frame is drawn.
- Reproduced in the owner's session: black with `--msaa off`, see-through
  with the default MSAA, and **correct with `LIBGL_ALWAYS_SOFTWARE=1`**.
- Cause: `GlxContext::build_inner` (`crates/draw-sys/src/glx.rs`) and
  `Connection::glx_visual` (`crates/draw-sys/src/x11/xlib.rs`) ask
  `glXChooseFBConfig` for RGBA8 and take the first config, without
  `GLX_DOUBLEBUFFER`. On this machine the first config is single-buffered
  (`GLX_DOUBLEBUFFER = 0`, visual depth 24), for the hardware driver and
  for llvmpipe alike (checked with a ctypes probe of the same
  attributes). On a single-buffered window `glXSwapBuffers` does nothing:
  the hardware path (DRI3 and Present) never shows a frame, while the
  software path copies to the window on every flush, which is why the
  e2e suite (Xvfb, `LIBGL_ALWAYS_SOFTWARE=1`) never saw it.
- With `GLX_DOUBLEBUFFER True` added to both lists, the first config is
  double-buffered (same depth-24 visual) and the window draws correctly
  on the hardware path (checked by hand in the owner's session, then the
  change was reverted for the workflow).
- The two attribute lists are copies that must stay equal (the window is
  created on the visual of the config the context picks, T175); one
  shared list removes that risk.
- The pbuffer path (tests only) stays single-buffered: pbuffers have no
  back buffer.

## Spec

- **AC-1** — Regression: a GLX context made on an X11 window uses a
  double-buffered config.
  Test: `regression_T195_glx_window_config_is_double_buffered`
  (`crates/draw-sys/tests/x11.rs`, Xvfb; it reads `GLX_DOUBLEBUFFER` of the
  context's config through `GlxContext::is_double_buffered`).
- **AC-2** — The window's visual and the context's config come from one
  attribute list, which asks for a double-buffered RGBA8 window config.
  Test: `window_config_asks_for_a_double_buffer` (`glx.rs` unit test).
- **AC-3** — The pbuffer context still works.
  Test: `glx_pbuffer_context_makes_current_under_xvfb` (unchanged).

Manual check (owner): `draw --backend x11` on KDE Plasma Wayland shows
the canvas and the toolbar and redraws after maximising.

## Out of scope

- The EGL path on X11 (an EGL window surface is back-buffered already).
- Running the e2e suite on hardware GL.

## Files owned

- `crates/draw-sys/src/glx.rs`
- `crates/draw-sys/src/x11/xlib.rs` (`glx_visual`)
- `crates/draw-sys/tests/x11.rs`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T195): specify the double-buffered GLX window`
- [x] regression — `test(T195): add regression test for the single-buffered GLX window`
- [x] models — `feat(T195): add GlxContext::is_double_buffered`
- [x] behaviour — `fix(T195): ask GLX for a double-buffered window config`
- [ ] quality — `chore(T195): pass clippy and rustfmt`
- [ ] docs — `docs(T195): update the notes`

## Log

- 2026-11-24 — written from the owner's screenshots of `--backend x11`;
  cause found with a GLX config probe and confirmed by a temporary patch.
