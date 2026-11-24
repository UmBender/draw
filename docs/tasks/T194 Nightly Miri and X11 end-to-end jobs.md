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

The Nightly workflow is green again: the `unsafe (miri and sanitizers)`
job runs Miri over the unit tests of `draw-sys` to the end, and the
`end-to-end` job runs the X11 half of the suite as well as the Wayland
half.

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

## Out of scope

- The other Nightly jobs (fuzz, soak, perf).
- Running Miri on more crates.

## Files owned

- `.github/workflows/nightly.yml` (the `end-to-end` job's packages)
- `tests/ci.rs`
- `crates/draw-sys/src/**/*.rs` (`cfg_attr(miri, ignore …)` lines on tests only)
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T194): specify the Nightly fixes`
- [x] regression — `test(T194): add regression test for the e2e job's tools`
- [x] behaviour — `fix(T194): install the X11 tools and skip FFI tests under Miri`
- [ ] quality — `chore(T194): pass clippy and rustfmt`
- [ ] docs — `docs(T194): update the notes`

No models step: no new type.

## Log

- 2026-11-24 — written from the failed Nightly runs, at the owner's
  request.
