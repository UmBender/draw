---
id: T73
title: Continuous integration on GitHub
status: done
wave: 33
branch: task/T73-github-ci
depends_on: []
adrs: ["[[ADR-T73-1 Track the latest stable Rust]]"]
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

Settled 2026-11-03:

- **Toolchain** ([[ADR-T73-1 Track the latest stable Rust]]): draw does
  not build on the old `rust-version = "1.85"` (`fontdue 0.9.4` needs
  1.87). The owner chose to follow the latest stable: `rust-version =
  "1.99"`, the Containerfile pins 1.99.1, the README says 1.99.
- **Headless**: `cargo test --all-targets` passes with `DISPLAY` and
  `WAYLAND_DISPLAY` unset (miniquad uses `dlopen`), so no `xvfb-run`
  and no system packages are needed on the runner.
- **`ci.yml`** — `push` (all branches) and `pull_request` to `main`;
  `concurrency` cancels older runs of the same ref; `permissions:
  contents: read`. Jobs on `ubuntu-latest`:
  - `gates`: checkout, `dtolnay/rust-toolchain` (`stable`, `rustfmt`,
    `clippy`), `Swatinem/rust-cache`, then `scripts/check.sh`.
  - `msrv`: toolchain `1.99` (= `rust-version`), cache,
    `cargo check --all-targets --locked`.
- **`nightly.yml`** — `schedule` (daily, 04:17 UTC) and
  `workflow_dispatch`; same permissions. Jobs:
  - `fuzz`: `scripts/check.sh --fuzz`; `if: failure()` uploads
    `tests/fuzz.proptest-regressions` with `actions/upload-artifact`.
  - `soak`: `cargo test --release --test soak -- --ignored --nocapture
    --test-threads=1`.
  - `perf`: `cargo test --release --test perf -- --ignored --nocapture
    --test-threads=1` with `continue-on-error: true` (report only).
- **Packaging** stays with [[T53 Release 1.5.0]]: it needs the
  container build and release artifacts, and nothing in this task
  changes how packages are made.
- Every `uses:` is pinned to a 40-hex commit SHA with the tag in a
  comment. `scripts/check.sh` needs no CI flag.

Acceptance criteria:

- **AC-1** — A push to a branch runs `ci.yml`; a format, clippy, test or
  doc failure fails the run. *Test:* a throwaway branch with a
  deliberate clippy warning shows a red run, the fix shows green (links
  in *Log*). Static part: `tests/ci.rs`
  `ci_workflow_runs_on_push_and_pull_request`.
- **AC-2** — The MSRV job builds on the `rust-version` in `Cargo.toml`,
  and the packaging image is not older. *Test:* `tests/ci.rs`
  `ci_msrv_job_uses_cargo_rust_version`,
  `containerfile_rust_is_at_least_rust_version`,
  `readme_build_requirement_names_rust_version`.
- **AC-3** — `ci.yml` runs the same steps as `scripts/check.sh` (it
  calls it). *Test:* `tests/ci.rs` `ci_gates_job_runs_check_script`.
- **AC-4** — `nightly.yml` runs fuzz and soak on schedule and by hand,
  and uploads the regressions file on failure. *Test:* `tests/ci.rs`
  `nightly_runs_fuzz_and_soak_on_schedule_and_dispatch`,
  `nightly_uploads_fuzz_regressions_on_failure`; one manual
  `workflow_dispatch` run, linked in *Log*.
- **AC-5** — All actions are pinned to a SHA and the workflow token is
  read-only. *Test:* `tests/ci.rs`
  `workflow_actions_are_pinned_to_full_sha`,
  `workflow_permissions_are_read_only`; plus
  `readme_has_ci_badge`.

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
- `README.md` (badge line and the Rust requirement line)
- `Cargo.toml` (`rust-version` line only, ADR-T73-1)
- `packaging/Containerfile` (`RUST_VERSION` only, ADR-T73-1)
- clippy fixes forced by the 1.99 `rust-version` (owner approved):
  `src/core/smoothing.rs`, `src/core/tex.rs`, `src/shell/app.rs`,
  `src/shell/crash.rs`, `src/shell/diag/file.rs`, `src/shell/diag/mod.rs`,
  `src/shell/render.rs`, `tests/perf.rs`, `tests/soak.rs`
- `docs/process/Quality Gates.md` (a *CI* section)
- `docs/decisions/ADR-T73-1 Track the latest stable Rust.md` (new)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T73): …`
- [x] tests — `test(T73): …`
- [x] models — `ci(T73): …` (workflow files)
- [x] behaviour — `ci(T73): …` (nightly; packaging moved to T53)
- [x] quality — `chore(T73): …`
- [x] docs — `docs(T73): …`

## Learning path

None expected.

## Log

- 2026-11-03 — note written at the owner's request.
- 2026-11-03 — spec: `cargo +1.85 check` fails in `fontdue`; 1.87 builds.
  Owner chose to track the latest stable instead (ADR-T73-1); `Cargo.toml`,
  `packaging/Containerfile` and the README line added to *Files owned*.
- 2026-11-03 — tests: `tests/ci.rs` (10 tests) red before the workflows
  existed; the Containerfile and README guards passed on the old values,
  as they should.
- 2026-11-03 — quality: `rust-version = "1.99"` switches on clippy lints
  gated on the MSRV (let-chain `collapsible_if`, `is_multiple_of`,
  `Duration::from_hours`); 11 sites fixed with `cargo clippy --fix`, no
  behaviour change. `scripts/check.sh` green locally (stable 1.99.1).
- 2026-11-03 — headless: `cargo test --all-targets` passes with
  `DISPLAY`/`WAYLAND_DISPLAY` unset (15 s on the laptop); no system
  packages or `xvfb-run` on the runner.
- 2026-11-04 — AC-1: task branch green
  (https://github.com/UmBender/draw/actions/runs/41755050175). Throwaway
  branch `ci-check-T73` with a deliberate clippy warning went red in
  `gates`, `msrv` green
  (https://github.com/UmBender/draw/actions/runs/41755771190); the fix
  went green (https://github.com/UmBender/draw/actions/runs/41755897634).
  Branch deleted. The first push of that branch started no run at all
  (GitHub dropped the event); a second commit triggered it.
- 2026-11-04 — AC-4 static part green; the manual run is pending:
  `workflow_dispatch` needs `nightly.yml` on the default branch, so the
  integrator runs `gh workflow run nightly.yml` after merging and pushing
  `main`, and links the run here.
- 2026-11-04 — integrated on `main` (merge `6b7988b`), no conflicts.
- 2026-11-04 — AC-4: manual `workflow_dispatch` of `nightly.yml` on
  `main` green: fuzz (20 000 cases), soak and perf report
  (https://github.com/UmBender/draw/actions/runs/41789023411). CI on the
  merged `main` green too
  (https://github.com/UmBender/draw/actions/runs/41788998134).
