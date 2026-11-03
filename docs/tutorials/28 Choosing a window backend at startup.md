---
title: Choosing a window backend at startup
step: 28
requires: ["[[12 The app loop, idle redraw and UI]]", "[[26 Reusing a frame during a gesture]]", "[[27 Shipping a binary to other Linux machines]]"]
feature: "[[Native Wayland]]"
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
it without risking the default that is known to work — and, once it has
proven itself, how to make it the default and retire the old path
without surprising anyone.

## The concept

A backend choice made at start-up has three parts:

1. **Parse once, purely.** The setting is a string from the environment.
   A pure function from `Option<&OsStr>` to a decision is trivial to test
   for every value — including garbage, which must never crash and should
   be reported, not silently ignored.
2. **Opt in, then flip, then deprecate.** At first (T28) the new path
   was chosen only by name. Once its checks passed (T46) the default
   follows the *session* instead, so the decision function needs a
   second input, `WAYLAND_DISPLAY`. A failure to connect still drops
   back to the old path, and every run that ends up there announces
   that it goes away in the next major release:

   ```text
   DRAW_BACKEND        Wayland session                 X11 session
   unset / unknown ──▶ try Wayland ──fails──▶ X11      X11 + notice
   x11             ──▶ X11 + notice                    X11 + notice
   wayland         ──▶ try Wayland ──fails──▶ X11      X11 + notice
   ```

   Deprecating before removing gives users one release in which the old
   path still works but tells them to move.

3. **Every check follows the choice.** Code written for one backend —
   here the T27 preflight that demands `DISPLAY` and libX11 — has to ask
   which backend is coming, or it will block the very case the option
   was added for.

A second effect hides in the build: while the backend was a compile-time
constant, link-time optimisation removed the Wayland code and the names
of its libraries from the binary. Choosing at run time keeps both, so
the library list must now account for them.

## How draw implements it

- **`shell::backend`.** `choose(value, wayland_display)` trims the
  value, compares it ignoring ASCII case and returns a
  `Choice { backend, warning, notice }`; a non-UTF-8 value goes through
  `to_string_lossy` and ends up as a warning naming the session's
  default. `notice` is the one-line X11 deprecation (naming
  `REMOVAL_VERSION`, 2.0.0) whenever the result runs on X11.
  `Backend::linux_backend` maps to miniquad's `LinuxBackend`;
  `from_env` reads `DRAW_BACKEND` and `WAYLAND_DISPLAY`.
- **`shell::app::window_conf`.** Calls `from_env` once, prints the
  warning and notice, and stores the backend in `WindowSettings`. The
  `From<WindowSettings> for Conf` impl sets `linux_backend` next to
  `blocking_event_loop: true`, so the idle-sleep loop of step 12 and the
  wake-up used by the step 26 settle apply to both backends.
- **`shell::preflight::check`.** Takes the `Backend`. With
  `Backend::Wayland` *and* `WAYLAND_DISPLAY` set, the `wayland` role is
  checked and `DISPLAY` is not needed. When a Wayland library is missing,
  the check mirrors miniquad's fallback: if the X11 check (`check_x11`:
  `DISPLAY`, `required`, one `gl`) passes, the window can still open;
  only when both fail does it name the Wayland packages, the ones worth
  installing. Otherwise miniquad uses X11 anyway, so only the X11 check
  runs.
- **`packaging/runtime-libs`.** Two new roles: `wayland` and `optional`
  (`libdecor`, which only draws a title bar). `libxkbcommon` and `libEGL`
  appear twice, once per role, so the package-dependency set computed by
  `tests/packaging.rs` from `required` and `gl` is unchanged. Since T46
  the `.deb` and `.rpm` *recommend* the `wayland` and `optional`
  packages that are not dependencies already: a weak dependency the
  package manager installs by default but does not insist on.

## Try it

- On a Wayland session run `draw`, `DRAW_BACKEND=X11 draw` and
  `DRAW_BACKEND=wl draw`, and read stderr. Then
  `WAYLAND_DISPLAY= DRAW_BACKEND=wayland draw`: which backend opens, and
  why is the deprecation line printed?
- In `preflight.rs`, write a fixture where `libwayland-egl.so.1` is
  missing: with `DISPLAY` set the check passes, without it it fails.
  Which library does the message name, and why not `libX11`?
- In `preflight.rs`, add a test where `WAYLAND_DISPLAY` is set, the
  backend is `Wayland` and `libEGL.so.1` is missing from a fixture that
  lists it as `wayland`. What does the message suggest installing?
- Remove the `libdecor-0.so.0` line from `packaging/runtime-libs` and
  run `cargo test --test packaging`: the binary scan names it.

## Further reading

- [[ADR-T28-1 Opt-in native Wayland backend]]
- [[ADR-T46-1 Native Wayland by default and X11 deprecated]]
- Keep a Changelog, *Deprecated* section: announce before removing
- miniquad `src/conf.rs` (`LinuxBackend`) and `src/lib.rs` (`start`)
- Wayland fractional scaling: `wp_fractional_scale_v1` protocol
