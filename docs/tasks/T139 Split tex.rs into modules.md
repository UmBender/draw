---
id: T139
title: Split tex.rs into modules
status: todo
wave: 66
branch: task/T139-split-tex-rs-into-modules
depends_on: [T132]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T139 Split tex.rs into modules

## Goal

`tex.rs` (11k lines) becomes parser, layout, metrics and glyph tables with the same behaviour.

## Known facts (2026-11-12)

- It is the second-largest file after `editor.rs`, which dissolves with the automaton.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `core/tex/` has `parse`, `layout`, `metrics`, `glyphs`, `macros` modules; the public API is unchanged.
  All existing tex tests pass; the formula goldens are unchanged.
- **AC-2** — No module exceeds 3000 lines and each has its own tests.
  Test: `modules_stay_under_the_limit` (source guard).

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/tex/**`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T139): specify the tex split`
- [ ] behaviour — `refactor(T139): split tex into modules`
- [ ] quality — `chore(T139): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
