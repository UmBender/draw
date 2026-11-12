---
id: T131
title: No dynamic dispatch and stricter lints
status: review
wave: 65
branch: task/T131-no-dynamic-dispatch-and-stricter-lints
depends_on: [T127]
adrs: [ADR-T131-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T131 No dynamic dispatch and stricter lints

## Goal

The source has no `dyn` outside an allow-list that is itself an ADR, matches on enums have no wildcard arm, and `unsafe` blocks are documented.

## Known facts (2026-11-12)

- Owner, 2026-11-12: minimise or remove dynamic dispatch; catch bugs at compile time. The audit of T127 counts today's `dyn`.

## Spec

Decision: [[ADR-T131-1 No dyn outside an allow-list]] (file-level allow-list with reason and removing task).

- **AC-1** — `tests/no_dyn.rs` fails on the word `dyn` in code under `src/` and `vendor/` outside the allow-list of ADR-T131-1, with the same structure as the colour guard of `src/theme.rs`.
  Tests: `scanner_finds_dyn`, `scanner_ignores_comments_and_strings`, `guard_finds_no_dyn_outside_allow_list`, `allow_list_entries_must_exist` (the file exists and still has `dyn`).
- **AC-2** — `Cargo.toml` `[lints]` denies `clippy::wildcard_enum_match_arm` and `clippy::undocumented_unsafe_blocks` and sets `unsafe_op_in_unsafe_fn`; every existing wildcard match is replaced by the listed variants.
  `scripts/check.sh` is the test.
- **AC-3** — Existing `dyn` uses are removed or listed with a reason and a task that removes them.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `tests/no_dyn.rs`, `Cargo.toml`, `src/**` (only to remove wildcard arms and `dyn`)
- `tests/e2e.rs`, `tests/fuzz.rs` (one wildcard arm each, listed as variants)
- `docs/decisions/ADR-T131-1*`, `docs/process/Quality Gates.md`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T131): specify the dyn guard and the lints`
- [x] tests — `test(T131): add failing tests for the dyn guard`
- [x] behaviour — `refactor(T131): remove wildcard arms and dyn`
- [x] quality — `chore(T131): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-12 — UNATTENDED CHOICE: key tables over `Key` and `KeyCode` keep a scoped allow, and unit-test code is exempt via `cfg_attr(test)` in `src/lib.rs`; both recorded in [[ADR-T131-1 No dyn outside an allow-list]]. Every other wildcard arm in `src/` lists its variants; `dyn Fn` and `dyn Outlines` became generics.
- 2026-11-12 — BLOCKED (batch): `cargo clippy --all-targets` still fails on one wildcard arm each in `tests/e2e.rs:474` (`Step`) and `tests/fuzz.rs:674` (`InputEvent`); both files are outside *Files owned*. Recommended fix: list the variants there (or add `#[allow(clippy::wildcard_enum_match_arm)]`), then run `scripts/check.sh`, tick the quality subtask and set `status: review`.
- 2026-11-12 — UNBLOCKED by the owner: `tests/e2e.rs` and `tests/fuzz.rs` added to *Files owned*; their wildcard arms now list the `Step` and `InputEvent` variants. `scripts/check.sh --fuzz` green.
