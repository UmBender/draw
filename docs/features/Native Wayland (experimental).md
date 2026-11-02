---
title: Native Wayland (experimental)
task: "[[T28 Experimental native Wayland backend]]"
adrs: ["[[ADR-T28-1 Opt-in native Wayland backend]]", "[[ADR-T44-1 Wayland seat devices follow the announced capabilities]]"]
tutorial: "[[28 Choosing a window backend at startup]]"
shortcuts: []
tags: [feature, experimental]
---

# Native Wayland (experimental)

## What it does

By default draw is an X11 app; on a Wayland session it runs through
Xwayland. `DRAW_BACKEND=wayland` makes it talk to the Wayland compositor
directly instead, so it also starts on sessions without Xwayland. It is
experimental: off unless asked for, announced by one line on stderr, and
falling back to X11 when the Wayland connection fails.

## How to use

| Action | How |
|--------|-----|
| Start on native Wayland | `DRAW_BACKEND=wayland draw` |
| Back to X11 (default) | unset `DRAW_BACKEND`, or `DRAW_BACKEND=x11` |
| Start despite a failed preflight | `DRAW_SKIP_PREFLIGHT=1` |

An unknown value prints a warning and keeps X11.

## How it works

`shell::backend::choose` maps the variable to a `Backend`
(`X11` → miniquad's `LinuxBackend::X11Only`, `Wayland` →
`WaylandWithX11Fallback`) and a warning. `shell::app::window_conf` reads
it once, prints the warning or the experimental notice and stores the
backend in `WindowSettings`; the `Conf` keeps the blocking event loop of
[[ADR-T12-1 Blocking event loop with cached frame]] for both.
`shell::preflight::check` takes the backend: in a Wayland session with
`wayland` it checks the libraries of role `wayland` in
`packaging/runtime-libs` (`libwayland-client`, `libwayland-egl`,
`libxkbcommon`, `libEGL`) and does not need `DISPLAY`. `libdecor` is
`optional`. The Flatpak manifest grants both the `x11` and `wayland`
sockets. See [[ADR-T28-1 Opt-in native Wayland backend]].

The vendored miniquad keeps at most one keyboard, pointer and touch
device per seat. The compositor may send `wl_seat.capabilities` again
when devices change; `miniquad::seat::device_change` then keeps the
devices it already has, releases those whose capability went away and
creates those that came back, so every key and button arrives once
([[T44 One keyboard per Wayland seat]],
[[ADR-T44-1 Wayland seat devices follow the announced capabilities]]).

## Limits

- miniquad 0.4.11 has no fractional-scale support, so text may be no
  sharper than under Xwayland at 125 % / 150 %.
- Without `libdecor` the window may have no title bar on GNOME.
- The `.deb` and `.rpm` do not yet recommend the Wayland libraries (the
  package metadata lives in `Cargo.toml`); on most desktops they are
  installed anyway.
- Not the default until the owner's checks pass on the target (AC-4–AC-6
  of [[T28 Experimental native Wayland backend]]).
