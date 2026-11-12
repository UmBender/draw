---
id: T145
title: OpenGL ES 3.0 context and loader (EGL and GLX)
status: todo
wave: 69
branch: task/T145-opengl-es-3-0-context-and-loader-egl-and
depends_on: [T142]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T145 OpenGL ES 3.0 context and loader (EGL and GLX)

## Goal

A GL context on a native window handle, with the GL functions loaded and a typed `Gpu` implementation over them.

## Known facts (2026-11-12)

- Needs a window from a backend; tested first on a headless EGL surface (surfaceless or pbuffer) so it does not wait for T143 and T144.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — EGL context creation (Wayland and X11 platforms) and GLX as a fallback return errors as values; the context is a typed value that owns its surface.
  Integration tests on a headless surface; unit tests for every failure; regression tests per bug.
- **AC-2** — A GL function table is loaded once, checked for the entry points draw needs, and each call is wrapped to check `glGetError` in debug builds.
  Test: `missing_entry_point_is_reported`.
- **AC-3** — `Gpu` is implemented for it: buffers, textures, programs, one command list per frame; a test draws a known triangle set and reads back the pixels.
  Test: `triangle_set_reads_back_exactly`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-sys/src/egl.rs`, `crates/draw-sys/src/glx.rs`, `crates/draw-gfx/src/gpu/gl.rs`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T145): specify the GL context and loader`
- [ ] tests — `test(T145): add failing tests for the context`
- [ ] models — `feat(T145): add the context and function table types`
- [ ] behaviour — `feat(T145): implement EGL, GLX and the GL Gpu`
- [ ] quality — `chore(T145): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
