---
id: T139
title: Split tex.rs into modules
status: in-progress
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
  Tests: every existing `core::tex::tests::*` test keeps its name and
  passes (moved under `tests/`), plus the unchanged goldens in `tests/`.
- **AC-2** — No module exceeds 3000 lines and each has its own tests.
  Test: `modules_stay_under_the_limit` (source guard, reads every file of
  `core/tex/` and checks the line count).
- Layout: `mod.rs` re-exports every item of the modules (`pub use`), so
  paths like `tex::Node` and `tex::layout` stay. Modules are private;
  items shared between siblings are `pub(super)`. Tests live in
  `tex/tests/{parse,layout,alphabets,symbols,macros}.rs`.
- No ADR: a mechanical move, no architecture choice.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/core/tex/**` (the file is `core/tex.rs` today; owner's
  triage 2026-11-12)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T139): specify the tex split`
- [ ] behaviour — `refactor(T139): split tex into modules`
- [ ] quality — `chore(T139): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — BLOCKED (batch): the split moves ~11k lines of `crates/draw-core/src/core/tex.rs` into `core/tex/{parse,layout,metrics,glyphs,macros}.rs` plus split test files. This needs `mkdir`, `git mv` and `sed -n 'a,bp' file > out` (line-range extraction); all were refused (`mkdir`, `git mv`), and retyping 11k lines through Write is not safe. Also: *Files owned* says `crates/draw-core/src/tex/**` but the file lives at `crates/draw-core/src/core/tex.rs`; the owner should confirm `core/tex/**`. Recommended fix: allow `mkdir`, `git mv` and `sed -n` with redirection to a file under `crates/draw-core/src/core/tex/` (or run the split by hand), then re-run the batch for T139.
