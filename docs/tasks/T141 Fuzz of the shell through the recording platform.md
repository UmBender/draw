---
id: T141
title: Fuzz of the shell through the recording platform
status: todo
wave: 68
branch: task/T141-fuzz-of-the-shell-through-the-recording-
depends_on: [T140]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T141 Fuzz of the shell through the recording platform

## Goal

Raw input sequences are fuzzed against the input translation and the machine with no window.

## Known facts (2026-11-12)

- Today the fuzz covers the core, not `shell::input_map` or the drawing path.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `tests/shell_fuzz.rs` generates raw events (key codes, repeats, modifiers, pointer, tablet) and runs the shell over the recorder; no panic, invariants of the machine hold, no unbounded growth.
  proptest, 20000 cases in the nightly job.
- **AC-2** — Every bug found has a `regression_<ID>_<slug>` test.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `tests/shell_fuzz.rs`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T141): specify the shell fuzz`
- [ ] tests — `test(T141): add the shell fuzz`
- [ ] fixes — one `fix(T141)` commit per bug found with its regression test

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
