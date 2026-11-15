---
id: T168
title: Wayland connection, window and protocol states
status: todo
wave: 70
branch: task/T168-wayland-connection-window-and-protocol-
depends_on: [T142, T145]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T168 Wayland connection, window and protocol states

## Goal

Slice 1 of [[T143 Wayland backend with window, keyboard, pointer, tablet and clipboard]]: `draw-sys` connects to a Wayland compositor through `libwayland-client` (loaded with `dlopen`), opens an `xdg_toplevel` window with an OpenGL ES 3.0 context on it, follows resize, scale and close, and paces on frame callbacks. The surface protocol order is a type state.

## Known facts (2026-11-15)

- Split out of T143 by the owner before the batch (T143 was as large as [[T156 Platform migration of input and the loop]] and [[T164 Generic run over Platform and Gpu]], which blocked).
- "Our own code" means our own code over `libwayland-client`, `libwayland-egl` and EGL loaded with `dlopen` ([[2.0 Architecture proposal]] §5), through the `loader` of [[ADR-T142-1 dlopen through libc symbols and TEST markers]]. No new crate dependency ([[ADR-T127-3 The dependency list]]). Each `unsafe` primitive follows [[ADR-T127-2 The unsafe policy]].
- **Protocol code (owner decision, 2026-11-15):** generated from the protocol XML and **checked in**. The XML files draw uses (`wayland.xml`, `xdg-shell.xml`, later `tablet-unstable-v2.xml`) are copied into `crates/draw-sys/protocols/` with their licence headers; a small generator (no new dependency; the spec step chooses where it lives, e.g. a test helper or a script) writes the `wl_interface` tables and the typed request/event wrappers into `crates/draw-sys/src/wayland/generated/`. A test fails when the committed code differs from what the generator produces. The build never reads `/usr/share`.
- `EglContext::from_window` (T145) takes a `wl_display*` and a `wl_egl_window*` and has no window test yet (T145 Log): this slice adds it.
- Type states ([[2.0 Architecture proposal]] §4): a surface is `Created -> Configured -> Mapped`; attaching or committing a buffer before the first `configure` is acked does not compile. Compile-fail tests run in `scripts/compile_fail.sh`, which today covers only `draw-types` and is widened to `draw-sys` here.
- Integration tests run against a private headless sway, as `scripts/e2e.sh` does (`WLR_BACKENDS=headless`, `WLR_RENDERER=pixman`, own `XDG_RUNTIME_DIR`); a missing sway fails the test, never skips it (same rule as the Xvfb helper of T145).
- The app does not use the backend yet: wiring it into `main` and e2e is [[T171 Wayland platform in main and the e2e suite]].

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — The generator and the checked-in code: `wl_display`, `wl_registry`, `wl_compositor`, `wl_surface`, `wl_callback`, `wl_output`, `xdg_wm_base`, `xdg_surface`, `xdg_toplevel`. Test: the committed code equals the generator's output; unit tests of the generator on a small XML sample.
- **AC-2** — Connect, bind the globals, ping/pong, open a toplevel with a title and app id, follow `configure` (size), integer buffer scale from the outputs, close; an EGL context on the window renders and swaps. Integration tests against headless sway.
- **AC-3** — Frame callbacks: `schedule_update` asks for one, the next frame resolves when it arrives. Integration test.
- **AC-4** — Out-of-order requests do not compile (attach/commit before configure; acking twice). Compile-fail tests.
- **AC-5** — `WaylandPlatform` implements `Platform` for the window part (`window()`, `poll` with resize and close, `schedule_update`, `next_frame`); keyboard, pointer, tablet and clipboard return nothing yet, documented as T169/T170.

## Out of scope

- Keyboard, pointer, seat ([[T169 Wayland keyboard, pointer and seat]]); tablet and clipboard ([[T170 Wayland tablet and clipboard]]); `main` and e2e ([[T171 Wayland platform in main and the e2e suite]]); fractional scale; server-side decorations.

## Files owned

- `crates/draw-sys/src/wayland/**`, `crates/draw-sys/protocols/**`, the `pub mod wayland` line of `crates/draw-sys/src/lib.rs`
- `crates/draw-sys/tests/wayland*.rs`, `crates/draw-sys/tests/common/sway.rs`, `crates/draw-sys/tests/compile_fail/**`, `crates/draw-sys/tests/compile_pass/**`
- `scripts/compile_fail.sh` (widen to `draw-sys`), the generator, the EGL window test
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T168): specify the slice`
- [ ] tests — `test(T168): add failing tests for the Wayland window`
- [ ] models — `feat(T168): add the generated protocol and the surface states`
- [ ] behaviour — `feat(T168): implement the Wayland window`
- [ ] quality — `chore(T168): pass clippy and rustfmt`

## Log

- 2026-11-15 — split from T143 by the owner; protocol code generated and checked in (owner decision).
