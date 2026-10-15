---
id: ADR-T28-1
title: Opt-in native Wayland backend
status: accepted
kind: decision
date: 2026-10-15
task: T28
builds_on: [ADR-T12-1, ADR-T26-1, ADR-T27-2]
amends: []
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T28-1 Opt-in native Wayland backend

## Context

draw runs on miniquad's X11 backend, on Wayland sessions through
XWayland ([[ADR-T12-1 Blocking event loop with cached frame]]). XWayland
is blurry under fractional scaling and missing on some minimal Wayland
sessions. miniquad 0.4.11 also ships a native Wayland backend, chosen by
`Platform::linux_backend`, but it is untested with draw: it has no
`wp_fractional_scale_v1`, its wake-up path for the T26 settle
([[ADR-T26-1 Reuse the document layer during pan and zoom]]) has not
been exercised, and it `dlopen`s libraries the
[[ADR-T27-2 Package formats from one staging tree and one library list|library list]]
marks as unused.

## Decision

- `DRAW_BACKEND` picks the window backend at start-up, read once by a
  pure function in `shell::backend`:
  unset, empty or `x11` → `LinuxBackend::X11Only` (the default, as
  before); `wayland` → `LinuxBackend::WaylandWithX11Fallback`; any other
  value → the default plus one `draw:` warning line on stderr. Values are
  trimmed and compared ignoring ASCII case.
- The Wayland backend is **experimental**: choosing it prints one
  `draw:` line on stderr saying so, and the README lists the variable
  under *Configuration → Experimental*. It is not a `DRAW_EXPERIMENTAL`
  feature because it is a choice between two values, not an on/off
  switch, and it is read before the window exists.
- Both backends keep the blocking event loop and the same update
  triggers; decorations stay miniquad's default
  (`ServerWithLibDecorFallback`).
- The preflight takes the backend. With `wayland` and `WAYLAND_DISPLAY`
  set, it checks the libraries of role `wayland` instead of the X11
  ones and does not need `DISPLAY`; missing ones are an error, reported
  like missing X11 libraries (the user opted in, so a silent fall back to
  X11 would hide the problem). With `wayland` but no `WAYLAND_DISPLAY`,
  miniquad falls back to X11, so the X11 check runs.
- `packaging/runtime-libs` gains two roles: `wayland` (needed by the
  Wayland backend; a library both backends need gets one line per role)
  and `optional` (loaded when present, never checked: `libdecor`, which
  draws the title bar on compositors without server-side decorations).
  Neither is a package dependency.
- The Flatpak manifest adds `--socket=wayland` and **keeps**
  `--socket=x11` rather than switching to `--socket=fallback-x11`:
  `fallback-x11` hides X11 on Wayland sessions, and X11 stays the
  default backend.

### What would make it the default

A later ADR, once the owner's checks on the target pass: text and lines
at least as sharp as XWayland at 125 % and 150 % (AC-4 of
[[T28 Experimental native Wayland backend]]), the full behaviour
checklist on GNOME and one other compositor (AC-5), and frame times
within [[ADR-T24-5 Redraw budgets]] and [[ADR-T26-2 Gesture frame budget]]
(AC-6).

## Alternatives considered

- **`WaylandOnly`** — fails hard where the Wayland connection fails;
  the fallback keeps the opt-in harmless on X11 sessions.
- **A `DRAW_EXPERIMENTAL=wayland` feature** — mixes a start-up choice
  with features toggled in the core; a named variable also leaves room
  for `x11` to stay meaningful if Wayland becomes the default.
- **A command-line flag** — draw has no argument parsing; an
  environment variable matches the other settings and works from a
  desktop entry's `Exec` with `env`.
- **Fall back to X11 silently when Wayland libraries are missing** —
  hides why the opt-in had no effect.

## Consequences

- The binary now contains the Wayland code and names its libraries;
  packages recommend them only once the package metadata says so (the
  `.deb` `recommends` and `.rpm` weak dependencies live in `Cargo.toml`,
  outside T28's files).
- Two backends to keep working; the X11 path is unchanged and stays the
  tested default.
- HiDPI may not improve until miniquad supports fractional scaling; that
  needs a follow-up task, not a fork here.

## Rollback plan

Revert T28's commits: `window_conf` goes back to the default
`LinuxBackend`, the preflight to its X11-only check, `runtime-libs` to
marking the Wayland libraries `unused` and the Flatpak manifest to
`--socket=x11` alone. A rollback ADR records why.
