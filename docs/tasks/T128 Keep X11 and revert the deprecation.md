---
id: T128
title: Keep X11 and revert the deprecation
status: done
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

- **AC-1** — `ADR-T128-1` (`kind: rollback`, `reverts` ADR-T46-1) says X11 is supported, tested and not scheduled for removal; the Wayland-first default is kept.
  Test: `x11_is_not_marked_deprecated` in `src/shell/backend.rs` over README, the Flatpak manifest, `packaging/runtime-libs` and the feature note (no "deprecat" word).
- **AC-2** — README, Flatpak manifest, runtime-libs and backend messages no longer call X11 deprecated; no deprecation line is printed; the `x11` socket stays. Preflight and `--help` had no such wording.
  Tests: `choose_x11_on_wayland_session_is_x11_without_notice`, `choose_unset_without_wayland_session_is_x11_without_notice`, `choose_wayland_without_wayland_session_has_no_notice`, `readme_documents_wayland_default_and_x11_supported` (replace the old deprecation tests); `tests/packaging.rs` stays green.
- **AC-3** — Both backends are in the e2e matrix by [[T129 Release gate with end-to-end on X11 and Wayland]].

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `docs/decisions/ADR-T128-1*`, `docs/features/Native Wayland.md`, `README.md`, `packaging/`, `src/shell/preflight.rs`, `src/shell/backend.rs` (messages only)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T128): specify keeping X11`
- [x] behaviour — `fix(T128): stop calling X11 deprecated`
- [x] docs — `docs(T128): update the notes`

## Log

- 2026-11-12 — done in the batch run; `scripts/check.sh` green. `ADR-T128-1` (rollback of ADR-T46-1) added as `accepted`. The `Choice::notice` field stays, always `None`, because `src/shell/app.rs` (not owned) still prints it; its doc comment there still says "X11 deprecation notice" and `REMOVAL_VERSION` is gone — a follow-up (T131 or the backend rewrite) can drop the field and fix the comment. Preflight and `--help` had no deprecation wording. AC-3 is T129's. No separate models/quality commits: no new types, nothing to restructure.
- Owner to check by hand: the README wording (Window backend section).
- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
