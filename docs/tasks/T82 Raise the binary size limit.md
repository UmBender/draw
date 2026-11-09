---
id: T82
title: Raise the binary size limit
status: todo
wave: 51
branch: chore/T82-binary-limit
depends_on: []
adrs: [ADR-T82-1]
feature: "[[Linux packages]]"
tutorial: ""
tags: [task]
---

# T82 Raise the binary size limit

## Goal

The shipped binary may be up to 20 MiB (20 971 520 bytes) instead of 2 MiB. The owner raised it on 2026-11-09 so that features are no longer held back by size. (owner's request, recorded in [[Feature proposal]])

## Known facts (2026-11-09)

- The check is `max_binary_bytes` in `scripts/package.sh`, enforced by `check_debug_symbols`.
- The old limit is stated in [[ADR-T29-1 Text shape with monospaced layout and an embedded font]], so the new one is an ADR amending it.
- The binary was 1 966 008 bytes on 2026-11-09.
- The owner said 20 MB; the script counts in MiB, as it did for 2 MiB.

## Spec

The spec step names the test for each criterion.

- **AC-1** — `scripts/package.sh` sets `max_binary_bytes=$((20 * 1024 * 1024))`. *Test:* a new test in `tests/packaging.rs` that reads the script.
- **AC-2** — `ADR-T82-1` records the new limit, amends ADR-T29-1, and says what still bounds size: start-up time, and the separate per-font guards, which stay.

## Out of scope

- The per-font size guards ([[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]).
- Shrinking or growing the binary.

## Files owned

- `scripts/package.sh`
- `tests/packaging.rs`
- `docs/features/Linux packages.md`
- this note, and new `ADR-T82-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T82): specify raise the binary size limit`
- [ ] tests — `test(T82): add failing tests`
- [ ] models — `feat(T82): add types`
- [ ] behaviour — `feat(T82): implement`
- [ ] quality — `chore(T82): pass clippy and rustfmt`
- [ ] docs — `docs(T82): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
