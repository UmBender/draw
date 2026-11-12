---
id: T114
title: Export to SVG
status: done
wave: 59
branch: task/T114-svg-export
depends_on: []
adrs: ["[[ADR-T114-1 The ttf-parser crate as a direct dependency]]", "[[ADR-T114-2 SVG export is a pure writer over glyph outlines]]"]
feature: "[[PNG export]]"
tutorial: ""
tags: [task]
---

# T114 Export to SVG

## Goal

`Ctrl+Shift+E` writes the drawing as `~/draw-<date>-<time>.svg`, a vector image that stays sharp at any size and looks the same as on screen, formulas included, without needing draw's fonts.

## Known facts (2026-11-10)

- `Ctrl+E` exports the view to PNG by reading back the document layer ([[PNG export]], ADR-T92-2); `Ctrl+Shift+E` is unbound.
- Text and formulas are drawn from two embedded fonts (the monospaced text font and the Latin Modern Math subset, ADR-T33-1, ADR-T55-1) with fontdue; an SVG viewer will not have them.
- The proposal's risk: a second rendering path that must match the first.

## Spec

Owner decisions are marked (owner); the spec step settles the rest in an ADR.

- **AC-1** (owner) — `Ctrl+Shift+E` writes every shape of the document (not only the view) to `~/draw-<date>-<time>.svg`, with the canvas background; the log and a notice name the file. The write is atomic and off the frame.
- **AC-2** — Every shape kind is exported with its colour, width, fill, label and texts: strokes as smoothed paths, lines, arrows (with heads), rectangles, ellipses, grids (cells, fills, index bands), text blocks and shape texts.
- **AC-3** — Text and formulas are exported as glyph outlines (`<path>`s taken from the embedded fonts), each glyph defined once and reused, so the file looks the same in any viewer and needs no font.
- **AC-4** — A golden test renders sample documents to SVG and compares them with committed files; the SVG parses as XML and every number is finite. An empty document writes a valid empty SVG.
- **AC-5** — Fuzzed: any document exports without panicking.

Refined (ADR-T114-1, ADR-T114-2). Tests:

- AC-1: `svg::tests::to_svg_has_canvas_background_and_viewbox_of_all_shapes`; `keymap::tests::resolve_ctrl_shift_e_exports_svg`; `editor::tests::export_svg_*` (asks once, ignored while editing or in a gesture); `input_map::tests::svg_export_chord_repeat_is_dropped`; `export::tests::svg_*` (path `.svg`, never replaces a file).
- AC-2: `svg::tests::{line,arrow,rect,ellipse,grid,stroke,pressure_stroke,text}_…`.
- AC-3: `svg::tests::text_glyphs_are_defined_once_and_reused`, `formula_rule_is_a_line`; `glyphs::tests::outline_*`.
- AC-4: `tests/svg.rs` goldens in `tests/svg/`, `is_well_formed`, finite numbers, `empty_document_is_valid_svg`.
- AC-5: `fuzz::any_document_exports_to_svg`.

## Out of scope

- Exporting a selection only, choosing the file name, PDF.
- Selectable text in the SVG.

## Files owned

- a new `src/core/svg.rs`, `src/core/keymap.rs`, `src/core/command.rs`, `src/core/editor.rs`
- `src/shell/export.rs`, `src/shell/glyphs.rs` (glyph outlines)
- `docs/features/PNG export.md`, `docs/features/Contest cheat sheet.md`, `README.md`
- `tests/fuzz.rs`, `tests/` SVG goldens
- `src/shell/app.rs`, `src/shell/input_map.rs`, `src/shell/render.rs`, `Cargo.toml`, `Cargo.lock` (owner added after the batch blocked on them; `ttf-parser` 0.25 as a direct dependency is approved, it is already in `Cargo.lock` as 0.25.1 — record it in an ADR)
- `src/core/mod.rs` (one line, `pub mod svg;`: inherent to the new module)
- this note, and new `ADR-T114-<n>` notes

## Subtasks (one commit each)

- [x] spec — `docs(T114): specify SVG export`
- [x] tests — `test(T114): add failing tests for SVG export`
- [x] models — `feat(T114): write documents as SVG with glyph outlines`
- [x] behaviour — `feat(T114): export to SVG on Ctrl+Shift+E`
- [x] quality — `chore(T114): pass clippy and rustfmt`
- [x] docs — `docs(T114): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from K5 of the [[Feature proposal]].
- 2026-11-11 — BLOCKED (batch): the work needs files outside *Files owned*. `src/shell/app.rs` must take the new export request (it alone calls `take_export_request`, builds the document off the frame and writes the file); `src/shell/input_map.rs:470` must also ignore key repeat for the new chord (holding it would write many files); `Cargo.toml` and `Cargo.lock` are needed for ttf-parser 0.25 (the plan allows them, the note does not list them); `src/shell/render.rs` is needed to share the stroke polyline function (AC-2 recommended default). Nothing was implemented. Recommended fix: add `src/shell/app.rs`, `src/shell/input_map.rs`, `src/shell/render.rs`, `Cargo.toml`, `Cargo.lock` to *Files owned*, then re-run the task.
- 2026-11-11 — resumed after the owner added those files; merged `main` into the branch. Implemented per ADR-T114-1 and ADR-T114-2. Deviations: `src/core/mod.rs` got `pub mod svg;` (added to *Files owned*); `render.rs` is untouched because stored stroke points are already smoothed, so there is no function to share; the label size constants are copied into `core::svg` (the renderer's are in the shell); a pressure stroke is one round-capped segment per pair of points at the mean width; the "notice" is the log line (there is no on-screen notice); the quality step needed a commit (clippy: `format!` pushes, bool fields). Goldens are rewritten with `cargo test --test svg -- --ignored regenerate` because an env-var prefix is not an allowed command here.
- Manual checks left for the owner: export a drawing with a grid, arrows and formulas with `Ctrl+Shift+E`; open the file in a browser and compare with the screen (`xmllint --noout <file>` was not run: not an allowed command in the batch).
