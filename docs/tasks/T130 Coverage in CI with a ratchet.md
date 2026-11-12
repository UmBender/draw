---
id: T130
title: Coverage in CI with a ratchet
status: review
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

A module is a top-level directory of `src/` (`core`, `shell`); its figure is line coverage in percent with one decimal. Tolerance is 0.5 points (owner decision of 2026-11-12). `scripts/coverage.sh` takes `--summary FILE` (an existing `cargo llvm-cov --json` summary, skips the run) and `--baseline FILE`, so the tests drive it with fixtures; `--update` rewrites the baseline.

- **AC-1** — `scripts/coverage.sh` runs `cargo llvm-cov` with the same resource caps as `check.sh` and writes a per-module summary.
  Tests: `coverage_summary_is_parsed` (fixture JSON gives `core 90.0`), `coverage_script_runs_under_the_caps` (cargo stand-in sees `CARGO_BUILD_JOBS`, `RUST_TEST_THREADS`).
- **AC-2** — `coverage-baseline.toml` holds the line coverage of each module; the script fails when a module falls below its value by more than 0.5 points, and when it rises by more than 0.5 (the change must update the file).
  Tests: `ratchet_fails_when_a_module_falls`, `ratchet_asks_to_raise_the_baseline`, `ratchet_passes_within_the_tolerance`, `ratchet_fails_for_a_module_without_a_baseline`. CI runs it: `ci_coverage_job_runs_the_ratchet`.
- **AC-3** — `docs/process/Quality Gates.md` lists the gate; the Definition of Done names it.
  Test: `quality_gates_name_the_coverage_ratchet`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `scripts/coverage.sh`, `coverage-baseline.toml`, `.github/workflows/ci.yml`, `tests/ci.rs`
- `docs/process/Quality Gates.md`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T130): specify coverage with a ratchet`
- [x] tests — `test(T130): add failing tests for the ratchet`
- [x] behaviour — `feat(T130): measure coverage in CI` (the Quality Gates edit went in here, so `quality_gates_name_the_coverage_ratchet` is green in this commit)
- [x] docs — `docs(T130): update the quality gates` (task note only)

## Log

- 2026-11-12 — status review. No models step (no new types) and no quality commit (`scripts/check.sh` green with no change). No feature or tutorial note (task has none).
- 2026-11-12 — `coverage-baseline.toml` holds the T127 figures (core 98.6, shell 80.2), not a fresh run. `scripts/coverage.sh` is mode 644 (chmod refused in the batch): tests and CI call it with `bash`; run `chmod +x scripts/coverage.sh` if wanted. Owner: confirm the `coverage` job runs on GitHub (it installs cargo-llvm-cov, slow) and that a real `bash scripts/coverage.sh` passes against the baseline.
- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
