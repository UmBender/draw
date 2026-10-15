---
id: T28
title: Experimental native Wayland backend
status: todo
wave: 21
branch: task/T28-wayland
depends_on: [T27]
adrs: []
feature: "[[Native Wayland (experimental)]]"
tutorial: "[[28 Choosing a window backend at startup]]"
tags: [task, experimental]
---

# T28 Experimental native Wayland backend

## Goal

`DRAW_BACKEND=wayland` starts draw on miniquad's native Wayland backend
instead of XWayland. It is **experimental**: opt-in only, X11 (XWayland)
stays the default of
[[ADR-T12-1 Blocking event loop with cached frame]], and the README and
`--help`-style startup message say so. The aim is a sharp canvas under
fractional scaling and a working app on Wayland sessions without
XWayland; the result on the target decides whether it ever becomes the
default (a later ADR, not this task).

Split from [[T27 Linux packaging]] (2026-10-14): changing the backend
touches the event loop and the T26 wake-up, so it does not belong in a
packaging task.

## Known risks (check in the spec step)

Read from miniquad 0.4.11 `src/native/linux_wayland.rs` on 2026-10-14:

- **HiDPI may not improve.** The backend binds `wl_output` and
  `wp_viewporter` but has no `wp_fractional_scale_v1` and no
  `wl_surface.set_buffer_scale` call, so it may render at scale 1 and be
  upscaled by the compositor — blurrier than XWayland on a scaled output.
  If so, AC-4 fails and the ADR records it; this does not block keeping
  the opt-in for sessions without XWayland.
- **Wake-up.** T26's settle relies on `schedule_update` from the main
  thread while `Moved` ([[ADR-T26-1 Reuse the document layer during pan and zoom]]).
  The Wayland loop honours `blocking_event_loop` and `update_requested`
  (lines ≈ 1254, 1322) but this has not been tested with draw.
- **New runtime libraries.** The Wayland branch `dlopen`s
  `libwayland-client`, `libwayland-egl`, `libwayland-cursor` and
  `libdecor` (fallback decorations). Today LTO removes that code because
  the backend is a constant; once it is chosen at runtime they appear in
  the binary and T27's `packaging/runtime-libs` check must list them
  (optional where the app falls back to X11).
- **Decorations.** Default `ServerWithLibDecorFallback`; GNOME has no
  server-side decorations, so without libdecor the window has no title
  bar.
- **Input parity.** Keys with modifiers, middle drag, wheel (including
  high-resolution wheel events), `Space`+drag and DPI changes must behave
  as on X11.

## Spec

To be refined in the spec step; every AC must name its tests there.
ADR expected: **ADR-T28-1 — Opt-in native Wayland backend** (env var,
fallback behaviour, why experimental, what would make it the default).

- **AC-1 — Backend choice.** A pure function maps `DRAW_BACKEND`
  (unset, `x11`, `wayland`, anything else) to a `LinuxBackend`: unset and
  `x11` → `X11Only` (unchanged default); `wayland` →
  `WaylandWithX11Fallback`; an unknown value → default plus one warning
  line on stderr. `window_conf` uses it, keeping the blocking loop.
- **AC-2 — Experimental is visible.** With the Wayland backend the app
  prints one stderr line saying the backend is experimental; the README
  documents the variable under an *Experimental* heading.
- **AC-3 — Preflight.** T27's `shell::preflight` takes the chosen
  backend: with `wayland`, a Wayland session without `DISPLAY` is no
  longer an error, and missing Wayland libraries are reported the same
  way as the X11 ones.
- **AC-4 — Sharp at fractional scale.** Owner on the target at 125 % or
  150 % scaling: compare lines and toolbar text under X11 and Wayland;
  result and screenshots noted in the *Log*.
- **AC-5 — Behaviour parity.** Owner checklist on the target with
  `DRAW_BACKEND=wayland`: idle CPU ≈ 0 %, every README shortcut, middle
  and `Space`+drag pan, wheel zoom, resize, the T26 settle sharpening
  ≈ 100 ms after a gesture without moving the mouse, window title bar
  present (GNOME and one wlroots/KDE compositor if available).
- **AC-6 — Budgets.** `DRAW_FRAME_TIMES=1` with the ADR-T24-5 scene:
  frame times within [[ADR-T24-5 Redraw budgets]] and
  [[ADR-T26-2 Gesture frame budget]], numbers in the *Log*.
- **AC-7 — Packaging.** `packaging/runtime-libs` lists the Wayland
  libraries as optional (Recommends / weak deps); the Flatpak manifest
  adds `--socket=wayland` next to `--socket=fallback-x11`.
- **AC-8 — No regressions.** With the variable unset the app is
  byte-for-byte the same backend as before; `scripts/check.sh` and the
  T14 perf tests stay green.

## Out of scope

- Making Wayland the default (needs AC-4/AC-5 results and its own ADR).
- Patching or forking miniquad (e.g. adding fractional-scale support);
  if AC-4 shows it is needed, open a follow-up task.
- macOS and Windows.

## Files owned

- `src/shell/app.rs` (`window_conf`, backend choice), `src/shell/backend.rs`
  (new, if the choice gets its own module)
- `src/shell/preflight.rs` (backend-aware checks, after T27)
- `packaging/runtime-libs`, `packaging/flatpak/*`
- `README.md` (*Experimental* section)
- new ADR `ADR-T28-1`, feature note `docs/features/Native Wayland (experimental).md`,
  tutorial `docs/tutorials/28 Choosing a window backend at startup.md`

## Subtasks (one commit each)

- [ ] spec — `docs(T28): …`
- [ ] tests — `test(T28): …`
- [ ] models — `feat(T28): …`
- [ ] behaviour — `feat(T28): …`
- [ ] quality — `chore(T28): …`
- [ ] docs — `docs(T28): …`

## Learning path

Step 28 — requires step 12 (app loop), step 26 (gesture redraw) and
step 27 (packaging).

## Log

- 2026-10-14 — Created as experimental at the owner's request, split out
  of T27. Risks above come from reading miniquad 0.4.11; nothing has been
  run on Wayland yet.
