---
tags: [process]
---

# Quality Gates

All gates run with one command:

```sh
scripts/check.sh          # fmt check, clippy -D warnings, tests, docs
scripts/check.sh --fuzz   # same + long fuzz run (PROPTEST_CASES=20000)
```

| Gate | Command | Must |
|------|---------|------|
| Format | `cargo fmt --all -- --check` | no diff |
| Lint | `cargo clippy --all-targets -- -D warnings` | zero warnings (lint levels in `Cargo.toml` `[lints]`, see [[ADR-0010 Clippy lint policy]]) |
| Tests | `cargo test --all-targets` | all pass, including property tests; `tests/no_dyn.rs` fails on `dyn` outside the allow-list of [[ADR-T131-1 No dyn outside an allow-list]] |
| Docs | `cargo doc --no-deps` with `RUSTDOCFLAGS=-D warnings` | no broken intra-doc links |
| Fuzz | `PROPTEST_CASES=20000 cargo test --release --test fuzz` | no failure (see [[ADR-0009 Fuzzing with proptest]]) |
| Coverage | `bash scripts/coverage.sh` (needs `cargo-llvm-cov`; separate from `check.sh`, the `coverage` job of `ci.yml`) | line coverage of each `src/` module within 0.5 points of its value in `coverage-baseline.toml`; a rise is recorded with `scripts/coverage.sh --update` (T130) |
| E2E | `scripts/e2e.sh [--backend wayland\|x11\|all]` (Wayland needs `sway`, `wtype`, `grim`; X11 needs `Xvfb`, `xdotool`, ImageMagick `import`; separate from `check.sh`, nightly in CI) | every case passes on the backends run (see [[End-to-end tests]]) |
| Release | `scripts/release.sh X.Y.Z` | `check.sh --fuzz`, then e2e on Wayland and on X11, then `package.sh`; any failing step stops before packaging. `package.sh` called directly says it is not the release path. `--dry-run` prints the steps. |

## Limits

Every `cargo` step of `scripts/check.sh` leaves the desktop usable
([[ADR-T77-1 Test gates run in capped low-priority scopes]]). The values
are named at the top of the script.

| Limit | Value | Applied with |
|-------|-------|--------------|
| Build jobs | 2 in `check.sh`, 3 in `e2e.sh` | `CARGO_BUILD_JOBS` |
| Test threads | 3 | `RUST_TEST_THREADS` |
| Memory | 1.5 GB (`1536M`) in `check.sh`, 1 GB in `e2e.sh`, no swap | `MemoryMax`, `MemorySwapMax` on a `systemd-run --user --scope` |
| CPU | 3 CPUs | `CPUQuota` on the scope |
| Priority | lowest | `nice`, `ionice` idle class, `CPUWeight=1`, `IOWeight=1` |

- A step that goes over the memory limit is stopped and `check.sh` fails
  with `<step> went over the 1536M memory limit`. Raise `MEMORY_MAX` only if
  the step really needs it.
- Without a usable `systemd-run --user` (CI runners), `check.sh` prints
  that the memory cap is not enforced and keeps the rest.
- A target dir never goes on `/tmp` here: it is a tmpfs, so its files
  count as memory.

## CI

GitHub Actions runs the same gates on every push and pull request
([[T73 Continuous integration on GitHub]]):

| Workflow | When | Jobs |
|----------|------|------|
| `.github/workflows/ci.yml` | push (any branch), pull request to `main` | `gates`: `scripts/check.sh` on stable; `coverage`: `bash scripts/coverage.sh`; `msrv`: `cargo check --all-targets --locked` on `rust-version` |
| `.github/workflows/nightly.yml` | daily 04:17 UTC, or *Run workflow* by hand | `fuzz`: `scripts/check.sh --fuzz`, uploads `tests/fuzz.proptest-regressions` on failure; `soak`: ignored soak tests; `e2e`: `scripts/e2e.sh`, uploads `target/e2e/` on failure; `perf`: ignored perf tests, report only |

- CI calls `scripts/check.sh`, so a new gate is added there, once.
- `rust-version` follows the latest stable
  ([[ADR-T73-1 Track the latest stable Rust]]); the msrv job, the
  packaging image and the README must agree with it (`tests/ci.rs`).
- A red nightly `fuzz` job leaves an artifact `fuzz-regressions`: add
  its new lines to `tests/fuzz.proptest-regressions` and fix the bug as
  a regression ([[Workflow#Bug fixes: regression step]]).
- Actions are pinned to commit SHAs (tag in a comment) and the token is
  read-only. To update an action, replace the SHA with the new tag's
  commit (`git ls-remote --tags https://github.com/<owner>/<repo>`).

## Definition of Done (per task)

- [ ] Every `AC-n` in the spec has at least one passing test.
- [ ] Every bug fixed has a `regression_<ID>_<slug>` test (snapshot-based
      once T74 is done) committed red before the fix (see
      [[Workflow#Bug fixes: regression step]]).
- [ ] `scripts/check.sh` is green on the branch head.
- [ ] Coverage ratchet holds: no `src/` module fell, and a rise updated `coverage-baseline.toml` (`bash scripts/coverage.sh`, run by CI).
- [ ] Only files in *Files owned* were touched.
- [ ] Feature note + tutorial note written; learning-path position stated.
- [ ] New architecture choices recorded as ADRs.
- [ ] Commit history follows [[Commit Convention]] (one commit per step).
- [ ] Task note status is `review`.

## Code rules

Follow the `rust-skills` skill. Most relevant categories for this project:
`own-`, `err-`, `mem-`, `api-`, `num-` (floats/NaN!), `type-`, `pat-`,
`name-`, `test-`, `doc-`, `perf-`, `proj-`, `lint-`, `anti-`. Not relevant:
`async-`, `conc-`, `serde-`, `unsafe-` (`unsafe` only where [[ADR-T132-1 Workspace of five crates with unsafe levels]] lists it, checked by `tests/unsafe_levels.rs`; dependencies are checked by `tests/deps.rs`).
