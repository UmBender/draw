---
id: T152
title: Command line and DRAW_* variables audit
status: in-progress
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

- **AC-1** — `--version` and `--help` are the only flags besides the `draw file.draw` argument and `--recover`; `--help` lists the debugging variables in a 'Debugging' section.
  Tests (in `src/main.rs`): `help_lists_debug_variables`,
  `version_prints_package_version`, `unknown_flag_is_refused`,
  `known_flags_and_files_are_accepted`. `--help`/`--version` act before the
  preflight, so they work without a display; a refused flag exits with 2.
- **AC-2** — The user documentation mentions the variables only in a Debugging section.
  Test: `readme_names_variables_only_in_debugging_section` (reads `README.md`).
- **AC-3** — `DRAW_EXPERIMENTAL` no longer exists in the code (nothing to
  remove: T128 already dropped it). `DRAW_TABLET` stays, see
  [[ADR-T152-1 Keep DRAW_TABLET as a debugging variable]]; `DRAW_MSAA` is
  listed as a debugging variable until T146 decides.
  Test: `help_lists_debug_variables` checks the list.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `src/main.rs` (the note said `crates/draw/src/main.rs`; the root package kept its place in T132), `README.md`, `docs/features/**` (mentions only)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T152): specify the command line`
- [ ] tests — `test(T152): add failing tests for the flags`
- [ ] behaviour — `feat(T152): trim the command line`
- [ ] docs — `docs(T152): move the variables to a Debugging section`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
