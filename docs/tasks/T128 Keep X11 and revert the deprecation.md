---
id: T128
title: Keep X11 and revert the deprecation
status: todo
wave: 65
branch: task/T128-keep-x11-revert-the-deprecation
depends_on: [T127]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T128 Keep X11 and revert the deprecation

## Goal

X11 is a supported backend again: the deprecation of ADR-T46-1 is reverted and the docs, packaging and defaults say so.

## Known facts (2026-11-12)

- Owner, 2026-11-12: users on X11 must keep working. This reverses goal 1 of [[T50 Version 2.0.0 refactor]].
- [[ADR-T46-1 Native Wayland by default and X11 deprecated]] made native Wayland the default and marked X11 deprecated; the default (Wayland first, X11 fallback) can stay.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `ADR-T128-1` (`kind: rollback`, `reverts` the deprecation part of ADR-T46-1) says X11 is supported, tested and not scheduled for removal; the Wayland-first default is kept.
  Test: `docs` consistency test `x11_is_not_marked_deprecated` over README, CHANGELOG notes and the feature notes.
- **AC-2** — The README, the Flatpak finish arguments, the preflight messages and the `--help` text no longer call X11 deprecated; the `x11` socket stays.
  Tests: the existing preflight tests, updated; `tests/ci.rs` packaging checks.
- **AC-3** — Both backends are in the e2e matrix by [[T129 Release gate with end-to-end on X11 and Wayland]].

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `docs/decisions/ADR-T128-1*`, `docs/features/Native Wayland.md`, `README.md`, `packaging/`, `src/shell/preflight.rs`, `src/shell/backend.rs` (messages only)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T128): specify keeping X11`
- [ ] behaviour — `fix(T128): stop calling X11 deprecated`
- [ ] docs — `docs(T128): update the notes`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
