---
id: T134
title: The control automaton states, inputs and effects
status: todo
wave: 66
branch: task/T134-the-control-automaton-states-inputs-effe
depends_on: [T132]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T134 The control automaton states, inputs and effects

## Goal

The editor's behaviour is written as a pure transition function `step(Machine, Input) -> (Machine, Vec<Effect>)` over closed sum types.

## Known facts (2026-11-12)

- [[2.0 Architecture proposal]] §3. Today behaviour hangs on `Option` fields (`palette`, `sheet`, `text_edit`, `active_gesture`) tested in different orders in `handle`, `apply`, the shell and the tests.
- The old `Editor` stays as an adapter during the migration; the fuzz and snapshot tests must agree with both.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `Input` (pointer, key, text, scroll, paste, resize, `Tick(Instant)`, `Reply(..)`), `Effect` (`Redraw`, `Clipboard`, `Save`, `ExportPng`, `ExportSvg`, `CopyImage`, `Notify`, `Schedule`) and the state enums are defined; `step` is total and pure.
  Tests: `step_is_deterministic` (proptest: the same input sequence gives the same states and effects).
- **AC-2** — Every `match` over `(State, Input)` has no wildcard arm; an added variant fails to compile until decided, and 'ignored' is written down.
  Guard: the lint of [[T131 No dynamic dispatch and stricter lints]].
- **AC-3** — The transition table can be listed; `docs/architecture/Control automaton.md` embeds a diagram generated from it, and a test fails when they differ.
  Test: `diagram_matches_the_table`.
- **AC-4** — An adapter makes the old `Editor` and the machine produce the same snapshot after any input sequence the fuzz generators can build.
  Test: `machine_and_editor_agree` (proptest).

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/machine/**`, `docs/architecture/Control automaton.md`
- this note and `ADR-T134-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T134): specify the control automaton`
- [ ] tests — `test(T134): add failing tests for the machine`
- [ ] models — `feat(T134): add the inputs, effects and states`
- [ ] behaviour — `feat(T134): implement step with the adapter`
- [ ] quality — `chore(T134): pass clippy and rustfmt`
- [ ] docs — `docs(T134): add the architecture note`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
