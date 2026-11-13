---
id: T155
title: Gpu migration of the drawing
status: todo
wave: 68
branch: task/T155-gpu-migration-of-the-drawing
depends_on: [T140]
adrs: []
feature: ""
tutorial: ""
tags: [task, 2.0]
---

# T155 Gpu migration of the drawing

## Goal

Everything the shell draws goes through the `Gpu` trait, with macroquad as one interpreter of it. Input and the loop are untouched.

## Known facts (2026-11-13)

- Split out of [[T140 Platform and Gpu traits with recording interpreters]] AC-2 after the batch blocked it. The traits, the recorder and [[ADR-T140-1 The Gpu and Platform traits]] already exist.
- What moves: `render.rs` `Batch`/`MeshSink` become `Command::Draw`; `glyphs.rs` and `minimap.rs` textures and the render target in `app.rs` become `Gpu` handles; `ui_text.rs` `draw_text_ex` becomes glyph quads; `toolbar`, `sheet`, `command_palette` and `ui_shapes` primitives become tessellated triangles.
- Pixels can be checked without a tablet (`e2e.sh`, screenshots), so this part can run unattended.

## Spec

Draft: the spec step refines each criterion into named tests and ADRs.

- **AC-1** — The drawing path (render, glyphs, minimap, ui, frame cache) emits `Gpu` calls only; no macroquad drawing call remains outside the macroquad `Gpu` interpreter.
- **AC-2** — A test draws a scene over `RecordingGpu` and asserts the command list.
- **AC-3** — `e2e.sh --backend all` green; no pixel change in the screenshot comparisons.

## Out of scope

- Input, the loop and `main` (see [[T156 Platform migration of input and the loop]]); anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `src/shell/render.rs`, `glyphs.rs`, `minimap.rs`, `ui_text.rs`, `toolbar`, `sheet`, `command_palette`, `ui_shapes`, the drawing part of `app.rs`
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T155): specify the Gpu migration of the drawing`
- [ ] tests — `test(T155): add failing tests for the drawing over the recorder`
- [ ] models — `feat(T155): add the macroquad Gpu interpreter`
- [ ] behaviour — `refactor(T155): draw through the Gpu trait`
- [ ] quality — `chore(T155): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T140 AC-2 by the owner, following the batch's recommendation.
