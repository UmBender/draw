---
id: T129
title: Release gate with end-to-end on X11 and Wayland
status: todo
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

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `scripts/e2e.sh` takes `--backend wayland|x11|all`; the X11 run uses a private Xvfb with no access to the desktop and the same resource caps.
  Tests: the harness's own tests for argument parsing and isolation.
- **AC-2** — `scripts/release.sh X.Y.Z` runs `check.sh --fuzz`, then the e2e suite on both backends, then `package.sh`, and refuses to package when any step fails.
  Test: a dry-run mode with injected failures (`release_stops_on_a_failing_step`).
- **AC-3** — `package.sh` called directly prints that it is not the release path, and `docs/process/Quality Gates.md` lists the release gate.
- **AC-4** — The goldens of both backends are regenerated and committed; a golden that differs only by backend is a separate file.

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
