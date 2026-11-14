---
id: T149
title: Cutover removing miniquad and macroquad
status: todo
wave: 73
branch: task/T149-cutover-remove-miniquad-and-macroquad
depends_on: [T143, T144, T146, T147, T148, T152, T153, T154, T155, T156, T157, T158, T159, T160, T161, T162, T163, T164, T165, T166, T167]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T149 Cutover removing miniquad and macroquad

## Goal

The vendored miniquad and macroquad are deleted; the app runs only on the new backend.

## Known facts (2026-11-12)

- A strangler: the new platform replaces the old only when it passes the same e2e suite on both backends.
- The source scan of [[T167 Generic run and the match in main]] allows a few macroquad uses outside its interpreters (`Color`, the menu font measure, the window config, the signals, `LinuxBackend`); this task empties that allowlist.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `vendor/miniquad` and the `macroquad` dependency are gone from `Cargo.toml`, `Cargo.lock`, packaging and docs.
  `tests/deps.rs` matches the dependency ADR.
- **AC-2** — `scripts/release.sh --dry-run` is green: `check.sh --fuzz`, e2e on X11 and Wayland, packages and the runtime-libs scan.
- **AC-3** — The frame times and the idle CPU are re-measured on the target and recorded.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `Cargo.toml`, `Cargo.lock`, `vendor/`, `packaging/`, `docs/**` (mentions)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T149): specify the cutover`
- [ ] behaviour — `refactor(T149): remove miniquad and macroquad`
- [ ] docs — `docs(T149): update the notes and the measurements`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
