---
id: T122
title: ASCII text in the palette and the sheet
status: in-progress
wave: 60
branch: fix/ui-text-ascii
depends_on: []
adrs: []
feature: "[[Shortcuts and macros]]"
tutorial: ""
tags: [task, bug]
---

# T122 ASCII text in the palette and the sheet

## Goal

Every character the command palette, the shortcut sheet and the toolbar draw exists in the font they use, so no key or word shows as an empty box.

## Known facts (2026-11-12)

- Reported by the owner from the command palette: the keys `Alt+↓` and `→` (grid columns, align edges) show as boxes.
- Those overlays draw with macroquad's built-in font through `draw_text`, which has no arrows. The canvas text uses the bundled subset font instead.
- `chord_label` writes the arrow keys as `←` `→` `↑` `↓`; the sheet footer says `←/→ page`; one sheet row says `45°`.

## Spec

- **AC-1** — The label of every key binding is ASCII: arrow keys read `Left`, `Right`, `Up`, `Down`.
  Tests: `chord_label_spells_modifiers_and_keys` (updated), `every_binding_label_and_description_is_ascii`.
- **AC-2** — Every text the sheet shows is ASCII: its rows, its footer (`Left/Right page`) and the `45°` row (`45-degree`).
  Tests: `every_sheet_row_is_ascii`, `footer_is_ascii`.

## Out of scope

- Bundling a font for the overlays.

## Files owned

- `src/core/command_palette.rs`, `src/core/sheet.rs`, `src/shell/sheet.rs`
- `docs/features/Shortcuts and macros.md` (only if it quotes the arrows)
- this note

## Subtasks (one commit each)

- [ ] spec — `docs(T122): specify ASCII overlay text`
- [ ] regression — `test(T122): add regression test for missing glyphs in the overlays`
- [ ] behaviour — `fix(T122): write the overlay text in ASCII`

## Log

- 2026-11-12 — written from the owner's bug report. The bug shows only in the running app (a missing glyph), so the regression test checks the text itself instead of a snapshot.
