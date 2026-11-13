---
id: T157
title: macroquad Gpu interpreter
status: todo
wave: 68
branch: task/T157-macroquad-gpu-interpreter
depends_on: [T140]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T157 macroquad Gpu interpreter

## Goal

Slice S1 of [[T155 Gpu migration of the drawing]]: `src/shell/gpu_mq.rs`: buffers, textures, targets, pipelines and `submit` through `draw_mesh` and `RenderPass`, tested for chunk limits and handle checks. Nothing else in the shell changes.

## Known facts (2026-11-13)

- Split out of T155 after the batch blocked it (about 7,000 lines; every slice needs `e2e.sh --backend all`). The design and the slice list are in the T155 note; the traits are in [[ADR-T140-1 The Gpu and Platform traits]].
- No pixel change allowed.

## Spec

Draft: the spec step refines each criterion into named tests.

- **AC-1** — The slice's code emits `Gpu` calls only, as described in the goal.
- **AC-2** — `e2e.sh --backend all` green; no pixel change in the screenshot comparisons.

## Out of scope

- Other slices of T155; input, the loop and `main` ([[T156 Platform migration of input and the loop]]); anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `src/shell/gpu_mq.rs`, `src/shell/mod.rs` (one line)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T157): specify the slice`
- [ ] tests — `test(T157): add failing tests`
- [ ] models — `feat(T157): add the types`
- [ ] behaviour — `refactor(T157): migrate the slice`
- [ ] quality — `chore(T157): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T155 by the owner, following the batch's recommendation.
