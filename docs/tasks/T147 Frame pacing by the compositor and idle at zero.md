---
id: T147
title: Frame pacing by the compositor and idle at zero
status: todo
wave: 71
branch: task/T147-frame-pacing-by-the-compositor-and-idle-
depends_on: [T143, T144, T146]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T147 Frame pacing by the compositor and idle at zero

## Goal

Frames are paced by `wl_surface.frame` or X11 presentation timing instead of a polling loop, and an idle window uses no CPU.

## Known facts (2026-11-12)

- [[ADR-T12-1 Blocking event loop with cached frame]] keeps idle at 0 %; this keeps it while adding paced redraws.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — A redraw is requested once per compositor frame at most; with no input and no timer the process sleeps.
  Tests: recorder-based `no_redraw_without_input`; the idle CPU check of the e2e suite.
- **AC-2** — A paused or hidden window draws nothing.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw/src/app.rs`, `crates/draw-sys/**` (timing only)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T147): specify frame pacing`
- [ ] tests — `test(T147): add failing tests for pacing`
- [ ] behaviour — `feat(T147): pace frames by the compositor`
- [ ] quality — `chore(T147): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
