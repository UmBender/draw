---
id: T151
title: Release 2.0.0
status: todo
wave: 74
branch: task/T151-release-2-0-0
depends_on: [T127, T128, T129, T130, T131, T132, T133, T134, T135, T136, T137, T138, T139, T140, T141, T142, T143, T144, T145, T146, T147, T148, T149, T152, T153, T154, T155, T156, T157, T158, T159, T160, T161, T162, T163, T164]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T151 Release 2.0.0

## Goal

draw 2.0.0 is tagged and packaged from the new architecture, with the release gate green.

## Known facts (2026-11-12)

- The steps follow [[Commit Convention#Versions and changelog]] and the release gate of [[T129 Release gate with end-to-end on X11 and Wayland]]. A major bump: the `CHANGELOG` `### Changed` explains the new backend, and nothing user-facing is removed.
- [[T150 Optional Vulkan backend for the Gpu trait]] does not block it.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `scripts/release.sh 2.0.0` passes and produces the packages, the Flatpak (with the `x11` and `wayland` sockets) and `SHA256SUMS`.
- **AC-2** — The owner's manual checks on both backends (pen pressure, clipboard, resize, scale, key repeat) are recorded in the Log.
- **AC-3** — `CHANGELOG.md`, `README.md`, `Cargo.toml` and the metainfo carry 2.0.0.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `README.md`, `packaging/`
- this note

## Subtasks (one commit each)

- [ ] checks — manual checks recorded
- [ ] release — `chore: release 2.0.0`, tag `v2.0.0`
- [ ] packages — built and checksummed

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
