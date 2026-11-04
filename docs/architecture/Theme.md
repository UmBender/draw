---
tags: [architecture]
status: accepted
adrs: ["[[ADR-0012 Fixed palette and theme tokens]]", "[[ADR-0015 Kanagawa Dragon theme]]", "[[ADR-T51-1 One theme file with every colour]]"]
---

# Theme

Kanagawa Dragon, taken from the user's Alacritty theme
([[ADR-0015 Kanagawa Dragon theme]]). Every colour is a constant in
`src/theme.rs`, the only file that defines colours
([[ADR-T51-1 One theme file with every colour]]); a guard test fails on a
colour literal anywhere else in `src/`. See [[Themes]] for how to change it.

## UI colours

| Constant | Use | Value |
|----------|-----|-------|
| `CANVAS` | Canvas background | `#181616` |
| `TEXT_ON_FILL` | Numbers, text and caret on a filled shape | `#181616` |
| `SELECTION_OUTLINE` | Selection and edit box outline | `#7fa8bc` |
| `MARQUEE_OUTLINE` | Marquee outline | `#7fa8bc` |
| `MARQUEE_FILL` / `_ALPHA` | Marquee fill | `#2d4f67`, 63 |
| `GUIDE` | Alignment guides | `#7fa8bc` |
| `DOT_GRID` / `_ALPHA` | Snap grid dots | `#a6a69c`, 89 |
| `FORMULA_SOURCE` | Formula source that does not parse | `#d16961` |
| `FORMULA_SOURCE_ALT` | The same, on text drawn in `FORMULA_SOURCE` | `#ceb680` |
| `TOOLBAR_BACKGROUND` | Toolbar and flyout background | `#0d0c0c` |
| `TOOLBAR_BORDER` | Toolbar edge, flyout buttons | `#a6a69c` |
| `TOOLBAR_TEXT` | Letters, labels, undo/redo | `#c5c9c5` |
| `TOOLBAR_ACTIVE` | Active tool, current colour, pen badge | `#7fa8bc` |
| `TOOLBAR_ACTIVE_FILL` | Fill behind an active toggle | `#2d4f67` |
| `TOOLBAR_DISABLED_ALPHA` | Opacity of unavailable undo/redo | 76 |

## Drawing palette (keys `1`–`6`)

| Key | Constant | Value |
|-----|----------|-------|
| `1` | `INK` | `#c5c9c5` |
| `2` | `RED` | `#d16961` |
| `3` | `GREEN` | `#8aa86e` |
| `4` | `BLUE` | `#7fa8bc` |
| `5` | `YELLOW` | `#ceb680` |
| `6` | `MAGENTA` | `#aa88ac` |

## Source palette (reference)

The original `theme.toml` was removed from the repo; its full contents are kept
here so future theme decisions can draw from it.

| Group | black | red | green | yellow | blue | magenta | cyan | white |
|-------|-------|-----|-------|--------|------|---------|------|-------|
| normal | `#0d0c0c` | `#d16961` | `#8aa86e` | `#ceb680` | `#7fa8bc` | `#aa88ac` | `#82b0ab` | `#d0c58b` |
| bright | `#a6a69c` | `#ec6070` | `#7bb57b` | `#ecc57e` | `#75b8d3` | `#8e7eb5` | `#6db5a7` | `#c5c9c5` |

| Other | Value |
|-------|-------|
| primary background | `#181616` |
| primary foreground | `#c5c9c5` |
| selection background | `#2d4f67` |
| selection foreground | `#c8c093` |
| indexed 16 | `#c28f6f` |
| indexed 17 | `#c4886f` |
