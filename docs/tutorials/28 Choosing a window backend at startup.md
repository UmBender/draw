---
title: Choosing a window backend at startup
step: 28
requires: ["[[12 The app loop, idle redraw and UI]]", "[[26 Reusing a frame during a gesture]]", "[[27 Shipping a binary to other Linux machines]]"]
feature: "[[Native Wayland (experimental)]]"
code: ["src/shell/backend.rs", "src/shell/app.rs", "src/shell/preflight.rs", "packaging/runtime-libs", "packaging/flatpak/io.github.UmBender.draw.yml"]
tags: [tutorial]
---

# Choosing a window backend at startup

> **Learning path step 28.** Requires: step 12 (app loop), step 26
> (gesture redraw), step 27 (packaging).

## Why this matters here

A Linux desktop today is usually a Wayland compositor that also runs
Xwayland, an X server for older apps. draw has always been one of those
older apps. That works, but Xwayland scales X11 windows as bitmaps under
fractional scaling, and some minimal sessions do not run it at all.
miniquad has a native Wayland backend too; the question is how to offer
it without risking the default that is known to work.

## The concept

A backend choice made at start-up has three parts:

1. **Parse once, purely.** The setting is a string from the environment.
   A pure function from `Option<&OsStr>` to a decision is trivial to test
   for every value — including garbage, which must never crash and should
   be reported, not silently ignored.
2. **Opt in, fall back.** The new path is chosen only by name, and a
   failure to connect drops back to the old one:

   ```text
   DRAW_BACKEND unset / x11 ──▶ X11Only
   DRAW_BACKEND=wayland     ──▶ try Wayland ──fails──▶ X11
   anything else            ──▶ X11Only + warning
   ```

3. **Every check follows the choice.** Code written for one backend —
   here the T27 preflight that demands `DISPLAY` and libX11 — has to ask
   which backend is coming, or it will block the very case the option
   was added for.

A second effect hides in the build: while the backend was a compile-time
constant, link-time optimisation removed the Wayland code and the names
of its libraries from the binary. Choosing at run time keeps both, so
the library list must now account for them.

## How draw implements it

- **`shell::backend`.** `choose` trims the value, compares it ignoring
  ASCII case and returns a `Choice { backend, warning }`; a non-UTF-8
  value goes through `to_string_lossy` and ends up as a warning.
  `Backend::linux_backend` maps to miniquad's `LinuxBackend`;
  `Backend::notice` is the one-line experimental notice. `from_env` reads
  `DRAW_BACKEND`.
- **`shell::app::window_conf`.** Calls `from_env` once, prints the
  warning or notice, and stores the backend in `WindowSettings`. The
  `From<WindowSettings> for Conf` impl sets `linux_backend` next to
  `blocking_event_loop: true`, so the idle-sleep loop of step 12 and the
  wake-up used by the step 26 settle apply to both backends.
- **`shell::preflight::check`.** Takes the `Backend`. `native_wayland` is
  true only with `Backend::Wayland` *and* `WAYLAND_DISPLAY` set; then
  `DISPLAY` is not needed and the `wayland` role is checked instead of
  `required` and `gl`. Otherwise miniquad would fall back to X11 anyway,
  so the X11 check runs unchanged.
- **`packaging/runtime-libs`.** Two new roles: `wayland` and `optional`
  (`libdecor`, which only draws a title bar). `libxkbcommon` and `libEGL`
  appear twice, once per role, so the package-dependency set computed by
  `tests/packaging.rs` from `required` and `gl` is unchanged.

## Try it

- Run `DRAW_BACKEND=wayland draw` and `DRAW_BACKEND=Wayland draw`, then
  `DRAW_BACKEND=wl draw`, and read stderr.
- In `preflight.rs`, add a test where `WAYLAND_DISPLAY` is set, the
  backend is `Wayland` and `libEGL.so.1` is missing from a fixture that
  lists it as `wayland`. What does the message suggest installing?
- Remove the `libdecor-0.so.0` line from `packaging/runtime-libs` and
  run `cargo test --test packaging`: the binary scan names it.

## Further reading

- [[ADR-T28-1 Opt-in native Wayland backend]]
- miniquad `src/conf.rs` (`LinuxBackend`) and `src/lib.rs` (`start`)
- Wayland fractional scaling: `wp_fractional_scale_v1` protocol
