---
id: T140
title: Platform and Gpu traits with recording interpreters
status: todo
wave: 68
branch: task/T140-platform-and-gpu-traits-with-recording-i
depends_on: [T132, T137]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T140 Platform and Gpu traits with recording interpreters

## Goal

The shell is written against `Platform` and `Gpu` traits, run with generics, and tested with a recording and a null interpreter, while still on macroquad.

## Known facts (2026-11-12)

- [[2.0 Architecture proposal]] §4. No `dyn`: `fn run<P: Platform, G: Gpu>()`.
- The `Gpu` trait is reviewed against Vulkan's model (explicit typed resources, one recorded command list per frame, no hidden global state) before any backend is written.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — `Platform` (poll input, clipboard, window size and scale, schedule) and `Gpu` (buffers, textures, pipelines, command list) are defined with associated types.
  Tests: the recording interpreter keeps every call in order (`recorder_keeps_the_call_order`).
- **AC-2** — The macroquad code is one interpreter of the traits; `main` picks it with a `match` and monomorphises.
  `e2e.sh` green on both backends.
- **AC-3** — A review note in the ADR shows each `Gpu` method mapped to OpenGL ES 3.0 and to Vulkan with no case that needs hidden state.
  `ADR-T140-1`.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-gfx/src/gpu.rs`, `crates/draw-sys/src/platform.rs` (traits only), `crates/draw/src/**`
- this note and `ADR-T140-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T140): specify the Platform and Gpu traits`
- [ ] tests — `test(T140): add failing tests for the recorder`
- [ ] models — `feat(T140): add the traits and the interpreters`
- [ ] behaviour — `refactor(T140): run the shell over the traits`
- [ ] quality — `chore(T140): pass clippy and rustfmt`

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions.
