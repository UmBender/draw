---
id: ADR-T46-1
title: Native Wayland by default and X11 deprecated
status: accepted
kind: decision
date: 2026-11-02
task: T46
builds_on: [ADR-T27-2, ADR-T44-1]
amends: [ADR-T28-1]
supersedes: []
reverts: []
superseded_by:
reverted_by:
tags: [adr]
---

# ADR-T46-1 Native Wayland by default and X11 deprecated

## Context

[[ADR-T28-1 Opt-in native Wayland backend]] made native Wayland an
opt-in (`DRAW_BACKEND=wayland`) and kept X11 — Xwayland on Wayland
sessions — as the default until the owner's checks passed. Since then:
behaviour parity (T28 AC-5) and frame times (T28 AC-6, Wayland faster on
every line) passed; doubled keys were fixed by
[[ADR-T44-1 Wayland seat devices follow the announced capabilities]]
and T45 found no regression after a long idle. The owner wants 2.0.0 to
drop X11 ([[T50 Version 2.0.0 refactor]]), so 1.x must make Wayland the
normal path and warn X11 users first. Only T28 AC-4 (sharpness at
fractional scale) is still open; miniquad 0.4.11 has no
`wp_fractional_scale_v1` either way.

## Decision

This ADR replaces the *default*, the *experimental notice* and the
*preflight on a failed opt-in* of ADR-T28-1; the variable, its parsing,
the library roles and the Flatpak sockets stay as decided there.

- **Default follows the session.** `shell::backend::choose` takes the
  values of `DRAW_BACKEND` and `WAYLAND_DISPLAY`. With `DRAW_BACKEND`
  unset, empty or unknown, a Wayland session (`WAYLAND_DISPLAY`
  non-empty) gets `Backend::Wayland`
  (`LinuxBackend::WaylandWithX11Fallback`) and any other session
  `Backend::X11`. `x11` and `wayland` keep forcing their backend. An
  unknown value still prints one warning, now naming the session's
  default.
- **Native Wayland is no longer experimental.** No start-up notice for
  it; the README moves it out of *Experimental*.
- **X11 is deprecated, to be removed in 2.0.0** — both Xwayland on
  Wayland sessions and plain X11 sessions: 2.0.0 needs a Wayland
  session. Whenever draw will run on X11 (`Backend::X11`, or
  `Backend::Wayland` without a Wayland session) `Choice::notice` holds
  one `draw:` stderr line saying so and naming 2.0.0; on a Wayland
  session it also says to unset `DRAW_BACKEND`.
- **Fallback.** When the Wayland libraries are missing or the connection
  fails, miniquad falls back to X11 and prints its own
  `Trying X11 instead` line; that is the only X11 use without a draw
  notice. The preflight follows it: on a Wayland session with
  `Backend::Wayland` it checks the `wayland` libraries first; if any is
  missing but the X11 check (Xwayland `DISPLAY`, `required` and `gl`
  libraries) passes, the window opens on X11. Only when both fail does it
  report the missing `wayland` libraries — the path the user should fix.
  This drops ADR-T28-1's error for a failed explicit opt-in: miniquad's
  own line already says why Wayland was not used.
- **Packages.** The `.deb` `recommends` and the `.rpm` `recommends`
  (weak dependencies) list the packages of the `wayland` and `optional`
  libraries that are not already hard dependencies. Hard dependencies
  stay the X11 set until 2.0.0, since X11 is still the fallback.
- **Flatpak** keeps `--socket=x11` next to `--socket=wayland` until
  2.0.0: `fallback-x11` would break `DRAW_BACKEND=x11` on Wayland
  sessions while it is still supported.
- **Fractional scale (T28 AC-4)** does not block the default. The owner
  records the comparison in T46; if Wayland is blurrier at 125 % / 150 %,
  `DRAW_BACKEND=x11` is the 1.x workaround and fractional-scale support
  becomes a task before 2.0.0 removes X11.

## Alternatives considered

- **Keep X11 the default until 2.0.0** — 2.0.0 would switch backends and
  remove the fallback in one release, with no 1.x period where most
  users already run on Wayland.
- **Wayland default without the deprecation notice** — users forcing
  `x11` or on X11 sessions would learn of the removal only when it
  happens.
- **Keep the preflight error for missing Wayland libraries** — with
  Wayland the default, every Wayland session lacking `libwayland-egl`
  would refuse to start although Xwayland works.
- **Deprecate only Xwayland, keep plain X11 sessions in 2.0.0** — keeps
  the whole X11 backend and its pen code alive for few users; T50's goal
  is one backend.
- **`--socket=fallback-x11` now** — hides X11 on Wayland sessions while
  `DRAW_BACKEND=x11` is still supported.

## Consequences

- Most Linux users move to native Wayland on upgrade; Xwayland users
  see one line per start until they unset the variable.
- Wheel scaling (`wheel_units_per_notch`) and the preflight follow the
  new default automatically through `backend::from_env`.
- Missing Wayland libraries no longer stop draw when Xwayland works; the
  user sees miniquad's fallback line instead.
- `CHANGELOG.md` gets a `### Changed` line (Wayland default) and a
  `### Deprecated` line (X11 removed in 2.0.0) at integration.

## Rollback plan

Revert T46's commits: `choose` goes back to X11 for an unset variable
and the experimental notice, the preflight to the T28 error, the package
metadata to no `recommends`. A rollback ADR records why (e.g. AC-4 shows
Wayland unusable at fractional scale).
