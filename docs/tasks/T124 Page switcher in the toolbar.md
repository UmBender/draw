---
id: T124
title: Page switcher in the toolbar
status: in-progress
wave: 60
branch: task/T124-page-switcher
depends_on: [T116, T123]
adrs: ["[[ADR-T124-1 Ten pages and a switcher under undo and redo]]"]
feature: "[[Infinite canvas, pan and zoom]]"
tutorial: ""
tags: [task]
---

# T124 Page switcher in the toolbar

## Goal

Under the undo and redo buttons there is a small switcher, like the workspaces of a status bar: the numbers 1 to 10, one look for the current page, one for a page with a drawing and one for an empty page. A click goes to that page. The number that appeared top-right on a page switch is gone.

## Known facts (2026-11-12)

- Requested by the owner. The pages of [[T116 Pages]] were nine (`Alt+1`…`Alt+9`); the owner asked for 1 to 10.
- The strip is 40 px wide and the default 1280×800 window has about 20 px left under the buttons, so the switcher is a compact grid of two columns by five rows and it needs a window of about 890 px to show whole. A window that is shorter shows the cells that fit.

## Spec

- **AC-1** — There are ten pages; `Alt+0` goes to page 10. A saved file or snapshot with up to ten pages loads, and files with nine or fewer pages still load.
  Tests: `alt_digits_go_to_pages`, `keymap_binds_alt_zero_to_page_ten`, `going_to_page_ten_works`, `page_count_is_ten`.
- **AC-2** — `Editor::page_has_shapes(page)` is true for the current page when it holds a shape and for another page when its stored document does; false otherwise and for an out-of-range page.
  Tests: `page_has_shapes_follows_the_documents`.
- **AC-3** — `toolbar::page_layout` lists the cells 1…10, row by row in two columns, below the history buttons and disjoint from every other button; cells that do not fit the window are left out. A click on a cell runs `Command::GoToPage`.
  Tests: `page_cells_are_two_columns_below_the_history_buttons`, `page_cells_do_not_overlap_buttons`, `page_cells_that_do_not_fit_are_left_out`, `page_cell_click_goes_to_its_page`.
- **AC-4** — The current page is drawn filled, a page with shapes in full colour with a bar under its number, an empty page dimmed.
  Test: `page_cell_look_follows_the_editor` (the choice of look, not pixels).
- **AC-5** — The page indicator top-right is removed.
  Tests: `page_label` and its tests are deleted.

## Out of scope

- Dragging shapes between pages; renaming pages; a horizontal bar.

## Files owned

- `src/core/editor.rs`, `src/core/keymap.rs`
- `src/shell/toolbar.rs`, `src/shell/render.rs`, `src/shell/app.rs`
- `README.md`, `docs/features/Infinite canvas, pan and zoom.md`, `docs/features/Contest cheat sheet.md`
- this note and `ADR-T124-1`

## Subtasks (one commit each)

- [ ] spec — `docs(T124): specify the page switcher`
- [ ] behaviour — `feat(T124): add the page switcher and the tenth page` (tests with the code)
- [ ] docs — `docs(T124): update the feature notes`

## Log

- 2026-11-12 — written from the owner's request.
