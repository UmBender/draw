---
id: T16
title: Shape model v2 and helper skeleton
status: ready
wave: 7
branch: task/T16-helper-skeleton
depends_on: [T12, T13]
adrs: ["[[ADR-0004 Vector object model]]", "[[ADR-T11-1 Exact modifier matching]]"]
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

To be refined at the spec step; starting point:

- **AC-1** — Shape model: new `Shape::Grid { a, b, cols, rows, style }`
  (`cols`, `rows` in `1..=GRID_MAX_CELLS`, 64) and `label: Option<u32>` on
  `Rect` and `Ellipse`. `bounds`, `hit` (any grid line), `contains`,
  `translate`, `is_finite`, `with_fill` handle them; existing behaviour unchanged.
  *Tests:* `shape::tests::grid_*`, `shape::tests::label_*`.
- **AC-2** — Renderer: grids draw outer box + inner lines; labels draw centred
  inside their shape with a font size fitted to the box (hidden when too small
  on screen). Culling uses `bounds`. *Tests:* pure layout helpers in
  `render::tests` (`label_size_fits_box`, `grid_lines_count`).
- **AC-3** — Input: `Key::{ArrowLeft, ArrowRight, ArrowUp, ArrowDown}` added and
  mapped by `shell::input_map`. *Test:* `input_map::tests::arrows_map`.
- **AC-4** — Commands, editor state and keymap (new ADR, see below):
  `SetTool(Tool::Grid)` = `G`; `ToggleSmartSnap` = `M`; `ToggleGridSnap` =
  `Shift+G`; `ToggleNumbering` = `N`; `ResetNumbering` = `Shift+N`;
  `GridCols(±1)` = `→`/`←`, `GridRows(±1)` = `↓`/`↑` (clamped to
  `1..=GRID_MAX_CELLS`, live during a grid drag). The editor stores the flags,
  grid dimensions and the numbering counter and exposes read accessors;
  `ToolView` carries them to tools. *Tests:* `keymap::tests::*`,
  `editor::tests::toggle_*`, `editor::tests::grid_dims_clamped`.
- **AC-5** — Skeleton: `core::snap`, `core::numbering`, `core::tools::grid`
  exist with `//!` docs and no-op public entry points that T17–T19 fill;
  `Tool::ALL` includes `Grid` and the toolbar shows it plus snap/numbering
  toggles. *Tests:* `tests/skeleton.rs::all_modules_are_reachable`,
  `toolbar::tests::*`.
- **AC-6** — Fuzz: `tests/fuzz.rs` covers the new keys and checks the new
  invariants (grid dims in range, labels only on Rect/Ellipse).
  *Test:* existing fuzz tests, extended.

ADRs to write at the spec step: `ADR-T16-1 Grid shape and shape labels`
(amends ADR-0004), `ADR-T16-2 Helper key bindings` (builds on ADR-T11-1;
updates [[Keymap]]).

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

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 16 — requires steps 5, 11 and 13.

## Log
