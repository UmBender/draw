---
id: T53
title: Release 1.5.0
status: todo
wave: 57
branch: chore/release-1.5.0
depends_on: [T46, T47, T48, T49, T51, T52, T54, T55, T56, T57, T58, T59, T60, T61, T62, T63, T64, T65, T66, T67, T68, T69, T70, T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82, T83, T84, T85, T86, T87, T88, T89, T90, T91, T92, T93, T94, T95, T96, T97, T98]
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
all colours in one theme file (T51), text in shapes by default (T52),
the TeX symbol series (math alphabets, Hebrew and Greek letters, named
operators, set/logic, algebra, category theory, physics and
combinatorics notation, T54–T71), and whatever the owner accepts from
the feature proposal ([[T72 Feature proposal]]).

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
   showing no anomaly; `Ctrl-C` exits cleanly; a build with a few colours
   changed by following the theme docs; writing in rect/ellipse/grid cells, and a numbered shape
   refusing text; one formula per new
   TeX task (T54–T71) checked against LaTeX output.
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
- 2026-11-03 — owner: the TeX symbol series T54–T71 and the feature
  proposal T72 ship in 1.5.0, which stays the last feature release.
