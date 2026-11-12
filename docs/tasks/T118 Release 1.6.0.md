---
id: T118
title: Release 1.6.0
status: todo
wave: 62
branch: chore/release-1.6.0
depends_on: [T53, T108, T112, T113, T114, T115, T116, T117, T119, T120, T121]
adrs: []
feature: ""
tutorial: ""
tags: [task, release]
---

# T118 Release 1.6.0

## Goal

draw 1.6.0 is tagged and packaged: the features that were not ready for [[T53 Release 1.5.0]] (save and load, pages, user macros, the shortcut sheet, SVG export, copy as image) and looser shape recognition ([[T117 Looser shape recognition]]).

## Known facts (2026-11-11)

- 1.5.0 was tagged and pushed on 2026-11-11; the current version is 1.5.0. The steps are in [[Commit Convention#Versions and changelog]]: `[Unreleased]` becomes `## [1.6.0] - <date>`, compare links, `Cargo.toml` version, commit `chore: release 1.6.0`, tag `v1.6.0`. Every feature adds `feat` commits, so the bump is minor.
- Packages come from `scripts/package.sh` (ADR-T27-2); the owner installs the Flatpak bundle with `flatpak install --user --reinstall ./dist/draw-1.6.0-x86_64.flatpak`.
- `package.sh` once failed on a false `liblib` library name (fixed in 1.5.0); if the library check fails again, read the name before adding it to `packaging/runtime-libs`.
- The owner's manual checks for each feature are in that feature's task *Log*.

## Plan

1. Every dependency is `done` and merged into `main`. `scripts/check.sh --fuzz` is green.
2. Manual checks on the target (i5-8250U, UHD 620): redraw budgets with `DRAW_FRAME_TIMES=1`; save, close and load a multi-page document; one macro recorded and replayed; the shortcut sheet matches the keymap; SVG opens in a browser; copied image pastes elsewhere; `Z` accepts a hurried circle, box and line.
3. `CHANGELOG.md` entries for T108, T112–T117 are already under `[Unreleased]`; check none is missing.
4. Version bump, tag, packages, Flatpak, `SHA256SUMS`, `draw.debug`. Push `main` and the tag only when the owner says so.

## Out of scope

- Any code change. A bug found here goes back to its task, or gets a new fix task, before the tag.

## Files owned

- `Cargo.toml` (version), `Cargo.lock`, `CHANGELOG.md`, `README.md` (install tag)
- packaging metadata that carries the version (metainfo release entry)

## Subtasks

- [ ] checks — manual checks recorded in *Log*
- [ ] release — `chore: release 1.6.0`, tag `v1.6.0`
- [ ] packages — built and checksummed

## Log

- 2026-11-11 — planned at the owner's request: 1.5.0 shipped what was done; the rest of the batch (T108, T112–T116) and T117 go in 1.6.0.
