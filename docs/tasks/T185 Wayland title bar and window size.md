---
id: T185
title: Wayland title bar and window size
status: in-progress
wave: 76
branch: fix/wayland-decorations-and-size
depends_on: []
adrs: ["[[ADR-T185-1 Server-side decorations through xdg-decoration]]"]
feature: "[[Native Wayland]]"
tutorial: ""
tags: [task, bug]
---

# T185 Wayland title bar and window size

## Goal

On a KDE Plasma Wayland session (and any compositor with
`zxdg_decoration_manager_v1`) the draw window has the compositor's title
bar and frame, and it opens at 1280×800 like the X11 window, so the whole
toolbar fits.

## Known facts (2026-11-24)

- Reported by the owner on 2.0.0 under KDE Plasma Wayland (Fedora 44,
  two 1920×1080 outputs at scale 1): the window has no title bar or
  frame, and the colour island of the toolbar is cut at the bottom.
- Both are regressions of [[T149 Cutover removing miniquad and macroquad]].
  miniquad asked for server-side decorations and opened the window at
  `window_conf`'s 1280×800. The own backend of
  [[T168 Wayland connection, window and protocol states]] left
  server-side decorations out of scope:
  - it never binds `zxdg_decoration_manager_v1`. KWin then treats the
    window as client-decorated and draws no frame. `wayland-info` on the
    owner's session lists `zxdg_decoration_manager_v1` version 2;
  - when the first `configure` lets the client choose (`0×0`, as KWin and
    sway send to a new window), it takes `DEFAULT_SIZE`, 640×480. `main`
    gives X11 `NATIVE_SIZE` (1280×800) but gives Wayland no size.
- The toolbar is laid out for 800 px of height (`islands_fit_the_default_window`).
  At 480 px the colour island crosses the bottom of the window.
- Headless sway (the test compositor) treats a client without the protocol
  as client-decorated (`"border": "csd"` in `swaymsg -t get_tree`), and it
  answers `set_mode(server_side)` with `configure(server_side)` for a
  tiled window. It lets an unmapped window choose its size (T168 Log).

## Spec

Decision: [[ADR-T185-1 Server-side decorations through xdg-decoration]].

- **AC-1** — Regression: on a compositor with `zxdg_decoration_manager_v1`,
  the window asks for server-side decorations before its first commit, so
  the compositor does not treat it as client-decorated.
  Test: `regression_T185_window_asks_for_server_side_decorations`
  (`crates/draw-sys/tests/wayland.rs`, headless sway, `swaymsg -t get_tree`).
- **AC-2** — The mode the compositor configures is reported:
  `Window::decoration()` is `Some(Decoration::ServerSide)` after sway's
  answer, and `None` before any answer or without the manager.
  Tests: `wayland_decoration_configure_reports_server_side`
  (`tests/wayland.rs`), `decoration_mode_maps_the_protocol_values`
  (unit test in `window.rs`).
- **AC-3** — Regression: when the first `configure` lets the client choose,
  the window takes the size given with `Window::with_size`.
  Test: `regression_T185_window_opens_at_the_requested_size`
  (`tests/wayland.rs`).
- **AC-4** — `main` opens the Wayland window with `NATIVE_SIZE`, the size
  of the X11 window: `WaylandPlatform::connect_with` takes a size.
  Tests: `main_opens_wayland_at_the_native_size` (`src/main.rs`),
  `wayland_platform_connect_with_zero_msaa_swaps_cleanly` (updated call).
- **AC-5** — The generator emits `zxdg_decoration_manager_v1` and
  `zxdg_toplevel_decoration_v1` from
  `protocols/xdg-decoration-unstable-v1.xml`, and the checked-in code
  matches. Test: `generated_code_matches_the_checked_in_files`.

## Out of scope

- A title bar on compositors without server-side decorations (GNOME's
  Mutter): that needs client-side decorations or `libdecor`
  (ADR-T185-1).
- KDE's older `org_kde_kwin_server_decoration_manager`: KWin offers
  `zxdg_decoration_manager_v1` too.
- Hiding toolbar islands that do not fit a window made shorter than
  800 px by hand (X11 behaves the same).
- A minimum window size.

## Files owned

- `crates/draw-sys/protocols/xdg-decoration-unstable-v1.xml` (new, copied
  from `wayland-protocols` with its licence header)
- `crates/draw-sys/src/wayland/generated/**`
- `crates/draw-sys/src/wayland/connection.rs`,
  `crates/draw-sys/src/wayland/window.rs`,
  `crates/draw-sys/src/wayland/platform.rs`,
  `crates/draw-sys/src/wayland/mod.rs` (re-export)
- `crates/draw-sys/tests/wayland_generator.rs`,
  `crates/draw-sys/tests/wayland.rs`
- `src/main.rs`
- `docs/features/Native Wayland.md` (window and decoration lines)
- `docs/decisions/ADR-T185-1 Server-side decorations through xdg-decoration.md`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T185): specify Wayland title bar and size acceptance criteria`
- [ ] regression — `test(T185): add regression tests for the Wayland title bar and size`
- [ ] models — `feat(T185): add the xdg-decoration protocol and the window size`
- [ ] behaviour — `fix(T185): ask for server-side decorations and the native size`
- [ ] quality — `chore(T185): pass clippy and rustfmt`
- [ ] docs — `docs(T185): update the native Wayland notes`

## Log

- 2026-11-24 — written from the owner's bug report (screenshot on KDE
  Plasma Wayland); cause found by reading `wayland/window.rs`,
  `wayland/connection.rs` and `main.rs`, and by comparing with miniquad's
  `window_conf` before T149.
