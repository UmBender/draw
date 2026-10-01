---
id: T16
title: Shape model v2 and helper skeleton
status: done
wave: 7
branch: task/T16-helper-skeleton
depends_on: [T12, T13]
adrs: ["[[ADR-0004 Vector object model]]", "[[ADR-T11-1 Exact modifier matching]]", "[[ADR-T16-1 Grid shape and shape labels]]", "[[ADR-T16-2 Helper key bindings]]", "[[ADR-T16-3 Helper settings and hooks]]"]
feature:
tutorial: "[[16 Growing a data model without breaking it]]"
tags: [task]
---

# T16 Shape model v2 and helper skeleton

## Goal

Every shared file that the helper features (T17 snapping, T18 grid tool,
T19 numbering) need is changed once, here, so those tasks can run in parallel
and each own only its own logic file — the same trick as [[T00 Bootstrap]].
After this task the new shapes render, hit-test, move, copy, undo and fuzz
correctly; the new tools and toggles exist but their logic files are stubs
that do nothing.

## Spec

Decisions: [[ADR-T16-1 Grid shape and shape labels]] (shape model),
[[ADR-T16-2 Helper key bindings]] (keys), [[ADR-T16-3 Helper settings and hooks]]
(how settings reach tools and the renderer).

- **AC-1** — Shape model: new `Shape::Grid { a, b, cols, rows, style }`
  (`cols`, `rows`: `u32`, valid range `1..=GRID_MAX_CELLS` = 64; uniform cells
  in the box `a`–`b`) and `label: Option<u32>` on `Rect` and `Ellipse`.
  `grid_lines(a, b, cols, rows)` yields the `cols + 1` vertical then `rows + 1`
  horizontal lines (dims clamped, so it is total). A grid is an open shape:
  `bounds` = box + half width, `hit` = within reach of any grid line,
  `contains` = false, `is_closed` = false, `fill` = `None`, `with_fill` is a
  no-op. `translate`/`is_finite` cover the corners. Labels survive
  `translate` and `with_fill`; `label()` reads and `with_label()` sets them
  (no-op on other shapes). Existing behaviour unchanged.
  *Tests (`shape::tests`):* `grid_bounds_include_half_width`,
  `grid_hit_on_inner_line`, `grid_hit_between_lines_is_false`,
  `grid_contains_nothing_and_is_open`, `grid_with_fill_is_noop`,
  `grid_translate_moves_corners`, `grid_is_finite_checks_corners`,
  `grid_lines_count_and_positions`, `grid_lines_clamps_dims`,
  `label_accessor_and_with_label`, `label_with_label_on_open_shape_is_noop`,
  `label_survives_translate_and_with_fill`; property
  `translate_moves_bounds_by_delta` extended to grids and labels.
- **AC-2** — Renderer: grids draw their lines; labels draw centred inside
  their shape (ellipses: inside the inscribed box, `rect / √2`) in a font size fitted to
  the box, hidden below `LABEL_MIN_PX` on screen and capped at
  `LABEL_MAX_PX`. Glyphs are rasterized at a few fixed sizes and scaled
  (`label_raster`) so zooming does not grow the glyph cache. Culling uses
  `bounds` (unchanged). Two no-op hooks for T17: `draw_underlay` (before
  shapes) and `draw_guides` (after the overlay), called from `shell::app`.
  *Tests (`render::tests`):* `label_size_fits_box`,
  `label_size_shrinks_with_more_digits`, `label_size_hidden_when_too_small`,
  `label_size_is_capped`, `label_size_non_finite_is_none`,
  `label_area_ellipse_is_inscribed_box`, `label_raster_quantizes`.
- **AC-3** — Input: `Key::{ArrowLeft, ArrowRight, ArrowUp, ArrowDown}` added
  and mapped by `shell::input_map`. *Test:* `input_map::tests::arrows_map`.
- **AC-4** — Commands, editor state and keymap ([[ADR-T16-2 Helper key bindings]]):
  `SetTool(Tool::Grid)` = `G`; `ToggleSmartSnap` = `M`; `ToggleGridSnap` =
  `Shift+G`; `ToggleNumbering` = `N`; `ResetNumbering` = `Shift+N`;
  `GridCols(±1)` = `→`/`←`, `GridRows(±1)` = `↓`/`↑`, clamped to
  `1..=GRID_MAX_CELLS`. None of these cancels the gesture, so dims change live
  during a grid drag. The editor stores a `Helpers` value (flags, grid dims
  default 4 × 4, numbering counter starting at `FIRST_NUMBER` = 1) inside
  `DrawStyle`, so `ToolCtx`/`ToolView` carry it unchanged
  ([[ADR-T16-3 Helper settings and hooks]]); `Editor::helpers()` reads it.
  *Tests:* `keymap::tests::every_documented_binding` (table extended),
  `keymap::tests::helper_bindings`, `keymap::tests::shift_chords_are_exact`,
  `editor::tests::helpers_default_off_and_4x4`,
  `editor::tests::toggle_smart_snap_flips`, `toggle_grid_snap_flips`,
  `toggle_numbering_flips`, `reset_numbering_restarts_at_one`,
  `grid_dims_step_by_one`, `grid_dims_clamped`,
  `grid_dims_change_keeps_gesture`, `command::tests::tool_all_lists_each_tool_once`.
- **AC-5** — Skeleton: `core::snap` (`snap_end`, identity), `core::numbering`
  (`label_new`, identity; `FIRST_NUMBER`), `core::tools::grid` (`State`,
  `on_pointer`/`preview`/`cancel` doing nothing) exist with `//!` docs;
  `Overlay` gains `guides` (world segments, empty until T17); `Tool::ALL` and
  the toolbar include `Grid`, and the toolbar has a helper group with
  smart-snap, grid-snap and numbering toggles.
  *Tests:* `tests/skeleton.rs::all_modules_are_reachable`,
  `tools::tests::overlay_with_guides_is_not_empty`,
  `toolbar::tests::layout_has_every_button_in_order`,
  `layout_buttons_do_not_overlap`, `hit_each_button_returns_its_command`,
  `layout_fits_default_window`.
- **AC-6** — Fuzz: `tests/fuzz.rs` generates the arrow keys, adds grid-drag
  sessions with arrow keys pressed mid-drag, and checks the new invariants:
  grid dims of shapes and of `Editor::helpers()` in `1..=GRID_MAX_CELLS`,
  labels `≥ FIRST_NUMBER` (labels exist only on Rect/Ellipse by
  construction), numbering counter `≥ FIRST_NUMBER`.
  *Tests:* existing fuzz tests, extended; `strategies_generate_all_variants`.

## Out of scope

The logic of snapping (T17), grid drawing (T18) and numbering (T19).

## Files owned

`src/core/shape.rs`, `src/core/command.rs`, `src/core/input.rs`,
`src/core/keymap.rs`, `src/core/editor.rs`, `src/core/mod.rs`,
`src/core/tools/mod.rs`, `src/core/tools/shape_tool.rs` (only the new
`Tool::Grid` match arm), new `src/core/snap.rs`, `src/core/numbering.rs`,
`src/core/tools/grid.rs`, `src/shell/render.rs`, `src/shell/input_map.rs`,
`src/shell/toolbar.rs`, `tests/skeleton.rs`, `tests/fuzz.rs`,
`docs/architecture/Keymap.md`, `docs/architecture/Architecture.md`,
new `ADR-T16-*`.

Amended at the spec step (approved by the owner; all mechanical, needed for
the new fields and variants to compile or for T17 to have its call sites):
`src/core/clipboard.rs` (testkit literals get `label: None`),
`src/core/tools/shape_tool.rs` (also `label: None` in `Drag::shape` and a
`Grid` arm in the test helper `ends`), `src/core/tools/select.rs` (its
`Overlay` literal uses `..Overlay::default()`), `src/shell/app.rs` (calls the
two render hooks).

## Subtasks (one commit each)

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs

## Learning path

Step 16 — requires steps 5, 11 and 13.

## Log

- 2026-10-01 — Spec: adding the `label` field, the `Grid` variant and the
  T17 guide/underlay plumbing needed edits outside the planned files owned
  (clipboard testkit, `shape_tool.rs` beyond the Grid arm, one `select.rs`
  literal, two calls in `app.rs`). Approved by the owner and listed under
  *Files owned*. Helper settings ride in `DrawStyle` so no `ToolCtx`/
  `ToolView` fixture (pen, clipboard, shape tool) had to change
  ([[ADR-T16-3 Helper settings and hooks]]).
- `grid_lines_count` moved from `render::tests` to
  `shape::tests::grid_lines_count_and_positions`: the line geometry lives in
  `core::shape` (shared by hit-testing and drawing), the renderer only maps
  it. `label_area_ellipse_is_inscribed_square` was renamed
  `…_inscribed_box`: for a non-circle the largest inscribed axis-aligned box
  is `rect / √2`, not a square.
- Behaviour: one label-size test case was wrong (a 40 px wide area fits a
  5-digit label at 10.7 px); changed to 25 px. Fuzz grid drags are weighted 1
  of 14 chunks: at weight 2 the still-inert grid tool, which stays selected,
  pushed "sessions with shapes" to 94/200 (< 100). T18 makes grid drags
  create shapes, which raises it again.
- Quality: `Editor::apply` exceeded clippy's 100-line limit; helper
  commands moved to `apply_helper`. Toolbar tool and helper buttons share
  `draw_toggle`.
- Not checked on screen: label legibility and grid rendering need a manual
  look once T18/T19 can create grids and labels (nothing creates them yet).
