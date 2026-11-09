---
id: T86
title: Zero-based numbering
status: in-progress
wave: 51
branch: task/T86-zero-based
depends_on: []
adrs: [ADR-T86-1]
feature: "[[Numbered nodes]]"
tutorial: ""
tags: [task]
---

# T86 Zero-based numbering

## Goal

Node numbers can start at 0, to match 0-indexed code. (idea E2 of [[Feature proposal]])

## Known facts (2026-11-09)

- `core::numbering` has `FIRST_NUMBER` and `roll_back`, which never goes below it.
- `Shift+N` restarts at 1; grid indices are already 0-based.
- `Ctrl+N` is unbound.

## Spec

`Helpers` gets a `zero_based` switch; `Helpers::first_number()` is 0 or
`FIRST_NUMBER`. See [[ADR-T86-1 Zero-based numbering switch]].

- **AC-1** — `Ctrl+N` switches the first number between 1 and 0 and restarts the counter there.
  Tests: `keymap::tests::ctrl_n_switches_the_first_number`,
  `editor::tests::zero_based_switch_flips_and_restarts_the_counter`,
  `editor::tests::zero_based_nodes_count_from_zero`,
  `editor::tests::zero_based_switch_keeps_the_gesture`.
- **AC-2** — `Shift+N` restarts at the current first number.
  Tests: `editor::tests::reset_numbering_restarts_at_the_first_number`.
- **AC-3** — Undo never rolls the counter below the first number.
  Tests: `numbering::tests::roll_back_stops_at_the_given_first`,
  `editor::tests::zero_based_undo_gives_zero_back_and_stops_there`,
  `editor::tests::one_based_undo_ignores_labels_left_at_zero`,
  and the invariant `next_number >= first_number()` in `tests/fuzz.rs`.
- **AC-4** — The setting survives a snapshot round trip.
  Tests: `snapshot::tests::round_trip_keeps_zero_based`,
  `snapshot::tests::parse_checks_the_counter_against_the_first_number`,
  `snapshot::tests::parse_accepts_label_zero`,
  `editor::tests::snapshot_helpers_last_token_is_zero_based`.
- **AC-5** — Number 0 draws and hit-tests like any other label.
  Tests: `shape::tests::label_zero_behaves_like_any_label`,
  `numbering::tests::label_new_gives_zero`,
  `shell::render::tests::number_zero_is_shown_like_any_label`.

## Out of scope

- Letters or any other start value.
- Editing labels.

## Files owned

- `src/core/numbering.rs`, `src/core/snapshot.rs`, `src/core/shape.rs`
- `docs/features/Numbered nodes.md`
- `src/core/editor.rs`, `src/core/command.rs`, `src/core/keymap.rs`, `src/core/input.rs`
- `docs/architecture/Keymap.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/snapshots/**` (a changed snapshot format regenerates fixtures, never drops them)
- `src/shell/render.rs` (one test only, see *Log*)
- this note, and new `ADR-T86-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T86): specify zero-based numbering`
- [ ] tests — `test(T86): add failing tests`
- [ ] models — `feat(T86): add types`
- [ ] behaviour — `feat(T86): implement`
- [ ] quality — `chore(T86): pass clippy and rustfmt`
- [ ] docs — `docs(T86): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — UNATTENDED CHOICE: added `src/shell/render.rs` to *Files owned* for one AC-5 test, because only the shell decides which label is drawn.
- 2026-11-09 — UNATTENDED CHOICE: the setting takes over the reserved seventh `helpers` snapshot token instead of bumping `VERSION`, so every fixture stays byte-identical ([[ADR-T86-1 Zero-based numbering switch]]).
