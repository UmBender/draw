---
id: T145
title: OpenGL ES 3.0 context and loader (EGL and GLX)
status: done
wave: 69
branch: task/T145-opengl-es-3-0-context-and-loader-egl-and
depends_on: [T142]
adrs: [ADR-T145-1]
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

Decisions: [[ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0]].

- **AC-1** — EGL context creation (Wayland and X11 platforms) and GLX as a fallback return errors as values; the context is a typed value that owns its surface.
  Integration tests on a headless surface; unit tests for every failure; regression tests per bug.
  Tests (`crates/draw-sys/tests/egl.rs`, `glx.rs`): `egl_headless_context_makes_current`, `egl_context_resolves_gl_entry_points`, `egl_missing_library_is_an_error_value` (`EglContext::headless_with_library("libdraw-none.so")`), `egl_zero_size_pbuffer_is_an_error_value`, `glx_pbuffer_context_makes_current_under_xvfb`, `glx_bad_display_is_an_error_value`.
- **AC-2** — A GL function table is loaded once, checked for the entry points draw needs, and each call is wrapped to check `glGetError` in debug builds.
  Test: `missing_entry_point_is_reported`; also `all_entry_points_present_loads` and `bad_gl_call_is_reported_with_its_name` (debug builds).
- **AC-3** — `Gpu` is implemented for it: buffers, textures, programs, one command list per frame; a test draws a known triangle set and reads back the pixels.
  Test: `triangle_set_reads_back_exactly`; also `texture_round_trips_through_write_and_read`, `alpha_blend_matches_straight_alpha_over`, `foreign_handle_is_unknown`, `bad_pixel_data_is_reported`, `zero_size_target_is_bad_size`, `multisampled_target_resolves`, `set_filter_changes_sampling`, `buffer_geometry_matches_inline_geometry`, and the same set over GLX (`glx_gpu_draws_triangle`).

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-sys/src/egl.rs`, `crates/draw-sys/src/glx.rs`, `crates/draw-gfx/src/gpu/gl.rs`
- the module lines in `crates/draw-sys/src/lib.rs`, `crates/draw-gfx/src/lib.rs` and `crates/draw-gfx/src/gpu.rs` (`pub mod gl;`)
- `crates/draw-gfx/Cargo.toml` (path dependency on `draw-sys`) and `tests/deps.rs`, `tests/unsafe_levels.rs` if they must learn it
- `crates/draw-sys/tests/`, `crates/draw-gfx/tests/` (GL integration tests, an Xvfb helper)
- `.github/workflows/ci.yml`: install the packages the GL tests need (`libegl1`, `libgl1-mesa-dri`, `xvfb`)
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T145): specify the GL context and loader`
- [x] tests — `test(T145): add failing tests for the context`
- [x] models — `feat(T145): add the context and function table types`
- [x] behaviour — `feat(T145): implement EGL, GLX and the GL Gpu`
- [x] quality — `chore(T145): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-13 — planned for a batch by the owner: *Files owned* widened in advance (module lines, Cargo, tests, CI). GL tests run in `scripts/check.sh` (headless EGL, surfaceless with a pbuffer fallback), and CI installs mesa so they run there too. GLX gets a real integration test: the test starts its own Xvfb (`-displayfd`) and fails, never skips, when it is missing.
- 2026-11-13 — implemented in 5 commits on `task/T145-opengl-es-3-0-context-and-loader-egl-and`; `scripts/check.sh` green on this machine (Mesa on Intel and llvmpipe). Notes for the owner:
  - Deviations: the `Gl` error test is `bad_gl_call_is_reported_with_its_name` (listed in AC-2 above); `ci.yml` also installs `libgl1` (`libGL.so.1` for GLX); the Xvfb helper lives in `crates/draw-sys/tests/common/xvfb.rs` and `draw-gfx` tests include it with `#[path]`; clippy needed a few `#[allow]`-free rewrites only.
  - `EglContext::from_window` and `GlxContext::from_window` have no test yet: they need a native window from T143/T144. Their `// TEST:` markers point at the headless tests that cover the shared code. Add window tests when those backends land.
  - `eglTerminate` is never called (the display is shared per process; ADR-T145-1 records it).
  - Mesa answers any `gl*` name through `eglGetProcAddress`, so the "unknown entry point" test uses a name without that prefix; the table check still catches a driver that returns null.
  - GL tests do not run under Miri; the pure helpers (`encode_vertices`, `indices_in_range`) have plain unit tests.
  - Not run here: the CI job (GL on llvmpipe) and the coverage ratchet (`scripts/coverage.sh`), which the new files may move.
