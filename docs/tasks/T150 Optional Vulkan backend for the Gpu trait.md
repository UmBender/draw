---
id: T150
title: Optional Vulkan backend for the Gpu trait
status: todo
wave: 74
branch: task/T150-optional-vulkan-backend-for-the-gpu-trai
depends_on: [T149]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0, optional]
---

# T150 Optional Vulkan backend for the Gpu trait

## Goal

A Vulkan implementation of `Gpu`, chosen at start-up, built only if a measurement asks for it. Not part of the 2.0.0 release.

## Known facts (2026-11-12)

- Condition (proposal, *OpenGL or Vulkan*): the audit or a benchmark shows that driver overhead or frame pacing is the ceiling on the target, or the owner drops hardware without Vulkan. If the condition does not hold this task stays `todo` or is closed as not needed.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — The condition is stated with the measurement that triggers it, in `ADR-T150-1`, before any code.
- **AC-2** — If built: instance, device, swapchain with resize and suboptimal handling, synchronisation and an own memory allocator, each as an `unsafe` primitive with the full set of [[T142 draw-sys foundation with dlopen, safe wrappers and the unsafe test set]] and validation layers on in the tests.
  `Gpu` conformance tests shared with the OpenGL backend.
- **AC-3** — Selected by a `match` in `main`, monomorphised, still no `dyn`; the e2e suite passes with it.

## Out of scope

- Anything in another task of [[T50 Version 2.0.0 refactor]]; new user-facing features.

## Files owned

- `crates/draw-sys/src/vulkan/**`, `crates/draw-gfx/src/gpu/vulkan.rs`
- this note and `ADR-T150-*`

## Subtasks (one commit each)

- [ ] spec — `docs(T150): decide whether Vulkan is needed`
- [ ] (only if built) tests, models, behaviour, quality and docs as in the workflow

## Log

- 2026-11-12 — written from [[2.0 Architecture proposal]] and the owner's decisions. Optional: built only if its condition holds.
