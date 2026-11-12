---
id: T126
title: Menu islands
status: done
wave: 62
branch: task/T126-menu-islands
depends_on: [T123, T125]
adrs: ["[[ADR-T126-1 Menus drawn as translucent rounded islands]]"]
feature: "[[Theme and palette]]"
tutorial: ""
tags: [task]
---

# T126 Menu islands

## Goal

The tool strip, the command palette and the shortcut sheet look like the owner's waybar: translucent rounded islands with a thin border, a blue fill behind the active item, and small key caps. The tool icons stay the letters of their keys, the palette of the theme stays, and the colours still come from `src/theme.rs` only. Everything the menus show and every key stays as it is.

## Known facts (2026-11-12)

- Direction A of the proposals page, chosen by the owner after reviewing three. The waybar values it follows: `border-radius: 8px`, background `bg` at 75 % opacity, a 1 px border `surface1` at 80 %, active in the accent colour.
- macroquad has no rounded rectangles; they are built from triangles that never overlap, so a translucent fill does not show seams.
- The strip kept to the default 1280×800 window: four islands fit (788 px), the pages island only when the window is taller.
- Menu text keeps the bundled font of [[T123 Menu font]]; the owner has not asked for another one.

## Spec

- **AC-1** — `shell::ui_shapes` builds a rounded rectangle and a rounded ring as triangles: finite, inside the rectangle, never overlapping, and with the area of the shape they stand for (a rounded rectangle `w·h − (4 − π)·r²`, a ring the difference of two). A radius larger than half the shorter side is clamped; a radius of 0 gives the plain rectangle.
  Tests: `rounded_fill_covers_the_rounded_rectangle`, `rounded_fill_never_overlaps_itself`, `rounded_ring_covers_the_border`, `radius_is_clamped_and_zero_is_a_rectangle`, `degenerate_rectangles_give_no_triangles`.
- **AC-2** — The theme gains the menu tokens (island fill and opacity, border and its opacity, key cap fill, muted and dim text, bright accent) with the Kanagawa Dragon values; no colour is written outside `theme.rs`, and the contrast checks cover the new text tokens.
  Tests: `theme_matches_kanagawa_dragon` (extended), `menu_text_contrasts_with_canvas`, the colour guard unchanged.
- **AC-3** — The tool strip is four islands (tools, colours, helpers, undo and redo) and a fifth for the page buttons when they fit; the buttons are 32 px squares inside them. The four islands, the strip and the `pen` badge fit a 1280×800 window. A click between the islands inside the strip is still swallowed.
  Tests: `islands_frame_their_groups`, `islands_fit_the_default_window`, `islands_do_not_overlap`, `page_island_frames_the_page_buttons`, and the existing layout and routing tests adapted to the new geometry.
- **AC-4** — The palette and the sheet are islands too: rounded rows, the selected row in the active fill with the bright accent, each key of a chord as its own key cap, the sheet topics as pills. Their layouts and the keys they handle do not change.
  Tests: `key_caps_split_a_chord`, `key_caps_fit_inside_the_row`, the existing palette and sheet layout tests, and the e2e cases `menus-palette` and `menus-sheet` (goldens `expected.ppm`).

## Out of scope

- A different font; the toolbar's behaviour; the minimap and canvas overlays; light themes.

## Files owned

- `src/theme.rs`, `src/shell/ui_shapes.rs`, `src/shell/mod.rs`
- `src/shell/toolbar.rs`, `src/shell/command_palette.rs`, `src/shell/sheet.rs`
- `docs/features/Theme and palette.md`, `docs/features/Shortcuts and macros.md`
- `tests/e2e/*/expected.ppm` (the goldens that show the toolbar)
- this note and `ADR-T126-1`

## Subtasks

- [x] spec — `docs(T126): specify the menu islands`
- [x] tests and code — `feat(T126): draw the menus as rounded islands`
- [x] goldens and docs — `docs(T126): update the goldens and the notes`

## Log

- 2026-11-12 — written from the owner's choice of proposal A.
- 2026-11-12 — done; `scripts/check.sh` green and `scripts/e2e.sh` green with new goldens. The goldens were looked at by eye: islands, active fill, ringed swatch, key caps and topic pills match the proposal. The pages island shows only in a window taller than about 1000 px, so no golden shows it.
