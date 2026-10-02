---
id: T22
title: Numbering drives grid indices
status: review
wave: 10
branch: task/T22-numbered-grids
depends_on: [T18, T19, T21]
adrs: ["[[ADR-T18-3 Grid axis indices]]", "[[ADR-T19-1 Numbering counter and undo]]", "[[ADR-T22-1 Numbering toggle drives grid indices]]"]
feature: "[[Numbered nodes]]"
tutorial: "[[22 One toggle, two meanings]]"
tags: [task]
---

# T22 Numbering drives grid indices

## Goal

One numbering switch for everything: the `N` key / toolbar `N` button that
numbers circles and squares also turns on the axis indices of new grids.
A numbered grid does not use up a number: circle 5, a numbered grid, then
square 6. The separate `I` key and the flyout's `axes` row go away.

Requested by the owner on 2026-10-01. Branched from `task/T21-cell-fill`
(T18 and T21 are in review, not merged yet).

## Spec

Decision: [[ADR-T22-1 Numbering toggle drives grid indices]].

- **AC-1** — New grids (and the grid preview) get `axes = true` exactly
  when `Helpers::numbering` is on. *Tests:*
  `grid::tests::numbering_turns_on_axes` (replaces
  `axes_setting_reaches_grid`), `grid::tests::axes_off_by_default`.
- **AC-2** — A numbered grid leaves the counter alone, also on undo and
  redo. *Tests:* `editor::tests::numbered_grid_keeps_counter`,
  `editor::tests::undo_numbered_grid_keeps_counter`.
- **AC-3** — The separate setting is gone: no `Helpers::grid_axes`, no
  `Command::ToggleGridAxes`, `I` is unbound, the flyout has only the
  `cols`/`rows` rows. *Tests:* `keymap::tests::every_documented_binding`
  (row removed), `keymap::tests::i_is_unbound`,
  `toolbar::tests::flyout_has_four_buttons_in_order` (replaces the
  five-button test), `flyout_buttons_return_their_commands`,
  `flyout_buttons_inside_panel_and_disjoint`;
  `editor::tests::toggle_grid_axes_flips` removed.

## Out of scope

Numbering inside cells, any other index scheme, changing a committed
grid's indices.

## Files owned

`src/core/tools/grid.rs`, `src/core/editor.rs`, `src/core/command.rs`,
`src/core/keymap.rs`, `src/shell/toolbar.rs`,
`docs/architecture/Keymap.md`, `docs/features/Grid tool.md`,
`docs/features/Numbered nodes.md`, new `ADR-T22-*` and tutorial note.

## Subtasks (one commit each)

- [x] spec · [x] tests · [x] models · [x] behaviour · [x] quality · [x] docs

## Learning path

Step 22 — requires steps 18 and 19.

## Log

- 2026-10-01 — spec: owner asked for the numbering button to drive grid
  indices too, without the grid consuming a number. The counter rule of
  ADR-T19-1 already counts only labelled shapes, so AC-2 needs tests, not
  code.
- tests: red as a compile error (the flyout test match no longer lists
  `GridAxes`) plus two behaviour tests.
- models: removals only; `axes: false` until the behaviour step.
- behaviour: one line, `axes: helpers.numbering`.
- quality: `scripts/check.sh` green with no changes, so no quality commit.
- docs: Keymap, Grid tool and Numbered nodes notes, tutorial 22.
  `docs/tutorials/18 A grid tool with live parameters.md` (owned by T18)
  still mentions the `I` key; left for the integrator or a T18 follow-up.
