---
title: Native Wayland
aliases: ["Native Wayland (experimental)"]
task: "[[T46 Native Wayland by default and Xwayland deprecated]]"
adrs: ["[[ADR-T28-1 Opt-in native Wayland backend]]", "[[ADR-T44-1 Wayland seat devices follow the announced capabilities]]", "[[ADR-T46-1 Native Wayland by default and X11 deprecated]]"]
tutorial: "[[28 Choosing a window backend at startup]]"
shortcuts: []
tags: [feature]
---

# Native Wayland

## What it does

On a Wayland session draw talks to the compositor directly, with no
variable to set. If the Wayland libraries are missing or the connection
fails, it falls back to X11 through Xwayland. X11 — Xwayland on Wayland
sessions and plain X11 sessions — is **supported** and not scheduled for
removal ([[ADR-T128-1 X11 stays supported]], which rolls back the X11
part of ADR-T46-1); starting on X11 prints no notice. Introduced as
an opt-in by [[T28 Experimental native Wayland backend]], made the default
by [[T46 Native Wayland by default and Xwayland deprecated]].

## How to use

| Action | How |
|--------|-----|
| Native Wayland (default on Wayland sessions) | `draw` |
| Force X11 / Xwayland | `DRAW_BACKEND=x11 draw` |
| Force Wayland | `DRAW_BACKEND=wayland draw` |
| Start despite a failed preflight | `DRAW_SKIP_PREFLIGHT=1` |

An unknown value prints a warning and uses the session's default.

## How it works

`shell::backend::choose(value, wayland_display)` maps `DRAW_BACKEND` and
`WAYLAND_DISPLAY` to a `Choice { backend, warning, notice }`:
unset/empty/unknown → `Wayland` on a Wayland session, `X11` elsewhere;
`x11` and `wayland` force theirs. `Wayland` is miniquad's
`WaylandWithX11Fallback`, `X11` is `X11Only`. `notice` is the
information line, always `None` since X11 is supported (ADR-T128-1).
`shell::app::window_conf` prints the warning and notice and keeps the blocking event loop of
[[ADR-T12-1 Blocking event loop with cached frame]] for both backends.

`shell::preflight::check` on a Wayland session with `Wayland` checks the
libraries of role `wayland` in `packaging/runtime-libs`
(`libwayland-client`, `libwayland-egl`, `libxkbcommon`, `libEGL`) and
needs no `DISPLAY`. If one is missing but the X11 check passes, the
window opens on X11 (miniquad prints `Trying X11 instead`); if both fail
it names the Wayland packages. `libdecor` is `optional`. The `.deb` and
`.rpm` *recommend* the Wayland packages; the Flatpak grants both `x11`
and `wayland` sockets. See
[[ADR-T46-1 Native Wayland by default and X11 deprecated]] and
[[ADR-T128-1 X11 stays supported]].

The vendored miniquad keeps at most one keyboard, pointer and touch
device per seat. The compositor may send `wl_seat.capabilities` again
when devices change; `miniquad::seat::device_change` then keeps the
devices it already has, releases those whose capability went away and
creates those that came back, so every key and button arrives once
([[T44 One keyboard per Wayland seat]],
[[ADR-T44-1 Wayland seat devices follow the announced capabilities]]).

A keymap with no usable content (`no_keymap`, size 0, or one that does not
compile) is ignored and the last good one stays, so draw keeps running and
typing ([[T79 Wayland keyboard without a keymap]],
[[ADR-T79-1 Ignore an unusable Wayland keymap]]).

## Limits

- miniquad 0.4.11 has no fractional-scale support, so text may be no
  sharper than under Xwayland at 125 % / 150 %; `DRAW_BACKEND=x11` is
  the 1.x workaround (T46 AC-11).
- Without `libdecor` the window may have no title bar on GNOME.
- A fallback to X11 caused by missing libraries or a failed connection
  prints miniquad's line only.
