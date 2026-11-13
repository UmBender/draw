---
id: T160
title: Toolbar and menus over Gpu
status: todo
wave: 68
branch: task/T160-toolbar-and-menus-over-gpu
depends_on: [T159]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T160 Toolbar and menus over Gpu

## Goal

Slice S4 of [[T155 Gpu migration of the drawing]]: `toolbar`, `sheet`, `command_palette` and `ui_shapes` primitives tessellated into `Gpu` draws.

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

- `src/shell/toolbar*`, `sheet*`, `command_palette*`, `ui_shapes*`
- call sites of the migrated API in other `src/shell/` files, and the shared interpreter accessor in `src/shell/render.rs` (added by T159), when the slice needs them to compile; a change to the `Gpu` trait still blocks
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T160): specify the slice`
- [ ] tests — `test(T160): add failing tests`
- [ ] models — `feat(T160): add the types`
- [ ] behaviour — `refactor(T160): migrate the slice`
- [ ] quality — `chore(T160): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T155 by the owner, following the batch's recommendation.
- 2026-11-13 — widened in advance by the owner after T158 and T159 blocked on *Files owned*: call sites in other shell files and the `render.rs` accessor are allowed; the `Gpu` trait is not.
