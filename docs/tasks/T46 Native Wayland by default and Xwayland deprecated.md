---
id: T46
title: Native Wayland by default and Xwayland deprecated
status: todo
wave: 31
branch: task/T46-wayland-default
depends_on: [T28, T44, T45]
adrs: []
feature: "[[Native Wayland (experimental)]]"
tutorial: "[[28 Choosing a window backend at startup]]"
tags: [task]
---

# T46 Native Wayland by default and Xwayland deprecated

## Goal

On a Wayland session draw talks to the compositor directly without any
variable, and the native Wayland backend stops being experimental. The
X11 backend (Xwayland on Wayland sessions, plain X11 elsewhere) still
works but is deprecated: choosing it prints a notice saying it will be
removed in **2.0.0**, and the docs and changelog say the same. The
removal itself is [[T50 Version 2.0.0 refactor]].

## Known facts (2026-11-02)

- Today `shell::backend::choose` maps an unset `DRAW_BACKEND` to `X11`
  (`LinuxBackend::X11Only`); `DRAW_BACKEND=wayland` gives
  `WaylandWithX11Fallback` and prints an experimental notice
  ([[ADR-T28-1 Opt-in native Wayland backend]]).
- T28 left two checks open: **AC-4** (sharpness at 125 % / 150 %
  fractional scale, X11 vs Wayland; miniquad 0.4.11 has no
  fractional-scale support) was deferred, and T28 said making Wayland
  the default needs AC-4/AC-5 results and its own ADR.
- [[T45 Doubled keys after a long idle on Wayland]] must be settled
  first: a default backend that types letters twice is worse than
  Xwayland.
- Known Wayland limits (feature note): no title bar on GNOME without
  `libdecor`; `.deb`/`.rpm` do not recommend the Wayland libraries.
- The Flatpak grants both `x11` and `wayland` sockets; keep `x11` until
  2.0.0, or `fallback-x11`.

## Proposal (settle in the spec step)

1. New ADR superseding the default of ADR-T28-1: on a Wayland session
   (`WAYLAND_DISPLAY` set) an unset `DRAW_BACKEND` means `wayland`, with
   fallback to X11 when the connection fails (that fallback is the only
   silent X11 use left). Without a Wayland session (plain X11) X11 stays
   the only choice, also deprecated for 2.0.0 (decide in the ADR whether
   2.0.0 drops X11 sessions entirely or only Xwayland on Wayland
   sessions).
2. `DRAW_BACKEND=x11` keeps working and prints one deprecation line on
   stderr: X11/Xwayland is deprecated and will be removed in draw 2.0.0.
   The experimental notice for `wayland` goes away.
3. Run the deferred T28 AC-4 check (fractional scale) before flipping;
   if Wayland is blurrier, record it and decide in the ADR whether it
   blocks the default.
4. Preflight: on a Wayland session check the `wayland` role libraries
   first; `.deb`/`.rpm` recommend the Wayland libraries.
5. Docs: rename the feature note to *Native Wayland* (drop
   *experimental* and the tag), update the backend tutorial, the
   README, and add a `### Deprecated` entry in `CHANGELOG.md` for the
   next minor release (1.5.0).

## Spec

To be written in the spec step. Expected ACs: unset variable on a
Wayland session chooses Wayland (`backend::choose` tests); unset on an
X11-only session chooses X11 with the deprecation notice; `x11` prints
the notice; `wayland` prints no experimental notice; failed Wayland
connection falls back to X11; owner check on Hyprland with the Flatpak.

## Out of scope

- Removing the X11 code (T50).
- Fractional-scale support in miniquad (follow-up if AC-4 needs it).

## Files owned

- `src/shell/backend.rs`, `src/shell/app.rs` (notice), `src/shell/preflight.rs`
- `Cargo.toml` (package metadata), `packaging/`
- `docs/features/Native Wayland (experimental).md` (renamed),
  tutorial 28, `README.md`, new ADR `ADR-T46-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T46): …`
- [ ] tests — `test(T46): …`
- [ ] models — `feat(T46): …`
- [ ] behaviour — `feat(T46): …`
- [ ] quality — `chore(T46): …`
- [ ] docs — `docs(T46): …`

## Learning path

Update [[28 Choosing a window backend at startup]]; no new tutorial.

## Log

- 2026-11-02 — note written at the owner's request; X11/Xwayland to be
  removed in 2.0.0.
