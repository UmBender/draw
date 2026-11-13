---
id: T159
title: Minimap and frame cache over Gpu
status: todo
wave: 68
branch: task/T159-minimap-and-frame-cache-over-gpu
depends_on: [T158]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T159 Minimap and frame cache over Gpu

## Goal

Slice S3 of [[T155 Gpu migration of the drawing]]: `minimap.rs` and the MSAA frame cache in `app.rs` (render target, blit) on `Gpu` handles.

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

- `src/shell/minimap.rs`, the frame cache part of `src/shell/app.rs`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T159): specify the slice`
- [ ] tests — `test(T159): add failing tests`
- [ ] models — `feat(T159): add the types`
- [ ] behaviour — `refactor(T159): migrate the slice`
- [ ] quality — `chore(T159): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T155 by the owner, following the batch's recommendation.
