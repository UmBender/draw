---
id: T123
title: Menu font
status: done
wave: 60
branch: task/T123-menu-font
depends_on: [T122]
adrs: ["[[ADR-T123-1 Menus use the bundled font on whole pixels]]"]
feature: "[[Shortcuts and macros]]"
tutorial: ""
tags: [task]
---

# T123 Menu font

## Goal

The toolbar, the command palette, the shortcut sheet and the page indicator are drawn in the bundled text font and sit on whole pixels, so they look clean and do not shimmer.

## Known facts (2026-11-12)

- Reported by the owner: the menu fonts are ugly and look shaky.
- Those overlays used macroquad's built-in font through `draw_text`, at fractional positions.
- The bundled Adwaita Mono subset already covers printable ASCII, the arrows and the degree sign.

## Spec

- **AC-1** — One module, `shell::ui_text`, draws and measures every menu string with the bundled font, and falls back to the built-in font if it does not load.
  Tests: `menu_font_has_every_printable_ascii_character`.
- **AC-2** — Every string is placed on a whole physical pixel for the window's scale.
  Tests: `snap_rounds_to_physical_pixels`, `snap_leaves_odd_input_alone`.
- **AC-3** — The toolbar, palette, sheet and page indicator use it; nothing else in `src/shell` calls `draw_text` or `measure_text` for menus.

## Out of scope

- The canvas text (its own atlas) and a font setting.

## Files owned

- `src/shell/ui_text.rs`, `src/shell/mod.rs`, `src/shell/toolbar.rs`, `src/shell/command_palette.rs`, `src/shell/sheet.rs`, `src/shell/render.rs`
- this note and `ADR-T123-1`

## Subtasks

- [x] code and tests — `feat(T123): draw the menus in the bundled font`

## Log

- 2026-11-12 — done after the owner's report; `scripts/check.sh` green. Not checked by eye in the running app: the owner judges the look.
