---
id: T168
title: Wayland connection, window and protocol states
status: done
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
- **Protocol code (owner decision, 2026-11-15):** generated from the protocol XML and **checked in**. The XML files draw uses (`wayland.xml`, `xdg-shell.xml`, and `tablet-v2.xml` for T170) are in `crates/draw-sys/protocols/` (copied in by the owner after the first batch blocked on it) with their licence headers; a small generator (no new dependency; the spec step chooses where it lives, e.g. a test helper or a script) writes the `wl_interface` tables and the typed request/event wrappers into `crates/draw-sys/src/wayland/generated/`. A test fails when the committed code differs from what the generator produces. The build never reads `/usr/share`.
- `EglContext::from_window` (T145) takes a `wl_display*` and a `wl_egl_window*` and has no window test yet (T145 Log): this slice adds it.
- Type states ([[2.0 Architecture proposal]] §4): a surface is `Created -> Configured -> Mapped`; attaching or committing a buffer before the first `configure` is acked does not compile. Compile-fail tests run in `scripts/compile_fail.sh`, which today covers only `draw-types` and is widened to `draw-sys` here.
- Integration tests run against a private headless sway, as `scripts/e2e.sh` does (`WLR_BACKENDS=headless`, `WLR_RENDERER=pixman`, own `XDG_RUNTIME_DIR`); a missing sway fails the test, never skips it (same rule as the Xvfb helper of T145).
- The app does not use the backend yet: wiring it into `main` and e2e is [[T171 Wayland platform in main and the e2e suite]].

## Spec

Decisions: [[ADR-T168-1 Generated Wayland protocol code and surface type states]]. Test files: `tests/wayland_generator.rs` (generator, AC-1), `tests/wayland.rs` (integration on headless sway, AC-2/3/5), `tests/compile_fail/` and `tests/compile_pass/` (AC-4).

- **AC-1** — The generator and the checked-in code: `wl_display`, `wl_registry`, `wl_compositor`, `wl_surface`, `wl_callback`, `wl_output`, `xdg_wm_base`, `xdg_surface`, `xdg_toplevel`. Tests: `generated_code_matches_the_checked_in_files`, `parser_reads_interfaces_requests_and_events`, `generator_emits_signature_for_new_id_and_object_args`, `generator_skips_wrappers_for_unknown_interfaces`.
- **AC-2** — Connect, bind the globals, ping/pong, open a toplevel with a title and app id, follow `configure` (size), integer buffer scale from the outputs, close; an EGL context on the window renders and swaps. Integration tests against headless sway: `wayland_connect_binds_the_globals`, `wayland_window_configures_with_a_size`, `wayland_window_sets_title_and_app_id` (seen through `swaymsg -t get_tree`), `wayland_egl_context_renders_and_swaps`, `wayland_resize_event_follows_the_compositor`, `wayland_close_is_reported`, `wayland_output_scale_is_an_integer`, `wayland_missing_server_is_an_error_value`.
- **AC-3** — Frame callbacks: `schedule_update` asks for one, the next frame resolves when it arrives. Integration tests: `wayland_frame_callback_resolves_next_frame`, `wayland_schedule_update_requests_a_frame`.
- **AC-4** — Out-of-order requests do not compile (attach/commit before configure; acking twice). Compile-fail tests: `compile_fail/wayland_commit_before_configure.rs`, `wayland_attach_before_configure.rs`, `wayland_ack_twice.rs`; `compile_pass/wayland_full_sequence.rs`.
- **AC-5** — `WaylandPlatform` implements `Platform` for the window part (`window()`, `poll` with resize and close, `schedule_update`, `next_frame`); keyboard, pointer, tablet and clipboard return nothing yet, documented as T169/T170. `RawEvent` has no close variant and `platform.rs` is not owned here, so close is `close_requested()` (ADR-T168-1). Tests: `wayland_platform_window_reports_metrics`, `wayland_platform_input_parts_return_nothing`.

## Out of scope

- Keyboard, pointer, seat ([[T169 Wayland keyboard, pointer and seat]]); tablet and clipboard ([[T170 Wayland tablet and clipboard]]); `main` and e2e ([[T171 Wayland platform in main and the e2e suite]]); fractional scale; server-side decorations.

## Files owned

- `crates/draw-sys/src/wayland/**`, `crates/draw-sys/protocols/**`, the `pub mod wayland` line of `crates/draw-sys/src/lib.rs`
- `crates/draw-sys/tests/wayland*.rs`, `crates/draw-sys/tests/common/sway.rs`, `crates/draw-sys/tests/compile_fail/**`, `crates/draw-sys/tests/compile_pass/**`
- `scripts/compile_fail.sh` (widen to `draw-sys`), the generator, the EGL window test
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T168): specify the slice`
- [x] tests — `test(T168): add failing tests for the Wayland window`
- [x] models — `feat(T168): add the generated protocol and the surface states`
- [x] behaviour — `feat(T168): implement the Wayland window`
- [x] quality — `chore(T168): pass clippy and rustfmt`

## Log

- 2026-11-15 — split from T143 by the owner; protocol code generated and checked in (owner decision).
- 2026-11-15 — BLOCKED (batch): the protocol XML cannot be copied into `crates/draw-sys/protocols/`: `cp /usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml …`, `ls` of `/usr/share/wayland*` and Read outside the repo are all refused (no allow rule), and the generator (AC-1) needs the XML. Recommended fix: allow `cp /usr/share/wayland*/*` (or copy `wayland.xml` and `xdg-shell.xml` by hand into `crates/draw-sys/protocols/`) and confirm `wayland-protocols-devel` is installed; then re-run.
- 2026-11-15 — unblocked by the owner: `wayland.xml`, `xdg-shell.xml` and `tablet-v2.xml` copied into `crates/draw-sys/protocols/` (commit on this branch).
- 2026-11-15 — done (batch): all five steps committed; `scripts/check.sh` and `scripts/compile_fail.sh` green; 12 sway tests, 5 generator tests, 3 compile-fail and 1 compile-pass case.
  - Deviations: (1) the tests commit does not compile `tests/wayland.rs` until the models/behaviour steps (new `draw_sys::wayland` API), as T163/T165 did; the generator test failed for the right reason. (2) `RawEvent` has no close variant and `platform.rs` is not owned: close is `WaylandPlatform::close_requested()` (ADR-T168-1). (3) Sway lets an unmapped window choose its size (the fallback 640x480 is used); the configure with 800x600 arrives after the first present, so `wayland_window_configures_with_a_size` waits for it. (4) `next_frame` blocks inside `poll` on the display fd (no reactor yet); the frame wait is capped at 500 ms. (5) No feature or tutorial note: the task has no `feature` or `tutorial`. (6) `rustfmt` skips `wayland::generated` (`#[rustfmt::skip]` on the module).
  - Not done here, by design: wiring into `main`, keyboard, pointer, tablet, clipboard (T169 to T171). Ping/pong is implemented but not exercised by a test (sway rarely pings).
- 2026-11-15 — unblocked by the owner: rerun after triage (EGL display of a Wayland connection is never terminated).
  - Done: regression test `wayland_many_connections_in_sequence_and_threads` (50 sequential + 4×12 threaded connections with a swap each) and `eglTerminate` on drop of a Wayland `EglContext`, before `wl_display_disconnect` ([[ADR-T168-2 Terminate the EGL display of a Wayland connection]]). In about 40 runs of `cargo test -p draw-sys --test wayland` there was no `invalid object` or `roundtrip` error any more.
- 2026-11-16 — unblocked by the owner (night 3): the SIGSEGV was Mesa's `eglInitialize` racing `eglTerminate` (a `dlclose` of the Vulkan driver) in another thread. Fix: one process-wide lock in `egl.rs` around every `eglInitialize` and the whole drop of an `EglContext` ([[ADR-T168-3 Serialise EGL display initialisation and termination]]); `scripts/repeat.sh` added. Proof: 70 consecutive green runs of the `wayland` test binary; `scripts/check.sh` green.
- 2026-11-15 — (superseded, see the line above) BLOCKED (batch): the same runs hit a new failure, `SIGSEGV` of the test process, 2 times in about 40 runs (one silent, one with `signal: 11`), so the 30-clean-runs proof is not met and `scripts/check.sh` cannot be called green. It may be the same stale-state bug showing differently or a fault introduced by `eglTerminate`; it is not known which, because `gdb` is not allowed in the batch (`gdb -batch -ex run -ex bt --args target/debug/deps/wayland-<hash> --quiet` was refused). Recommended fix: allow `gdb -batch …` (or run it by hand), take the backtrace of the segfault, and check whether it also happens with the `eglTerminate` commit reverted; suspects are Mesa's teardown of its wayland proxies at `eglTerminate` racing with other threads, or the order `eglTerminate` → `wl_egl_window_destroy` in `GlPart` (the terminate runs in `EglContext::drop`, before `NativeWindow` is dropped).
