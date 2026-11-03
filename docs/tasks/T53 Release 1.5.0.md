---
id: T53
title: Release 1.5.0
status: todo
wave: 33
branch: chore/release-1.5.0
depends_on: [T46, T47, T48, T49, T51, T52]
adrs: []
feature: ""
tutorial: ""
tags: [task, release]
---

# T53 Release 1.5.0

## Goal

draw 1.5.0 is tagged and packaged: the last release before the 2.0.0
refactor ([[T50 Version 2.0.0 refactor]]), with every feature 2.0.0
will have. Native Wayland by default with X11 deprecated (T46), the
diagnostics log (T47), the Ctrl-C and long-running fixes (T48, T49),
build-time themes (T51), and text in shapes by default (T52).

## Known facts (2026-11-02)

- The current version is 1.4.3. The release steps are in
  [[Commit Convention#Versions and changelog]]: `[Unreleased]` becomes
  `## [1.5.0] - <date>`, compare links, `Cargo.toml` version, commit
  `chore: release 1.5.0`, tag `v1.5.0`.
- Packages come from `scripts/package.sh` (ADR-T27-2). `draw.debug` is
  kept beside them for every published build id (ADR-T27-3).
- The manual redraw budgets of [[ADR-T24-5 Redraw budgets]] are checked
  on the target before a release.

## Plan

1. Every dependency is `done` and merged into `main`. `scripts/check.sh
   --fuzz` is green.
2. Manual checks on the target (i5-8250U, UHD 620, Hyprland): redraw
   budgets with `DRAW_FRAME_TIMES=1`; one long session with the T47 log
   showing no anomaly; `Ctrl-C` exits cleanly; a non-default theme
   build; writing in rect/ellipse/grid cells, and a numbered shape
   refusing text.
3. `CHANGELOG.md`: `Deprecated` lists X11/Xwayland (removed in 2.0.0).
   `Changed` lists text in shapes, no longer experimental.
4. Version bump, tag, packages, Flatpak, `SHA256SUMS`, `draw.debug`.

## Out of scope

- Any code change. A bug found here goes back to its task, or gets a
  new fix task, before the tag.

## Files owned

- `Cargo.toml` (version), `Cargo.lock`, `CHANGELOG.md`
- packaging metadata that carries the version (metainfo release entry)

## Subtasks

- [ ] checks — manual checks recorded in *Log*
- [ ] release — `chore: release 1.5.0`, tag `v1.5.0`
- [ ] packages — built and checksummed

## Log

- 2026-11-02 — planned at the owner's request: 1.5.0 is the last
  feature release; 2.0.0 only removes X11, refactors and fixes bugs.
