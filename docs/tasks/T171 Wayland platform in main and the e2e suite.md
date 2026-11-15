---
id: T171
title: Wayland platform in main and the e2e suite
status: todo
wave: 73
branch: task/T171-wayland-platform-in-main-and-the-e2e-su
depends_on: [T170]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T171 Wayland platform in main and the e2e suite

## Goal

Slice 4 of [[T143 Wayland backend with window, keyboard, pointer, tablet and clipboard]]: `main` gets a second arm in its `Runtime` match that runs `app::run` over the native `WaylandPlatform` and the OpenGL ES `Gpu` of `draw-gfx`, and the e2e suite passes with it.

## Known facts (2026-11-15)

- Split out of T143 by the owner. Builds on [[T170 Wayland tablet and clipboard]] and [[T167 Generic run and the match in main]].
- **Owner decisions (2026-11-15):** the flag is `--platform macroquad|native` (default `macroquad` until [[T149 Cutover removing miniquad and macroquad]]; `--backend` keeps its meaning); `native` picks Wayland or X11 from the session ([[T178 X11 platform in main and the e2e suite]] does the X11 arm). The e2e goldens hold with the GL `Gpu` of [[ADR-T145-1 GL context, function table and Gpu over OpenGL ES 3.0]] within the tolerance `scripts/e2e.sh` already uses; a larger difference blocks the task with the difference images.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `main` runs `app::run::<WaylandPlatform, GlGpu>` for `--platform native` on a Wayland session.
- **AC-2** — `scripts/e2e.sh` passes with this backend selected (T143 AC-3).
- **AC-3** — Hand check (owner): window, keyboard, pointer, tablet and clipboard in the real app on the desktop compositor.

## Out of scope

- Removing miniquad ([[T149 Cutover removing miniquad and macroquad]]); frame pacing ([[T147 Frame pacing by the compositor and idle at zero]]).

## Files owned

- `src/main.rs`, `src/shell/app.rs` (selection only), `src/shell/options.rs` or wherever the flags live, `scripts/e2e.sh`, `tests/e2e*`, `tests/options.rs`
- the ADRs this task writes
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T171): specify the slice`
- [ ] tests — `test(T171): add failing tests`
- [ ] behaviour — `feat(T171): run the app over the Wayland platform`
- [ ] quality — `chore(T171): pass clippy and rustfmt`
- [ ] by hand — owner, recorded in the Log

## Log

- 2026-11-15 — split from T143 by the owner.
