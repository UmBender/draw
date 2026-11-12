---
title: Themes
task: "[[T51 Build-time themes]]"
adrs: ["[[ADR-T51-1 One theme file with every colour]]", "[[ADR-0012 Fixed palette and theme tokens]]", "[[ADR-0015 Kanagawa Dragon theme]]"]
tutorial: "[[51 Keeping every colour in one file]]"
shortcuts: []
tags: [feature]
---

# Themes

## What it does

Every colour draw shows — canvas, drawing colours, selection, marquee,
snapping guides, dot grid, formula warnings and toolbar — is a constant in
one file, `src/theme.rs`. To restyle draw (for example to match the rest of
a riced desktop) you edit that file and rebuild. One theme ships: Kanagawa
Dragon ([[ADR-0015 Kanagawa Dragon theme]]).

There is deliberately **no** command-line option, environment variable,
Cargo feature or configuration file for colours.

## How to use

| Step | What to do |
|------|------------|
| 1 | Open `src/theme.rs`. |
| 2 | Change a value: `#RRGGBB` from any palette is written `0xRRGGBB`. An `_ALPHA` is the opacity of the colour above it (`0` invisible, `255` opaque). |
| 3 | Run `cargo test theme`. The sanity tests say if two drawing colours are the same or if `INK` / `SELECTION_OUTLINE` are too close to `CANVAS` (below 3:1 contrast). |
| 4 | Update `theme_matches_kanagawa_dragon` (it pins the shipped values; it is the one test meant to change). |
| 5 | Rebuild: `cargo build --release`, or `scripts/package.sh` for the packages. |

### Groups

| Group | Constants | What they colour |
|-------|-----------|------------------|
| Canvas | `CANVAS`, `TEXT_ON_FILL` | Background; numbers, text and caret on a filled shape |
| Drawing colours | `INK`, `RED`, `GREEN`, `BLUE`, `YELLOW`, `MAGENTA` | Keys `1`–`6`, in that order |
| Selection and marquee | `SELECTION_OUTLINE`, `MARQUEE_OUTLINE`, `MARQUEE_FILL`, `MARQUEE_FILL_ALPHA` | Selection / edit box outline, the drag-to-select box |
| Snapping | `GUIDE`, `DOT_GRID`, `DOT_GRID_ALPHA` | Alignment guides, snap grid dots |
| Formulas | `FORMULA_SOURCE`, `FORMULA_SOURCE_ALT` | Formula source that does not parse |
| Toolbar | `TOOLBAR_BACKGROUND`, `TOOLBAR_BORDER`, `TOOLBAR_TEXT`, `TOOLBAR_ACTIVE`, `TOOLBAR_ACTIVE_FILL`, `TOOLBAR_DISABLED_ALPHA` | Letters, active tool, disabled undo/redo, active fill |
| Menu islands | `MENU_ISLAND` (+ `_ALPHA`), `MENU_PANEL_ALPHA`, `MENU_BORDER` (+ `_ALPHA`), `MENU_KEY_FILL`, `MENU_MUTED`, `MENU_DIM_TEXT`, `MENU_ACTIVE_BRIGHT` | The rounded islands of the strip, the palette and the sheet, their ring, key caps, quiet and bright text ([[ADR-T126-1 Menus drawn as translucent rounded islands]]) |

### Worked example: a lighter background, darker ink

```rust
pub const CANVAS: u32 = 0xf2ecbc;       // was 0x181616
pub const TEXT_ON_FILL: u32 = 0xf2ecbc; // text on fills follows the canvas
pub const INK: u32 = 0x1f1f28;          // was 0xc5c9c5
```

A light background needs a dark ink: with the old ink, `cargo test theme`
fails `ink_contrasts_with_canvas` and prints the ratio. Check the other
five drawing colours and `SELECTION_OUTLINE` the same way.

## How it works

`src/theme.rs` has only `pub const` items. `core::palette::PALETTE` reads
the six drawing constants in key order, so shapes (which store a
`ColorId`) recolour with the theme. The shell turns constants into
macroquad colours with `render::hex_color` / `hex_color_alpha`; derived
colours keep their rule but take their inputs from the theme
(`render::label_color`, `render::warn_color`, the faded undo/redo arrow).
A test, `theme::tests::guard_finds_no_colour_literal_outside_theme`, scans
`src/` and fails on any hex colour, colour constructor with numbers,
colour struct literal, alpha edit or macroquad named colour outside the
theme file ([[ADR-T51-1 One theme file with every colour]]).

## Limits

- One theme per build; no switching at run time.
- Six drawing colours, fixed count and order ([[ADR-0012 Fixed palette and theme tokens]]).
- The contrast check covers `INK` and `SELECTION_OUTLINE` only; other
  colours are the tweaker's call.
