---
id: T139
title: Split tex.rs into modules
status: review
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
  `tex/tests/{basics,alphabets,structures,macros,split}.rs` (as built; the
  shared helpers are in `tests/mod.rs`).
- No ADR: a mechanical move, no architecture choice.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/core/tex/**` (the file is `core/tex.rs` today; owner's
  triage 2026-11-12)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T139): specify the tex split`
- [x] tests — `test(T139): add failing tests for the tex split`
- [x] behaviour — `refactor(T139): split tex into modules`
- [x] quality — folded into the refactor commit (clippy, rustfmt and docs were fixed before it)

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — BLOCKED (batch): the split moves ~11k lines of `crates/draw-core/src/core/tex.rs` into `core/tex/{parse,layout,metrics,glyphs,macros}.rs` plus split test files. This needs `mkdir`, `git mv` and `sed -n 'a,bp' file > out` (line-range extraction); all were refused (`mkdir`, `git mv`), and retyping 11k lines through Write is not safe. Also: *Files owned* says `crates/draw-core/src/tex/**` but the file lives at `crates/draw-core/src/core/tex.rs`; the owner should confirm `core/tex/**`. Recommended fix: allow `mkdir`, `git mv` and `sed -n` with redirection to a file under `crates/draw-core/src/core/tex/` (or run the split by hand), then re-run the batch for T139.
- 2026-11-12 — BLOCKED (batch, 2nd run): spec and failing-test commits are done (`modules_stay_under_the_limit` fails as intended). `mkdir` and `git mv` now work, but `sed -n 'a,bp' file > crates/draw-core/src/core/tex/x.rs` (single and multi-range) and the `python3 -c` slicing fallback were both refused, so the line ranges cannot be extracted. Planned ranges of `tex.rs` (1-indexed): metrics 18–196, 524–534, 1818–1835; parse 198–318, 402–455, 2291–3202; glyphs 319–401, 456–523, 536–983, 1176–1545; macros 984–1175, 1837–2289; layout 1547–1816, 3204–4272; tests 4274–end split into `tex/tests/*`. Each new file starts with `use super::*;`, and `mod.rs` does `pub use` of each module. Recommended fix: allow `sed -n * > crates/draw-core/src/core/tex/*` and `python3 -c *` (or `Bash(sed:*)`), then re-run.
- 2026-11-12 — done (batch, 3rd run) with `.batch/tools/slice.py`: the file was moved with `git mv` to `tex/mod.rs`, cut by line ranges (metrics 212 lines, parse about 1090, glyphs about 970, macros about 650, layout about 1340; tests in five files, the largest 2400). Items shared between siblings are `pub(super)`; imports are explicit (`clippy::wildcard_imports`); doc links to items in sibling modules use `super::` paths; the guard test uses `panic!` instead of `expect` (`clippy::expect_used`). Test bodies are verbatim, 546 tex-crate tests pass, goldens untouched, `scripts/check.sh` green. No separate quality commit: nothing was left to fix after the refactor. Modules are `metrics`, `parse`, `glyphs`, `macros`, `layout`; `expand` (the table expansion) lives in `macros`.
