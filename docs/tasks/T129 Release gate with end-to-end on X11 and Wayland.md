---
id: T129
title: Release gate with end-to-end on X11 and Wayland
status: review
wave: 65
branch: task/T129-release-gate-with-end-to-end-on-x11-and-
depends_on: [T127]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T129 Release gate with end-to-end on X11 and Wayland

## Goal

A release cannot be packaged unless `check.sh --fuzz` and the e2e suite pass on X11 and on Wayland.

## Known facts (2026-11-12)

- Owner, 2026-11-12: the release depends on the e2e suite. Today `scripts/e2e.sh` runs apart from `check.sh`; the goldens were stale for several tasks without anyone noticing.
- e2e uses a private headless sway (Wayland). X11 needs its own private server (Xvfb or Xephyr) with the same isolation rules as [[ADR-T75-1 Headless sway session for end-to-end tests]].

## Spec

Refined 2026-11-12 (unattended batch). All tests live in `self_tests` of
`tests/e2e.rs`.

- **AC-1** — `scripts/e2e.sh` takes `--backend wayland|x11|all` (default `all`); the X11 run uses a private Xvfb (`-nolisten tcp`, own `XAUTHORITY` in the private runtime directory, `DISPLAY` from `-displayfd`, no `WAYLAND_DISPLAY`) under the same resource caps.
  Tests: `script_isolates_the_session` (extended), `script_rejects_an_unknown_backend`, `script_x11_run_is_private`, `x11_guard_accepts_the_private_display`, `x11_guard_rejects_an_authority_outside_the_runtime`, `x11_guard_rejects_a_missing_display`.
- **AC-2** — `scripts/release.sh X.Y.Z` runs `check.sh --fuzz`, then the e2e suite on both backends, then `package.sh`, and refuses to package when any step fails. `--dry-run` prints the steps without running them; `DRAW_RELEASE_FAIL=<step>` makes a dry-run step fail.
  Tests: `release_dry_run_lists_the_steps_in_order`, `release_stops_on_a_failing_step`, `release_rejects_a_malformed_version`.
- **AC-3** — `package.sh` called directly prints that it is not the release path (`release.sh` sets `DRAW_RELEASE=1`), and `docs/process/Quality Gates.md` lists the release gate.
  Tests: `package_script_says_it_is_not_the_release_path`, `quality_gates_lists_the_release_gate`.
- **AC-4** — A golden that differs only by backend is a separate file (`expected.x11.ppm`, `expected.x11.snap`), preferred on X11; the goldens of both backends are regenerated and committed when the tools allow it.
  Tests: `golden_path_prefers_the_backend_file`, `golden_path_falls_back_to_the_shared_file`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `scripts/e2e.sh`, `scripts/release.sh`, `scripts/package.sh`, `tests/e2e.rs`, `tests/e2e/`
- `docs/process/Quality Gates.md`, `docs/architecture/End-to-end tests.md`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T129): specify the release gate`
- [x] tests — `test(T129): add failing tests for the release gate`
- [x] behaviour — `feat(T129): add the X11 e2e run and release.sh`
- [x] docs — `docs(T129): update the quality gates`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — implemented in an unattended batch run. `e2e.sh --backend wayland|x11|all` (default `all`); X11 runs on a private Xvfb (`-nolisten tcp`, `-displayfd`, private empty `XAUTHORITY`, no `WAYLAND_DISPLAY`) under the same caps. Input goes through `xdotool`, screenshots through ImageMagick `import -depth 8` of the app window, and the close is a `WM_DELETE_WINDOW` client message sent by the harness over the X protocol (`x11_close`), because the private server has no window manager. `release.sh` and the `package.sh` notice added; `Quality Gates.md` has a Release row.
- 2026-11-12 — AC-4: seven cases differ on X11 by about 200 to 1000 pixels (text and cursor rendering) and got `expected.x11.ppm`; all snapshots are shared. `e2e.sh` on both backends passes; `check.sh` green. The X11 server has no clipboard manager, so `clipboard` steps use the app's file hook there.
- 2026-11-12 — extra tests beyond the spec: `release_rejects_a_version_that_is_not_the_manifest_one`, `x11_intern_request_pads_the_name_to_four_bytes`, `x11_delete_request_is_a_client_message_to_the_window`, `xdotool_commands_translate_a_chord_and_text`, `backend_from_name_knows_both_and_rejects_others`. The `-displayfd` server has no cookie (the display is local-only and only this script's children know its number).
- 2026-11-12 — deviations: the end-to-end note is `docs/features/End-to-end tests.md` (not `docs/architecture/`, which has none), so it was not edited; it still describes only sway. The batch shell could not `chmod`, so `scripts/release.sh` is not executable: run `chmod +x scripts/release.sh` once (it works as `bash scripts/release.sh`). `release.sh` was only run with `--dry-run`; `package.sh` and a real release were not run. The first X11 run showed that `import` writes 16-bit PPMs, fixed with `-depth 8`.
