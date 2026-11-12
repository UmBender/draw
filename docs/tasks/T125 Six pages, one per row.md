---
id: T125
title: Six pages, one per row
status: done
wave: 60
branch: fix/six-pages
depends_on: [T124]
adrs: ["[[ADR-T125-1 Six pages, one button per row]]"]
feature: "[[Infinite canvas, pan and zoom]]"
tutorial: ""
tags: [task]
---

# T125 Six pages, one per row

## Goal

There are six pages, `Alt+1`…`Alt+6`, and the switcher under undo and redo is one square button per row, like the colours.

## Known facts (2026-11-12)

- Asked by the owner while the 1.6.0 release was starting: at most 1–6, one per line, like the palette colours.
- Changes [[T124 Page switcher in the toolbar]] (ten pages, two columns of small cells), which was not released.

## Spec

- **AC-1** — `PAGE_COUNT` is 6; `Alt+7`…`Alt+9` and `Alt+0` are unbound.
  Tests: `page_count_is_six`, `going_to_the_last_page_works`, `alt_digits_go_to_pages`.
- **AC-2** — `toolbar::page_layout` lists six 32 px buttons, one per row, in the strip's column, under the history buttons after a group gap; buttons that do not fit the window are left out.
  Tests: `page_cells_are_one_column_below_the_history_buttons`, `page_cells_do_not_overlap_buttons`, `page_cells_that_do_not_fit_are_left_out`, `page_cell_click_goes_to_its_page`.

## Files owned

- `src/core/editor.rs`, `src/core/keymap.rs`, `src/shell/toolbar.rs`
- `README.md`, `CHANGELOG.md`, `docs/features/Infinite canvas, pan and zoom.md`, `docs/features/Contest cheat sheet.md`
- this note and `ADR-T125-1`

## Log

- 2026-11-12 — done; `scripts/check.sh` green. The six buttons need a window about 1000 px tall.
