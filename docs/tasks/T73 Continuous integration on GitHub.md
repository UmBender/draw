---
id: T73
title: Continuous integration on GitHub
status: todo
wave: 33
branch: task/T73-github-ci
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, ci]
---

# T73 Continuous integration on GitHub

## Goal

Every push and pull request to `github.com/UmBender/draw` runs the
quality gates on GitHub Actions, so a broken branch is caught before it
merges and before a release ([[T53 Release 1.5.0]]). The checks match
what `scripts/check.sh` runs on the owner's laptop, so the two never
drift apart. Longer suites (fuzz, soak, packaging) run on a schedule or
by hand, so they don't slow down every push.

## Known facts (2026-11-03)

- There is no `.github/` folder yet. Remote: `git@github.com:UmBender/draw.git`.
- Gates ([[Quality Gates]]): `cargo fmt --check`, `cargo clippy
  --all-targets -D warnings`, `cargo test --all-targets`, `cargo doc`
  with `RUSTDOCFLAGS=-D warnings`; `scripts/check.sh --fuzz` adds
  `PROPTEST_CASES=20000 cargo test --release --test fuzz`.
- Test suites in `tests/`: `skeleton.rs`, `fuzz.rs` (proptest, with a
  committed regressions file), `packaging.rs` (no packaging tools
  needed), `perf.rs` and `soak.rs` (heavy cases are `#[ignore]`, release
  build, one thread; `perf.rs` budgets are for the target laptop).
- `Cargo.toml`: edition 2024, `rust-version = "1.85"`, lint levels in
  `[lints]` (clippy 1.99 lints adopted in [[T43 Adopt clippy 1.99 lints]]).
  No `rust-toolchain.toml`; `packaging/Containerfile` pins Rust 1.97.0.
- `miniquad` is patched from `vendor/miniquad` (ADR-T35-1) and loads
  X11/GL/Wayland with `dlopen`, so building and running the headless
  tests should need no system libraries and no display. Confirm in the
  spec step; if a test needs a display, use `xvfb-run` or mark it.
- `scripts/package.sh` builds `.deb`/`.rpm` inside the
  `packaging/Containerfile` image (glibc 2.28, ADR-T27-1).

## Proposal (settle in the spec step)

1. `ci.yml` — on push to any branch and on pull requests to `main`,
   Ubuntu runner:
   - job **gates**: stable toolchain with `rustfmt` + `clippy`, cargo
     cache, then `scripts/check.sh` (one source of truth for the gates).
   - job **msrv**: `cargo check --all-targets` on Rust 1.85, so
     `rust-version` stays true (or raise `rust-version` with an ADR).
   - Cancel older runs of the same branch (`concurrency`).
2. `nightly.yml` — daily cron plus `workflow_dispatch`:
   - `scripts/check.sh --fuzz` (20 000 cases); on failure, upload the
     proptest regressions file as an artifact so the case can be
     replayed and committed.
   - Soak: `cargo test --release --test soak -- --ignored --nocapture
     --test-threads=1` (the resident-memory check is Linux-only, which
     the runner is).
   - `perf.rs` ignored tests run for their output only (report, not
     gate): shared runners are too noisy for the laptop budgets.
3. Packaging (optional, `workflow_dispatch` and tags `v*`): build the
   Containerfile image and run `scripts/package.sh`; upload the
   `.deb`/`.rpm` as artifacts. Decide in the spec whether this belongs
   here or in [[T53 Release 1.5.0]].
4. Pin every action to a full commit SHA, least-privilege
   `permissions: contents: read`, and a status badge in `README.md`.

## Spec

To be written in the spec step. Expected acceptance criteria:

- **AC-1** — A push to a branch runs `ci.yml`; a format, clippy, test or
  doc failure fails the run. *Test:* a throwaway branch with a
  deliberate clippy warning shows a red run; the fix shows green (linked
  in *Log*).
- **AC-2** — The MSRV job builds on the `rust-version` in `Cargo.toml`.
  *Test:* `tests/ci.rs` (new) reads `Cargo.toml` and `ci.yml` and checks
  the toolchain versions match.
- **AC-3** — `ci.yml` runs the same steps as `scripts/check.sh` (it
  calls it). *Test:* `tests/ci.rs` checks `ci.yml` invokes
  `scripts/check.sh`.
- **AC-4** — `nightly.yml` runs fuzz and soak on schedule and by hand,
  and uploads the regressions file on failure. *Test:* one manual
  `workflow_dispatch` run, linked in *Log*.
- **AC-5** — All actions are pinned to a SHA and the workflow token is
  read-only. *Test:* `tests/ci.rs` checks every `uses:` line has a
  40-hex ref and `permissions` is set.

## Out of scope

- Windows and macOS runners (draw targets Linux only).
- Publishing releases or uploading packages to a store.
- Branch protection rules (owner sets them in GitHub settings after
  AC-1 is green).
- Making `perf.rs` budgets pass on shared runners.

## Files owned

- `.github/workflows/ci.yml`, `.github/workflows/nightly.yml` (new)
- `tests/ci.rs` (new)
- `scripts/check.sh` (only if a CI-friendly flag is needed)
- `README.md` (badge line only)
- `docs/process/Quality Gates.md` (a *CI* section)
- `docs/decisions/ADR-T73-1 …` (new, if the MSRV or toolchain choice needs one)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T73): …`
- [ ] tests — `test(T73): …`
- [ ] models — `ci(T73): …` (workflow files)
- [ ] behaviour — `ci(T73): …` (nightly, packaging)
- [ ] quality — `chore(T73): …`
- [ ] docs — `docs(T73): …`

## Learning path

None expected.

## Log

- 2026-11-03 — note written at the owner's request.
