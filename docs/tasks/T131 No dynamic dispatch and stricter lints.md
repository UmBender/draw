---
id: T131
title: No dynamic dispatch and stricter lints
status: todo
wave: 65
branch: task/T131-no-dynamic-dispatch-and-stricter-lints
depends_on: [T127]
adrs: []
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

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `tests/no_dyn.rs` fails on `dyn ` or `Box<dyn` in `src/` and `vendor/` outside `docs/decisions/ADR-T131-1`'s allow-list, with the same structure as the colour guard of `src/theme.rs`.
  Tests: `guard_finds_dyn`, `guard_ignores_comments_and_strings`, `allow_list_entries_must_exist`.
- **AC-2** — `Cargo.toml` `[lints]` denies `clippy::wildcard_enum_match_arm` and `clippy::undocumented_unsafe_blocks` and sets `unsafe_op_in_unsafe_fn`; every existing wildcard match is replaced by the listed variants.
  `scripts/check.sh` is the test.
- **AC-3** — Existing `dyn` uses are removed or listed with a reason and a task that removes them.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `tests/no_dyn.rs`, `Cargo.toml`, `src/**` (only to remove wildcard arms and `dyn`)
- `docs/decisions/ADR-T131-1*`, `docs/process/Quality Gates.md`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T131): specify the dyn guard and the lints`
- [ ] tests — `test(T131): add failing tests for the dyn guard`
- [ ] behaviour — `refactor(T131): remove wildcard arms and dyn`
- [ ] quality — `chore(T131): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
