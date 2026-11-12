---
id: T153
title: Text and select as sub-automata
status: todo
wave: 69
branch: task/T153-text-and-select-as-sub-automata
depends_on: [T136, T138]
adrs: [ADR-T153-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T153 Text and select as sub-automata

## Goal

The text tool and the select tool keep their sub-states in typed enums in `machine/`, so a state that the code does not mean cannot be built and a half-finished edit cannot meet a command that does not expect it.

## Known facts (2026-11-12)

- `core/tools/text.rs` keeps `target: Option<ShapeId>`, `host: Option<Host>`, `resized: bool` and `edit: Option<TextEdit>`; the combinations the code reaches are not written down.
- `core/tools/select.rs` has a private `enum Gesture` (marquee, move, resize, with `additive`, `duplicate`, `proportional`).
- Pen, line, arrow, rectangle, ellipse, grid, bucket and eraser are press, drag, release; `Stage` covers them ([[ADR-T136-1 One gesture variant per tool in the machine]]).

## Spec

See [[ADR-T153-1 Text and select are sub-automata of the machine]]. Step 1 of the task is to map which combinations of the text fields the code reaches, with a test or a trace for each, and to write the states in this note before any code.

- **AC-1** — The text states are an enum with one variant per reachable state; the old `Option` and flag combinations that are not a variant are gone. *Tests:* one per variant and one per transition, plus the existing text tests unchanged.
- **AC-2** — The select states are an enum in `machine/`; its transitions are a `match` without a wildcard. *Tests:* the existing select tests unchanged and one per transition.
- **AC-3** — Snapshots written before this task still load, and the ones written after load back to the same state. *Tests:* `snapshot_roundtrip` for every text and select state, and a fixture of an old snapshot.
- **AC-4** — The T138 model checks text and select against a reference model for any input sequence. *Tests:* `machine_matches_the_model` extended, and `no_state_traps_the_user` for the new states.
- **AC-5** — No `dyn` and no wildcard arm in the new code, and the pointer path does not allocate more than before. *Tests:* `tests/no_dyn.rs` and the perf tests of `tests/perf.rs`, unchanged.

## Out of scope

- The other tools; new user-facing features.
- Anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `crates/draw-core/src/machine/**`, `crates/draw-core/src/core/tools/text.rs`, `crates/draw-core/src/core/tools/select.rs`, `crates/draw-core/src/core/editor.rs`, `crates/draw-core/src/core/snapshot.rs`
- `tests/model.rs`, `tests/fuzz.rs`
- this note, `ADR-T153-1`

## Subtasks (one commit each)

- [ ] spec — `docs(T153): map the text and select states`
- [ ] tests — `test(T153): add failing tests for the states`
- [ ] behaviour — `refactor(T153): move text and select into the machine`
- [ ] quality — `chore(T153): pass clippy and rustfmt`
- [ ] docs — `docs(T153): update the architecture note`

## Log

- 2026-11-12 — created after the owner reviewed ADR-T136-1 and chose to rewrite the sub-states of text and select, not all tools.
