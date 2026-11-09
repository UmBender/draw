---
id: T82
title: Raise the binary size limit
status: done
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

- **AC-1** — `scripts/package.sh` sets `max_binary_bytes=$((20 * 1024 * 1024))`. *Test:* `package_script_binary_limit_is_20_mib` in `tests/packaging.rs`, which reads the script.
- **AC-2** — `ADR-T82-1` records the new limit, amends ADR-T29-1, and says what still bounds size: start-up time, and the separate per-font guards, which stay. *Test:* `binary_limit_adr_amends_t29_1_and_names_what_bounds_size` in `tests/packaging.rs`, which reads the ADR.

## Out of scope

- The per-font size guards ([[ADR-T55-1 Math font subset with the alphanumeric block and extra symbols]]).
- Shrinking or growing the binary.

## Files owned

- `scripts/package.sh`
- `tests/packaging.rs`
- `docs/features/Linux packages.md`
- this note, and new `ADR-T82-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T82): specify raise the binary size limit`
- [x] tests — `test(T82): add failing tests`
- [x] models — skipped, see *Log*
- [x] behaviour — `feat(T82): implement the 20 MiB binary limit`
- [x] quality — skipped, see *Log*
- [x] docs — `docs(T82): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-09 — note written from the owner's verdicts on [[Feature proposal]].
- 2026-11-09 — models step skipped: the change is one shell constant, with no types.
- 2026-11-09 — quality step skipped: `scripts/check.sh` was green with nothing to fix.
- 2026-11-09 — AC-1 test confirmed red against the 2 MiB line before the change. AC-2's test passed from its first run, because the ADR is written in the spec step.
- 2026-11-09 — `scripts/package.sh` itself was not run (it needs `podman` and a full packaging build); the limit is checked by reading the script. The binary size did not change.
- 2026-11-09 — UNATTENDED CHOICE: added a test for AC-2 that reads the ADR, because the Definition of Done asks for a test per criterion and the note named none.
- 2026-11-09 — integrated.
