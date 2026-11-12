---
id: T129
title: Release gate with end-to-end on X11 and Wayland
status: in-progress
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

- [ ] spec — `docs(T129): specify the release gate`
- [ ] tests — `test(T129): add failing tests for the release gate`
- [ ] behaviour — `feat(T129): add the X11 e2e run and release.sh`
- [ ] docs — `docs(T129): update the quality gates`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
