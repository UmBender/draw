---
id: T194
title: Nightly Miri and X11 end-to-end jobs
status: in-progress
wave: 77
branch: fix/nightly-miri-and-x11-e2e
depends_on: []
adrs: []
feature: ""
tutorial: ""
tags: [task, bug, ci]
---

# T194 Nightly Miri and X11 end-to-end jobs

## Goal

The Nightly and CI workflows are green again: the `unsafe (miri and
sanitizers)` job runs Miri over the unit tests of `draw-sys` to the end,
the `end-to-end` job runs the X11 half of the suite as well as the
Wayland half, the code passes the clippy of the latest stable Rust
(1.100) that CI uses, and the `gates` and `coverage` jobs have the
programs the integration tests start.

## Known facts (2026-11-24)

- Found by the owner's request to look at CI after pushing `main`: the
  Nightly failed on 2026-11-23 and 2026-11-24 (run 45419361480), with two
  jobs red.
- **Miri**: `scripts/unsafe-tests.sh` runs
  `cargo +nightly miri test -q -p draw-sys --lib`, which stops with
  `unsupported operation: can't call foreign function sigaltstack on OS linux`.
  The test is `signal::tests::fatal_signal_hook_is_installed`
  (`install_fatal` calls `sigaltstack` and `sigaction`). Miri aborts the
  whole run at the first unsupported call, so later tests never ran.
  The `loader.rs` tests already carry
  `#[cfg_attr(miri, ignore = "Miri cannot call dlopen")]`.
  Reproduced locally (Miri 2026-11-12).
- **X11 end-to-end**: every Wayland case passed, then `scripts/e2e.sh`
  stopped with `e2e.sh: xdotool is not installed`. The job's
  `apt-get install` has the Wayland tools (`sway`, `wtype`, `grim`) but
  not the X11 ones `run_x11` needs (`Xvfb`, `xdotool`, ImageMagick's
  `import`), nor the X11 libraries of `packaging/runtime-libs`
  (`libx11-6`, `libxi6`, `libgl1`).
- **Scope widened the same day** (the owner asked for the Nightly to be
  fixed; these failures came up while checking it):
  - **Clippy 1.100.** CI installs the latest stable (1.100); the local
    toolchain is 1.99.1. Clippy 1.100 extends `map_unwrap_or` to
    `.map(f).unwrap_or_default()`: 7 sites in `draw-sys`, `draw-core`
    and the shell fail `-D warnings`. It fails the `gates` job of CI
    (red on `main` since at least 2026-11-18) and the `fuzz` job of the
    Nightly, which runs `check.sh --fuzz`. `Option::map_or_default`,
    the fix clippy proposes, compiles on 1.99 (checked), so the `msrv`
    job stays green.
  - **CI test jobs without sway.** `gates` and `coverage` install only
    `xvfb`, but the `draw-sys` integration tests start `sway`, `wtype`,
    `xclip`, `xdotool` and `Xvfb` and fail, never skip, without them
    (`coverage` red since T168; `gates` stops at clippy first).

## Spec

- **AC-1** — Regression: the `end-to-end` job installs a package for
  every tool `scripts/e2e.sh` checks with `need_tools`.
  Test: `regression_T194_nightly_e2e_installs_every_e2e_tool`
  (`tests/ci.rs`; a table maps each tool to its Ubuntu package and fails
  on a tool it does not know).
- **AC-2** — The `end-to-end` job installs the Debian package of every
  `required`, `gl` and `wayland` line of `packaging/runtime-libs`.
  Test: `nightly_e2e_installs_the_runtime_libraries` (`tests/ci.rs`).
- **AC-3** — Miri runs every unit test of `draw-sys` it can run, and the
  tests that call `sigaltstack` or `sigaction` are ignored under Miri
  with a reason, like the `dlopen` ones.
  Check: `scripts/unsafe-tests.sh` passes locally with nightly Miri
  (red before the fix), and the Nightly workflow run by hand on this
  branch has both jobs green.
- **AC-4** — The workspace passes
  `cargo +1.100.0 clippy --workspace --all-targets -- -D warnings`.
  Check: the `gates` job of CI and the `fuzz` job of the Nightly on this
  branch (red before), and the command locally.
- **AC-5** — Regression: the jobs that run every test (`gates` and
  `coverage` of CI, `fuzz` of the Nightly, which runs `check.sh --fuzz`)
  install a package for every program the tests start, by name in
  `Command::new` or through the `tool` helpers (`wl-copy`, `wl-paste`).
  Test: `regression_T194_ci_test_jobs_install_every_test_program`
  (`tests/ci.rs`; a table maps each program to its Ubuntu package and
  fails on a program it does not know).

## Out of scope

- Raising `rust-version` to 1.100 ([[ADR-T73-1 Track the latest stable Rust]]):
  a separate change with the README and the packaging image.
- The new Cargo 1.100 warnings "unused dependency `fontdue`" and
  "`png`" in `crates/draw-core/Cargo.toml` (warnings, not failures).
- Running Miri on more crates.

## Files owned

- `.github/workflows/nightly.yml` (the `end-to-end` job's packages)
- `.github/workflows/ci.yml` (the `gates` and `coverage` jobs' packages)
- `tests/ci.rs`
- `crates/draw-sys/src/**/*.rs` (`cfg_attr(miri, ignore …)` lines on tests only)
- `crates/draw-sys/tests/x11.rs` (`x11_window_has_title_and_class`)
- the `map_unwrap_or` sites: `crates/draw-sys/src/wayland/clipboard.rs`,
  `crates/draw-core/src/core/document.rs`,
  `crates/draw-core/src/core/tools/shape_tool.rs`,
  `crates/draw-core/src/core/editor.rs` (tests), `src/shell/file.rs`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T194): specify the Nightly fixes`
- [x] regression — `test(T194): add regression test for the e2e job's tools`
- [x] behaviour — `fix(T194): install the X11 tools and skip FFI tests under Miri`
- [x] regression — `test(T194): add regression test for the CI jobs' programs`
- [x] behaviour — `fix(T194): pass clippy 1.100 and install the test programs in CI`
- [x] regression — `test(T194): check the test programs of every full test job`
- [x] behaviour — `fix(T194): install the test programs in the fuzz job and wl-clipboard`
- [x] regression — `test(T194): read WM_CLASS with xprop in the X11 test`
- [x] behaviour — `fix(T194): install x11-utils for xprop`
- [ ] quality — `chore(T194): pass clippy and rustfmt`
- [ ] docs — `docs(T194): update the notes`

No models step: no new type.

## Log

- 2026-11-24 — written from the failed Nightly runs, at the owner's
  request.
- 2026-11-24 — first fix pushed; the Nightly run by hand on the branch
  (45483757426) had `unsafe` and `end-to-end` green (X11 included), and
  `fuzz` red on clippy 1.100. The CI run of `main` (45482247657) was red
  on the same clippy lint and on sway missing in `coverage`. Scope
  widened (Known facts), with AC-4 and AC-5.
- 2026-11-24 — second CI run on the branch (45489008082, Nightly
  45489008492): clippy passed; `gates` and `coverage` then failed on
  `wl-copy`/`wl-paste` (the clipboard tests start them through a `tool`
  helper, which the first scan of `Command::new` literals missed), and
  the Nightly `fuzz` job failed on the GL tests because it installs no
  package at all. AC-5 now scans the `tool` helpers too and covers
  `fuzz`.
- 2026-11-24 — third run (CI 45490508476, Nightly 45490509505): the
  only failure left in `gates`, `coverage` and `fuzz` was
  `x11_window_has_title_and_class`: `xdotool getwindowclassname`
  (xdotool 3.20211022 on Fedora) does not exist in the xdotool of the
  Ubuntu runners, which prints nothing. The test reads `WM_CLASS` with
  `xprop` (package `x11-utils`) instead.
