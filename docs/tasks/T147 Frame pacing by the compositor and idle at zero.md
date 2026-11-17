---
id: T147
title: Frame pacing by the compositor and idle at zero
status: done
wave: 71
branch: task/T147-frame-pacing-by-the-compositor-and-idle-
depends_on: [T143, T144, T146]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T147 Frame pacing by the compositor and idle at zero

## Goal

Frames are paced by `wl_surface.frame` or X11 presentation timing instead of a polling loop, and an idle window uses no CPU.

## Known facts (2026-11-12)

- [[ADR-T12-1 Blocking event loop with cached frame]] keeps idle at 0 %; this keeps it while adding paced redraws.

## Spec

- **AC-1** — A redraw is requested once per compositor frame at most; with no input and no timer the process sleeps with ~0 % CPU:
  - `WaylandPlatform` paces active frames with `wl_surface.frame` and blocks in `poll()` on the Wayland socket fd and `signal::wake_fd()` with infinite timeout when idle (`next_idle`), using a short timeout only while a key is held for repeat.
  - `X11Platform` implements `skips_idle_swap() -> true` and `next_idle()`, flushing X11 and blocking in `poll()` on `XConnectionNumber` and `signal::wake_fd()` instead of a 1 ms `thread::sleep` polling loop (keeping short waits only while clipboard `INCR` state is active).
  - `app::run` skips GPU work and calls `next_idle()` when `skips_idle_swap()` is active and nothing changed (`Redraw::None`, `!frame.wake`, `!laser.fading()`).
  - Tests: recorder-based `no_redraw_without_input` (`src/shell/app.rs`), `wayland_idle_sleeps_until_wakeup` (`crates/draw-sys/tests/wayland.rs`), `x11_idle_skips_swap_and_wakes_on_scheduled_update` (`crates/draw-sys/tests/x11.rs`), and the idle CPU check `e2e_idle_cpu_is_near_zero` (`tests/e2e.rs`).
- **AC-2** — A paused or hidden window draws nothing:
  - `Platform::paused(&self) -> bool` (default `false`, forwarded by `Closing<P>` and controllable in `RecordingPlatform`).
  - `WaylandPlatform` reports `paused() == true` when `xdg_toplevel.configure` includes `XDG_TOPLEVEL_STATE_SUSPENDED` (`9`).
  - `X11Platform` reports `paused() == true` when the window is unmapped (`UnmapNotify` vs `MapNotify`) or fully obscured (`VisibilityNotify` with `VisibilityFullyObscured`).
  - `app::run` skips `frame.present(...)` and sleeps in `next_idle()` while `platform.paused()` is `true`.
  - Tests: `paused_window_draws_nothing` (`src/shell/app.rs`), `wayland_toplevel_suspended_state_pauses_the_window` (`crates/draw-sys/src/wayland/window.rs`), `x11_unmap_and_visibility_pause_the_window` (`crates/draw-sys/src/x11/event.rs` / `crates/draw-sys/src/x11/platform.rs`).

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `src/shell/app.rs`, `src/shell/gpu_native.rs`, `src/shell/recording.rs`, `crates/draw-sys/**`, `tests/e2e.rs`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T147): specify frame pacing`
- [x] tests — `test(T147): add failing tests for pacing`
- [x] behaviour — `feat(T147): pace frames by the compositor`
- [x] quality — `chore(T147): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-17 — spec refined: `crates/draw` flattened to `src/shell/{app,gpu_native,recording}.rs`, `tests/e2e.rs` added for the AC-1 idle CPU check, AC-1/AC-2 mapped to `WaylandPlatform` (`wl_surface.frame`, `xdg_toplevel` `SUSPENDED`, `poll` on fd + `signal::wake_fd`) and `X11Platform` (`skips_idle_swap`, `XConnectionNumber` `poll`, `UnmapNotify`/`VisibilityNotify`).
- 2026-11-17 — AC-1 and AC-2 implemented and verified: `signal::wake_fd()` self-pipe wakes `poll` on `SIGINT`/`SIGTERM`; `WaylandPlatform` and `X11Platform` sleep with `timeout = -1` in `next_idle` (short repeat wait only when a key is held); `X11Platform::next_frame` uses `poll` instead of 1 ms `thread::sleep`; `Platform::paused()` skips GPU draws while Wayland `xdg_toplevel` is `SUSPENDED` or X11 window is unmapped/obscured. `./scripts/check.sh` passes (`All checks passed.`, including `no_redraw_without_input`, `paused_window_draws_nothing`, `wayland_idle_sleeps_until_wakeup`, `x11_idle_skips_swap_and_wakes_on_scheduled_update`, `x11_idle_sleeps_until_wakeup`, `wayland_toplevel_suspended_state_pauses_the_window`, `x11_unmap_and_visibility_pause_the_window`), `./scripts/e2e.sh --backend all` passes (`e2e_every_case` + `e2e_idle_cpu_is_near_zero` on Wayland and X11), and `e2e_idle_cpu_is_near_zero` passes under `--platform native` on both Wayland and X11.
