---
id: T28
title: Experimental native Wayland backend
status: in-progress
wave: 21
branch: task/T28-wayland
depends_on: [T27]
adrs: [ADR-T28-1]
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

Decision: [[ADR-T28-1 Opt-in native Wayland backend]]. Spec-step findings
against miniquad 0.4.11: the Wayland branch loads `libwayland-client`,
`libwayland-egl`, `libxkbcommon`, `libEGL` and, for decorations,
`libdecor-0` — not `libwayland-cursor`. `WaylandWithX11Fallback` falls
back to X11 when the Wayland connection or a library fails. On Wayland
`linux_wm_class` becomes the `app_id`.

- **AC-1 — Backend choice.** `shell::backend::choose(value)` maps the
  value of `DRAW_BACKEND` to a `Choice { backend, warning }`; values are
  trimmed and ASCII-case-insensitive. Unset, empty and `x11` →
  `Backend::X11` (→ `LinuxBackend::X11Only`); `wayland` →
  `Backend::Wayland` (→ `WaylandWithX11Fallback`); anything else →
  `X11` plus one warning line. `window_conf()` reads the variable once,
  prints the warning and stores the backend in `WindowSettings`; the
  `Conf` keeps the blocking loop for both.
  Tests (`backend.rs`): `choose_unset_or_empty_is_x11_without_warning`,
  `choose_x11_is_x11_without_warning`,
  `choose_wayland_is_wayland_without_warning`,
  `choose_ignores_case_and_surrounding_space`,
  `choose_unknown_value_warns_and_keeps_x11`,
  `choose_any_value_warns_unless_known` (proptest),
  `backend_linux_backend_maps_to_miniquad`.
  Tests (`app.rs`): `window_settings_wayland_keeps_blocking_loop`.
- **AC-2 — Experimental is visible.** `Backend::notice()` is one stderr
  line containing "experimental" for `Wayland` and `None` for `X11`;
  `window_conf()` prints it. The README lists `DRAW_BACKEND` in the
  variable table and under *Experimental*.
  Tests (`backend.rs`): `backend_notice_only_for_wayland`,
  `warning_and_notice_are_one_line`,
  `readme_documents_draw_backend_as_experimental`.
- **AC-3 — Preflight.** `preflight::check(env, backend, libs, found)`.
  With `Backend::Wayland` and `WAYLAND_DISPLAY` set, `DISPLAY` is not
  needed and the libraries of the new role `wayland` are checked instead
  of `required`/`gl`; missing ones are `Problem::MissingLibs` with their
  packages. With `Wayland` but no `WAYLAND_DISPLAY` the X11 check runs.
  New role `optional` is never checked. `run()` reads the backend with
  `backend::from_env()`.
  Tests (`preflight.rs`): `parse_runtime_libs_reads_wayland_and_optional_roles`,
  `check_wayland_backend_without_display_is_ok`,
  `check_wayland_backend_ignores_x11_libs`,
  `check_wayland_backend_missing_lib_names_its_packages`,
  `check_wayland_backend_without_wayland_session_checks_x11`,
  `check_x11_backend_ignores_wayland_libs`,
  `check_optional_libs_are_not_needed`; existing X11 tests updated to
  pass `Backend::X11`.
- **AC-4 — Sharp at fractional scale.** Owner on the target at 125 % or
  150 % scaling: compare lines and toolbar text under X11 and Wayland;
  result and screenshots noted in the *Log*. (Manual.)
- **AC-5 — Behaviour parity.** Owner checklist on the target with
  `DRAW_BACKEND=wayland`: idle CPU ≈ 0 %, every README shortcut, middle
  and `Space`+drag pan, wheel zoom, resize, the T26 settle sharpening
  ≈ 100 ms after a gesture without moving the mouse, window title bar
  present (GNOME and one wlroots/KDE compositor if available), launcher
  icon matched by `app_id`. (Manual.)
- **AC-6 — Budgets.** `DRAW_FRAME_TIMES=1` with the ADR-T24-5 scene:
  frame times within [[ADR-T24-5 Redraw budgets]] and
  [[ADR-T26-2 Gesture frame budget]], numbers in the *Log*. (Manual.)
- **AC-7 — Packaging.** `packaging/runtime-libs` lists
  `libwayland-client`, `libwayland-egl`, `libxkbcommon` and `libEGL` as
  `wayland` and `libdecor-0` as `optional`; none of them becomes a
  package dependency (T27's `tests/packaging.rs` stays green). The
  Flatpak manifest adds `--socket=wayland` and keeps `--socket=x11`
  (ADR-T28-1: `fallback-x11` would hide X11, still the default).
  `.deb` Recommends / `.rpm` weak deps need `Cargo.toml`, which T28 does
  not own — reported for a follow-up.
  Tests (`preflight.rs`): `embedded_runtime_libs_list_wayland_libs`,
  `flatpak_manifest_allows_wayland_and_x11`; `tests/packaging.rs`
  (`runtime_libs_names_every_library_the_binary_mentions`,
  `deb_depends_match_runtime_libs`, `rpm_requires_match_runtime_libs`).
- **AC-8 — No regressions.** With the variable unset the `Conf` has
  `LinuxBackend::X11Only` and the same platform settings as before;
  `scripts/check.sh` and the T14 perf tests stay green.
  Tests (`app.rs`): `window_settings_x11_is_x11_only_with_blocking_loop`,
  existing `window_conf_sets_wm_class`.

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

- [x] spec — `docs(T28): …`
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
