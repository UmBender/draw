---
id: T138
title: Model-based tests of the automaton
status: todo
wave: 68
branch: task/T138-model-based-tests-of-the-automaton
depends_on: [T135, T136, T137]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T138 Model-based tests of the automaton

## Goal

The machine is checked against a small reference model and by exploring reachable states, not only by examples.

## Known facts (2026-11-12)

- [[2.0 Architecture proposal]] §3: a reference model of a few dozen lines and `proptest` over event sequences; reachability up to depth n.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — A reference model of the interaction (mode, tool, page, selection count, can-undo) and `machine_matches_the_model` over generated sequences.
  proptest, 20000 cases in the nightly fuzz.
- **AC-2** — Exploration to a stated depth shows every reachable state can return to `Idle` (`no_state_traps_the_user`).
- **AC-3** — Every bug found here gets a `regression_<ID>_<slug>` test.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/tests/model.rs`, `tests/fuzz.rs`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T138): specify the model-based tests`
- [ ] tests — `test(T138): add the model and the exploration`
- [ ] fixes — one `fix(T138)` commit per bug found, each with its regression test

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
