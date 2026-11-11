---
id: T109
title: Shape recognition
status: in-progress
wave: 59
branch: task/T109-shape-recognition
depends_on: []
adrs: ["[[ADR-T109-1 Shape recognition on Z]]"]
feature: "[[Drawing tools]]"
tutorial: ""
tags: [task]
---

# T109 Shape recognition

## Goal

Right after drawing a pen stroke, pressing `Z` with the left hand replaces it with the best-fitting straight line, rectangle or ellipse, in one undo step.

## Known facts (2026-11-10)

- Plain `Z` is unbound (`Ctrl+Z` is undo). The proposal had `Q`, now the laser pointer (T106); the owner chose `Z`.
- Pen strokes are already smoothed by anti-tremor ([[Anti-tremor strokes]]); the line, rectangle and ellipse tools exist ([[Drawing tools]]).
- The proposal's risk: a wrong guess costs more than redrawing, so a stroke that fits nothing well must stay as it is.

## Spec

Owner decisions are marked (owner). Rules and thresholds: [[ADR-T109-1 Shape recognition on Z]].

- **AC-1** (owner) — `Z` acts on the last pen stroke drawn, as long as nothing else was done since (a new stroke, an edit, an undo); otherwise it does nothing.
  - `keymap::tests::z_recognizes_the_last_stroke`, `keymap::tests::every_documented_binding`
  - `editor::tests::z_after_a_pen_stroke_replaces_it_with_its_shape`
  - `editor::tests::z_after_another_edit_does_nothing`
  - `editor::tests::z_after_undo_does_nothing`
  - `editor::tests::z_during_a_gesture_does_nothing`
  - `editor::tests::z_in_text_edit_types_z`
- **AC-2** — The stroke becomes a `Line` when it is nearly straight, a `Rect` when it is a closed four-cornered loop, an `Ellipse` when it is a closed round loop, with the stroke's colour and width. Thresholds are set in the ADR and tested on drawn samples (shaky circles, squares, lines, and scribbles that must stay strokes).
  - `recognize::tests::fit_shaky_lines_gives_lines`
  - `recognize::tests::fit_shaky_squares_and_rectangles_gives_rects`
  - `recognize::tests::fit_shaky_circles_and_ellipses_gives_ellipses`
  - `recognize::tests::fit_scribbles_gives_nothing`
  - `recognize::tests::recognize_keeps_colour_and_width`
  - `recognize::tests::recognize_ignores_non_strokes`
- **AC-3** — One undo step: undo brings the original stroke back. No fit good enough: nothing changes and no undo step is recorded.
  - `editor::tests::z_is_one_undo_step_and_undo_restores_the_stroke`
  - `editor::tests::z_on_a_scribble_changes_nothing_and_records_no_step`
  - `editor::tests::z_twice_acts_once`
- **AC-4** — Rotated rectangles and ellipses fit their axis-aligned box (draw has no rotation); an open curve is never closed into a shape.
  - `recognize::tests::fit_rotated_rect_gives_its_axis_aligned_box`
  - `recognize::tests::fit_rotated_ellipse_gives_its_axis_aligned_box`
  - `recognize::tests::fit_open_curves_never_close`
- **AC-5** — Fuzzed: any stroke, including degenerate ones (one point, all points equal, huge coordinates), never panics and gives finite shapes.
  - `recognize::tests::fit_degenerate_strokes_gives_nothing`
  - `recognize::tests::fit_huge_coordinates_stays_finite`
  - `recognize::tests::fit_is_total_and_finite` (proptest)
  - `fuzz::shape_session_circle_becomes_ellipse`, `fuzz::shape_recognition_sessions_keep_invariants` (proptest), and `Z` in `arb_command_burst` through `BINDINGS`

## Out of scope

- Triangles, arrows, polygons; recognising while drawing without a key.
- Recognising text or formulas.

## Files owned

- a new `src/core/recognize.rs`, `src/core/editor.rs`, `src/core/keymap.rs`, `src/core/command.rs`, `src/core/mod.rs`
- `docs/features/Drawing tools.md`, `docs/features/Contest cheat sheet.md`, `docs/architecture/Keymap.md`, `README.md`
- `tests/fuzz.rs`
- this note, and new `ADR-T109-<n>` notes

## Subtasks (one commit each)

- [ ] spec — `docs(T109): specify shape recognition`
- [ ] tests — `test(T109): add failing tests for shape recognition`
- [ ] models — `feat(T109): fit lines, rectangles and ellipses to a stroke`
- [ ] behaviour — `feat(T109): replace the last stroke with its shape on Z`
- [ ] quality — `chore(T109): pass clippy and rustfmt`
- [ ] docs — `docs(T109): update the feature note`

## Learning path

None: it extends an existing area.

## Log

- 2026-11-10 — note written with the owner, from I2 of the [[Feature proposal]].
- Key changed from `Q` to `Z`: `Q` is the laser pointer (T106).
- 2026-11-10 — UNATTENDED CHOICE: added `src/core/mod.rs` (declares the new `recognize` module) and `docs/architecture/Keymap.md` (the documented key table, as T100 did) to *Files owned*.
- 2026-11-10 — UNATTENDED CHOICE: "nothing else since" is read from the history (ADR-T109-1), so a redo that brings the stroke back re-enables `Z`, and so does a paste or duplicate of exactly one stroke; no snapshot format change.
