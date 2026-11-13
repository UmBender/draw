---
id: T142
title: draw-sys foundation with dlopen, safe wrappers and the unsafe test set
status: done
wave: 69
branch: task/T142-draw-sys-foundation-dlopen-safe-wrappers
depends_on: [T132, T140]
adrs: [ADR-T142-1]
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T142 draw-sys foundation with dlopen, safe wrappers and the unsafe test set

## Goal

The first `unsafe` code: loading C libraries at run time and the contract every later primitive follows.

## Known facts (2026-11-12)

- [[2.0 Architecture proposal]] §5. The C libraries stay (libwayland-client, libwayland-egl, libxcb, libEGL, libxkbcommon), loaded with `dlopen` as the vendored miniquad does.

## Spec

Decisions: [[ADR-T142-1 dlopen through libc symbols and TEST markers]].

- **AC-1** — `Library` and `Symbol<F>` wrap `dlopen`/`dlsym`: a missing library or symbol is an error value, a symbol cannot outlive its library, and the function type is fixed at load.
  Unit tests for every error path; Miri where the build allows it; regression tests per bug.
  Tests in `loader.rs`: `open_missing_library_returns_not_found`, `open_name_with_nul_returns_invalid_name`, `open_existing_library_succeeds`, `symbol_missing_returns_symbol_not_found`, `symbol_name_with_nul_returns_invalid_name`, `symbol_of_wrong_size_returns_bad_type`, `symbol_present_can_be_called`, `error_messages_name_the_library_or_symbol`; a `compile_fail` doctest `symbol_cannot_outlive_library`. Integration test `crates/draw-sys/tests/loader.rs`: `libm_cos_through_the_public_api`.
- **AC-2** — A checklist file `docs/process/Unsafe primitives.md` lists the six obligations (safe wrapper, `# Safety`, unit tests, integration tests, regression suite, sanitizers and fuzz); a script checks that each `unsafe` block names its test.
  Test: `every_unsafe_block_has_a_test_reference` in `crates/draw-sys/tests/unsafe_refs.rs`, plus checker tests `marker_missing_is_reported`, `marker_naming_unknown_test_is_reported`, `marker_on_previous_lines_is_accepted`, `unsafe_in_comment_is_ignored`. `scripts/unsafe-tests.sh` runs it.
- **AC-3** — The nightly job runs the address and thread sanitizers on the integration tests of `draw-sys`.
  Test: `tests/ci.rs` is untouched; `scripts/unsafe-sanitizers.sh` is the job's one command, and the job `sanitizers` is added to `nightly.yml`. Nightly toolchain is present locally (Miri runs); the sanitizer run is tried locally and recorded in the log.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-sys/src/loader.rs`, `crates/draw-sys/src/lib.rs` (the `pub mod loader;` line only), `crates/draw-sys/tests/`, `docs/process/Unsafe primitives.md`, `.github/workflows/nightly.yml`, `scripts/`
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T142): specify the loader and the unsafe contract`
- [x] tests — `test(T142): add failing tests for the loader`
- [x] models — `feat(T142): add the library and symbol types`
- [x] behaviour — `feat(T142): implement dlopen with safe wrappers`
- [x] quality — `chore(T142): pass clippy and rustfmt`
- [x] docs — `docs(T142): add the unsafe primitives note`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
- 2026-11-13 — BLOCKED (batch): `crates/draw-sys/src/loader.rs` cannot compile or be tested without `pub mod loader;` in `crates/draw-sys/src/lib.rs` (which has `#![deny(unsafe_code)]`; the module needs a local `#![allow(unsafe_code)]`). `lib.rs` is not in *Files owned*, and `crates/draw-sys/tests/` (integration tests for AC-3) is not either. Also, `rustup`/`git`-compound commands were refused, so nightly/Miri availability was not checked. Recommended fix: add `crates/draw-sys/src/lib.rs` and `crates/draw-sys/tests/` to *Files owned*, then re-run the task.
- 2026-11-13 — Files owned amended by the owner after the batch blocked: `lib.rs` (one `pub mod loader;` line) and `crates/draw-sys/tests/` (AC-3 integration tests) added.
- 2026-11-13 — done: loader (`Library`, `Symbol`, `LoadError`), marker checker, `scripts/unsafe-tests.sh`, `scripts/unsafe-sanitizers.sh`, nightly job `sanitizers`, [[Unsafe primitives]]. `scripts/check.sh` green. Local runs: Miri runs the unit tests that do not call `dlopen` (Miri cannot call it, the rest are ignored under Miri); address and thread sanitizers pass on the integration tests. UNATTENDED CHOICE: the `compile_fail` doctest is named by rustdoc (`loader::Library`), not `symbol_cannot_outlive_library`; doctests are not in `check.sh` (`--all-targets`), so `scripts/unsafe-tests.sh` runs it (`cargo test -p draw-sys --doc`). Quality step: clippy/fmt fixes were committed as `chore`. The new nightly job is untested on GitHub.
