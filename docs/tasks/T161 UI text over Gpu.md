---
id: T161
title: UI text over Gpu
status: todo
wave: 68
branch: task/T161-ui-text-over-gpu
depends_on: [T160]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T161 UI text over Gpu

## Goal

Slice S5 of [[T155 Gpu migration of the drawing]]: `ui_text.rs`: UI text as glyph quads from the shared atlas; the font metrics must not change (pixel parity). Final check: `no_macroquad_draw_calls_outside_the_interpreter` (a source scan for `draw_mesh|draw_line|draw_rectangle|draw_texture_ex|draw_text_ex|render_target_ex|set_camera` outside `gpu_mq.rs`).

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

- `src/shell/ui_text.rs`
- `src/shell/glyphs.rs` (the shared atlas)
- call sites of the migrated API in other `src/shell/` files, and the shared interpreter accessor in `src/shell/render.rs` (added by T159), when the slice needs them to compile; a change to the `Gpu` trait still blocks
- the `no_macroquad_draw_calls_outside_the_interpreter` test, wherever it lives
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T161): specify the slice`
- [ ] tests — `test(T161): add failing tests`
- [ ] models — `feat(T161): add the types`
- [ ] behaviour — `refactor(T161): migrate the slice`
- [ ] quality — `chore(T161): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T155 by the owner, following the batch's recommendation.
- 2026-11-13 — widened in advance by the owner after T158 and T159 blocked on *Files owned*: `glyphs.rs`, call sites in other shell files, the `render.rs` accessor and the scan test are allowed; the `Gpu` trait is not.
