---
id: T142
title: draw-sys foundation with dlopen, safe wrappers and the unsafe test set
status: todo
wave: 69
branch: task/T142-draw-sys-foundation-dlopen-safe-wrappers
depends_on: [T132, T140]
adrs: []
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

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `Library` and `Symbol<F>` wrap `dlopen`/`dlsym`: a missing library or symbol is an error value, a symbol cannot outlive its library, and the function type is fixed at load.
  Unit tests for every error path; Miri where the build allows it; regression tests per bug.
- **AC-2** — A checklist file `docs/process/Unsafe primitives.md` lists the six obligations (safe wrapper, `# Safety`, unit tests, integration tests, regression suite, sanitizers and fuzz); a script checks that each `unsafe` block names its test.
  Test: `every_unsafe_block_has_a_test_reference`.
- **AC-3** — The nightly job runs the address and thread sanitizers on the integration tests of `draw-sys`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-sys/src/loader.rs`, `docs/process/Unsafe primitives.md`, `.github/workflows/nightly.yml`, `scripts/`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T142): specify the loader and the unsafe contract`
- [ ] tests — `test(T142): add failing tests for the loader`
- [ ] models — `feat(T142): add the library and symbol types`
- [ ] behaviour — `feat(T142): implement dlopen with safe wrappers`
- [ ] quality — `chore(T142): pass clippy and rustfmt`
- [ ] docs — `docs(T142): add the unsafe primitives note`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
