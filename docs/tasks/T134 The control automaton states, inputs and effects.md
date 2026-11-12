---
id: T134
title: The control automaton states, inputs and effects
status: done
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

Design in [[ADR-T134-1 The automaton wraps the editor behind a routing table]].

- **AC-1** — `Input` (pointer, key, text, scroll, paste, resize, `Tick(Instant)`, `Reply(..)`), `Effect` (`Redraw`, `Clipboard`, `Save`, `ExportPng`, `ExportSvg`, `CopyImage`, `Notify`, `Schedule`) and the state enums are defined; `step` is total and pure.
  Tests: `step_is_deterministic` (proptest: the same input sequence gives the same states and effects); `step_requests_become_effects` (Ctrl+S gives `Save`); `step_tick_is_ignored`.
- **AC-2** — Every `match` over `(State, Input)` has no wildcard arm; an added variant fails to compile until decided, and 'ignored' is written down.
  Guard: the lint of [[T131 No dynamic dispatch and stricter lints]]; test `table_has_no_wildcard_arm` scans the table source; `interaction_of_follows_the_editor` checks the state classification.
- **AC-3** — The transition table can be listed; `docs/architecture/Control automaton.md` embeds a diagram generated from it, and a test fails when they differ.
  Tests: `table_lists_every_cell`, `diagram_matches_the_table`.
- **AC-4** — An adapter makes the old `Editor` and the machine produce the same snapshot after any input sequence the fuzz generators can build.
  Test: `machine_and_editor_agree` (proptest, over the event kinds the fuzz generators use).

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-core/src/machine/**`, `docs/architecture/Control automaton.md`
- this note and `ADR-T134-*`

## Subtasks (one commit each)

- [x] spec — `docs(T134): specify the control automaton`
- [x] tests — `test(T134): add failing tests for the machine`
- [x] models — merged into the behaviour commit (see Log)
- [x] behaviour — `feat(T134): implement step with the adapter`
- [x] quality — `chore(T134): pass clippy and rustfmt`
- [x] docs — `docs(T134): add the architecture note`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — UNATTENDED CHOICE: `pub mod machine;` added to `crates/draw-core/src/lib.rs` (not in *Files owned*, unavoidable); see ADR-T134-1.
- 2026-11-12 — Deviation: the models and behaviour steps are one commit (`feat(T134): implement step with the adapter`); the types and the table are small. The architecture note `docs/architecture/Control automaton.md` was written in that commit because `diagram_matches_the_table` reads it.
- 2026-11-12 — Scope: the table ignores only `Tick` and `Reply` for now; every other cell is forwarded to the editor, so `machine_and_editor_agree` holds by construction. `Clipboard` and `Schedule` effects are defined but not produced yet (T137). The proptest uses its own event generator in `machine/tests.rs` (the fuzz generators live in `tests/fuzz.rs`, outside the files owned).
- 2026-11-12 — `scripts/check.sh` green. Status set to `review`.
