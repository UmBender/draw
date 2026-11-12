---
id: T138
title: Model-based tests of the automaton
status: done
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

### Refined (spec step)

Location (owner's batch decision): the repo-level `tests/model.rs`, next to `tests/fuzz.rs`, because that is where the existing suite lives and runs (`cargo test --test …`, `PROPTEST_CASES` for the nightly count). `crates/draw-core/tests/` does not exist; no `[[test]]` entry is needed.

- **AC-1** — `machine_matches_the_model`: the model keeps `tool`, `page`, per-page shape count and undo depth, `selection` count. Actions: tool key, `Alt+digit` page switch, draw a rectangle, `Ctrl+A`, `Delete`, `Ctrl+Z`, `Escape`. After each step the machine's `tool()`, `page()`, `selection().len()` and `can_undo()` equal the model's. Proptest, `PROPTEST_CASES` cases (default quick).
- **AC-2** — `no_state_traps_the_user`: breadth-first search over a fixed alphabet of actions (tool keys, page keys, drag, `Ctrl+A`, `Ctrl+K`-style overlays if bound, `Escape`, press without release) to depth 3, deduplicated by an abstract key; from every reached machine, at most 3 `Escape`s and a button release return `interaction()` to `Idle`.
- **AC-3** — each bug found: `fix(T138)` commit with `regression_T138_<slug>`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `tests/model.rs` (new; `crates/draw-core/tests/` does not exist), `tests/fuzz.rs` if needed
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T138): specify the model-based tests`
- [x] tests — `test(T138): add the model and the exploration`
- [x] fixes — none needed: no bug found (model and exploration pass)

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — UNATTENDED CHOICE: tests live in the repo-level `tests/model.rs` (where the fuzz suite runs), not `crates/draw-core/tests/`; no `[[test]]` entry needed. The model tracks shape ids per page, selection and undo history; early mismatches were model imprecision (a draw keeps the selection; undo keeps the selected shapes that still exist), not machine bugs. Exploration depth 3 over 20 actions (tools, pages, drag, Ctrl+A, Delete, Undo, Escape, palette, sheet). No bug found, no `fix` commits. The 20000-case run (`PROPTEST_CASES=20000 cargo test --release --test model`) was refused by the batch permissions: the owner should run it once; `scripts/check.sh --fuzz` only runs `--test fuzz` (not owned here to change).
