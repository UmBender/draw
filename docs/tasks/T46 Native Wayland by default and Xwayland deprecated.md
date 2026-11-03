---
id: T46
title: Native Wayland by default and Xwayland deprecated
status: review
wave: 31
branch: task/T46-wayland-default
depends_on: [T28, T44, T45]
adrs: [ADR-T46-1]
feature: "[[Native Wayland]]"
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

Decisions: [[ADR-T46-1 Native Wayland by default and X11 deprecated]]
(amends [[ADR-T28-1 Opt-in native Wayland backend]]). 2.0.0 drops X11
entirely, plain X11 sessions included.

`backend::choose(value, wayland_display)` returns
`Choice { backend, warning, notice }`; a *Wayland session* is a
non-empty `WAYLAND_DISPLAY`.

- **AC-1 — Wayland by default on a Wayland session.** Unset, empty or
  blank `DRAW_BACKEND` with a Wayland session → `Backend::Wayland`, no
  warning, no notice.
  Tests (`backend.rs`): `choose_unset_on_wayland_session_is_wayland`.
- **AC-2 — X11 session: X11 with the deprecation notice.** Unset
  variable without a Wayland session (unset or empty `WAYLAND_DISPLAY`)
  → `Backend::X11`, no warning, a notice naming 2.0.0 and saying a
  Wayland session is needed.
  Tests: `choose_unset_without_wayland_session_is_x11_with_deprecation`.
- **AC-3 — `x11` prints the deprecation notice.** `DRAW_BACKEND=x11`
  (any case, trimmed) → `Backend::X11`, notice naming 2.0.0; on a
  Wayland session it also names `DRAW_BACKEND` (unset it for Wayland).
  Tests: `choose_x11_on_wayland_session_is_x11_with_deprecation`,
  `choose_ignores_case_and_surrounding_space`.
- **AC-4 — `wayland` is not experimental.** `DRAW_BACKEND=wayland` on a
  Wayland session → `Backend::Wayland`, no warning, no notice (nothing
  says *experimental*); without a Wayland session miniquad will use X11,
  so the deprecation notice is printed.
  Tests: `choose_wayland_on_wayland_session_has_no_notice`,
  `choose_wayland_without_wayland_session_warns_of_deprecation`.
- **AC-5 — Unknown values use the session default.** Unknown or
  non-UTF-8 values → the session's default backend plus one warning
  naming the variable, the value and the default.
  Tests: `choose_unknown_value_warns_and_uses_session_default`,
  `choose_non_utf8_value_warns_and_uses_session_default`,
  proptest `choose_any_value_follows_session`.
- **AC-6 — Fallback to X11.** `Backend::Wayland` maps to
  `WaylandWithX11Fallback`. Preflight on a Wayland session with
  `Backend::Wayland`: missing `wayland` libraries with a working
  Xwayland (`DISPLAY`, X11 libraries) → `Ok` (miniquad falls back);
  missing `wayland` libraries and no Xwayland → `MissingLibs` naming the
  `wayland` packages; missing `wayland` and X11 libraries → the
  `wayland` ones.
  Tests: `backend_linux_backend_maps_to_miniquad`;
  (`preflight.rs`) `check_wayland_backend_without_wayland_libs_falls_back_to_xwayland`,
  `check_wayland_backend_missing_lib_names_its_packages`,
  `check_wayland_backend_missing_both_names_wayland_libs`.
- **AC-7 — One line each.** Warning and notices start with `draw: ` and
  contain no newline. `window_conf` prints the warning and the notice.
  Tests: `warning_and_notice_are_one_line`.
- **AC-8 — Packages recommend Wayland.** `.deb` `recommends` and
  `[package.metadata.generate-rpm.recommends]` list exactly the packages
  of `wayland`/`optional` libraries that are not already dependencies;
  the Flatpak keeps `x11` and `wayland` sockets.
  Tests (`preflight.rs`): `package_metadata_recommends_wayland_libs`,
  `flatpak_manifest_allows_wayland_and_x11`.
- **AC-9 — README.** The variable table says Wayland is the default on
  Wayland sessions and X11 is deprecated for 2.0.0; native Wayland is no
  longer under *Experimental*.
  Tests (`backend.rs`): `readme_documents_wayland_default_and_x11_deprecation`.
- **AC-10 — Owner check on Hyprland with the Flatpak.** Without the
  variable draw runs natively (`hyprctl clients` shows `xwayland: 0`),
  no notice; `--env=DRAW_BACKEND=x11` prints the deprecation line and
  runs on Xwayland (`xwayland: 1`). (Manual.)
- **AC-11 — Fractional scale (T28 AC-4).** Owner compares lines and
  toolbar text at 125 % or 150 %, X11 vs Wayland; result in the *Log*.
  Does not block the default (ADR-T46-1). (Manual.)

## Out of scope

- Removing the X11 code (T50).
- Fractional-scale support in miniquad (follow-up if AC-4 needs it).

## Files owned

- `src/shell/backend.rs`, `src/shell/app.rs` (notice), `src/shell/preflight.rs`
- `Cargo.toml` (package metadata), `packaging/`
- `docs/features/Native Wayland (experimental).md` (renamed),
  tutorial 28, `README.md`, new ADR `ADR-T46-*`

## Subtasks (one commit each)

- [x] spec — `docs(T46): …`
- [x] tests — `test(T46): …`
- [x] models — `feat(T46): …`
- [x] behaviour — `feat(T46): …`
- [x] quality — `chore(T46): …`
- [x] docs — `docs(T46): …`

## Learning path

Update [[28 Choosing a window backend at startup]]; no new tutorial.

## Log

- 2026-11-02 — note written at the owner's request; X11/Xwayland to be
  removed in 2.0.0.
- 2026-11-02 — Spec: ADR-T46-1 (amends ADR-T28-1). 2.0.0 drops X11
  entirely, plain X11 sessions included. The T28 preflight error for
  missing Wayland libraries is replaced by a fallback to the X11 check,
  since miniquad falls back to X11 when `try_load` fails
  (`vendor/miniquad/src/native/linux_wayland.rs` `run`) and prints its
  own line. Fractional scale (AC-11) does not block the default.
- 2026-11-02 — Tests, models, behaviour, quality: AC-1–AC-9 covered by
  unit tests in `backend.rs` and `preflight.rs`; `scripts/check.sh`
  green. The recommends test lives in `preflight.rs` because
  `tests/packaging.rs` is not owned by T46. `Backend` lost its `Default`
  (the default now depends on the session) and `Backend::notice` (the
  notice is in `Choice`).
- 2026-11-02 — Docs: feature note renamed to *Native Wayland* with an
  `aliases` entry for the old name; tutorial 28 updated.
- Open: AC-10 (Hyprland + Flatpak) and AC-11 (fractional scale) are
  owner checks, not yet run.
- For the integrator: `CHANGELOG.md` under 1.5.0 / `[Unreleased]` —
  `### Changed`: "Native Wayland is the default on Wayland sessions;
  the .deb and .rpm recommend the Wayland libraries." and
  `### Deprecated`: "The X11 backend (XWayland on Wayland sessions and
  plain X11 sessions), removed in 2.0.0; `DRAW_BACKEND=x11` prints a
  notice." Links `[[Native Wayland (experimental)]]` remain in the
  Feature Index and in the T28, T44 and T45 notes (not owned); point
  them at `[[Native Wayland]]`. ADR-T28-1 is amended by ADR-T46-1 (add
  to the Decision Log).
