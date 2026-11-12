---
id: T130
title: Coverage in CI with a ratchet
status: todo
wave: 65
branch: task/T130-coverage-in-ci-with-a-ratchet
depends_on: [T127]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T130 Coverage in CI with a ratchet

## Goal

Test coverage is measured on every push and cannot fall.

## Known facts (2026-11-12)

- No coverage is measured today; the audit of [[T127 2.0 audit and baseline]] runs it by hand once.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `scripts/coverage.sh` runs `cargo llvm-cov` with the same resource caps as `check.sh` and writes a per-module summary.
  Test: the script's parsing of the summary (`coverage_summary_is_parsed`).
- **AC-2** — A file `coverage-baseline.toml` holds the line coverage of each module; CI fails when a module falls below its recorded value by more than 0.5 points, and a change that raises it must update the file.
  Test: `ratchet_fails_when_a_module_falls`, `ratchet_asks_to_raise_the_baseline`.
- **AC-3** — `docs/process/Quality Gates.md` lists the gate; the Definition of Done names it.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `scripts/coverage.sh`, `coverage-baseline.toml`, `.github/workflows/ci.yml`, `tests/ci.rs`
- `docs/process/Quality Gates.md`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T130): specify coverage with a ratchet`
- [ ] tests — `test(T130): add failing tests for the ratchet`
- [ ] behaviour — `feat(T130): measure coverage in CI`
- [ ] docs — `docs(T130): update the quality gates`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
