---
id: T152
title: Command line and DRAW_* variables audit
status: todo
wave: 72
branch: task/T152-command-line-and-draw-variables-audit
depends_on: [T128, T127]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T152 Command line and DRAW_* variables audit

## Goal

Only basic flags remain, and the `DRAW_*` variables are for debugging.

## Known facts (2026-11-12)

- Owner, 2026-11-02, with one change of 2026-11-12: `DRAW_BACKEND` stays because X11 stays.
- Debugging, kept: `DRAW_LOG`, `DRAW_FRAME_TIMES`, `DRAW_SKIP_PREFLIGHT`, `DRAW_BACKEND`. `DRAW_EXPERIMENTAL` goes with what it switched. `DRAW_MSAA` is decided with the renderer ([[T146 Retained renderer in draw-gfx]]); `DRAW_TABLET` is decided here.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `--version` and `--help` are the only flags besides the `draw file.draw` argument and `--recover`; `--help` lists the debugging variables in a 'Debugging' section.
  Tests: `help_lists_debug_variables`, `unknown_flag_is_refused`.
- **AC-2** — The user documentation mentions the variables only in a Debugging section.
  Test: a docs guard.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw/src/main.rs`, `README.md`, `docs/features/**` (mentions only)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T152): specify the command line`
- [ ] tests — `test(T152): add failing tests for the flags`
- [ ] behaviour — `feat(T152): trim the command line`
- [ ] docs — `docs(T152): move the variables to a Debugging section`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
