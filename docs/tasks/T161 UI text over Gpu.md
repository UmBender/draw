---
id: T161
title: UI text over Gpu
status: review
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

- **AC-1** — `ui_text.rs` uses no macroquad draw call (`draw_text_ex` and the calls of the goal): a label is drawn as glyph quads of a glyph atlas through a `Batch` that ends in the `Gpu` interpreter (T158). Test `ui_text_uses_no_macroquad_draw_calls` (source scan).
- **AC-2** — The quads are pure: `label_quads` places each glyph of a label at the snapped pen position, advancing by the atlas advance, and emits nothing for spaces. Tests `label_quads_place_glyphs_along_the_baseline`, `label_quads_skip_spaces`, `label_quads_snap_the_origin_to_physical_pixels`. Metrics do not change: `measure` still reports `measure_text` of the same font (existing behaviour, kept).
- **AC-3** — `e2e.sh --backend all` green; no pixel change in the screenshot comparisons.

Design (no new ADR): the UI font is the bundled text font, so the atlas is a `GlyphAtlas` over `TEXT_FONT` owned by `ui_text.rs`, rasterized at `raster_px(font_size, dpi)` (exactly what macroquad's `draw_text_ex` rasterizes: font size times DPI), drawn at scale `glyph_scale`. Its pages are uploaded with `GlyphTextures<MqGpu>`; the chunks go through `render::draw_inline` (made `pub`: the shared-interpreter accessor).

## Out of scope

- Other slices of T155; input, the loop and `main` ([[T156 Platform migration of input and the loop]]); anything in another task of [[T50 Version 2.0.0 refactor]].

## Files owned

- `src/shell/ui_text.rs`
- `src/shell/glyphs.rs` (the shared atlas)
- call sites of the migrated API in other `src/shell/` files, and the shared interpreter accessor in `src/shell/render.rs` (added by T159), when the slice needs them to compile; a change to the `Gpu` trait still blocks
- the `no_macroquad_draw_calls_outside_the_interpreter` test, wherever it lives
- this note

## Subtasks (one commit each)

- [x] spec — `docs(T161): specify the slice`
- [x] tests — `test(T161): add failing tests`
- [x] models — merged into the behaviour commit
- [x] behaviour — `refactor(T161): migrate the slice`, plus `refactor(T161): move the overlay draws to Batch and scan for draw calls`
- [x] quality — `chore(T161): pass clippy and rustfmt`

## Log

- 2026-11-13 — split from T155 by the owner, following the batch's recommendation.
- 2026-11-13 — widened in advance by the owner after T158 and T159 blocked on *Files owned*: `glyphs.rs`, call sites in other shell files, the `render.rs` accessor and the scan test are allowed; the `Gpu` trait is not.
- 2026-11-13 — done: `ui_text::draw` emits glyph quads of a menu `GlyphAtlas` (own atlas over `TEXT_FONT`, rasterized at `raster_px(size, dpi)`, so the same size macroquad used) through `Batch` and the shared interpreter; `measure` is unchanged (`measure_text`). `render::draw_inline` made `pub`. UNATTENDED CHOICE: the shared canvas atlas has no public accessor and quantizes label sizes, so the menus keep their own atlas (exact sizes, pixel parity). The remaining macroquad draw calls (text-edit caret, marquee, laser trail in `app.rs`) moved to `Batch`; the marquee outline is now `ui_shapes::push_ring` (1 px inside) instead of `draw_rectangle_lines`, and the laser end uses `Batch::disc` (segment count from `circle_segments`): owner to check by eye. `no_macroquad_draw_calls_outside_the_interpreter` passes. `scripts/check.sh` and `e2e.sh --backend all` green. No feature or tutorial note (task has none).
