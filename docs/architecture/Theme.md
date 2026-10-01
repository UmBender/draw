---
tags: [architecture]
status: accepted
adrs: ["[[ADR-0012 Fixed palette and theme tokens]]", "[[ADR-0015 Kanagawa Dragon theme]]"]
---

# Theme

Kanagawa Dragon, taken from the user's Alacritty theme
([[ADR-0015 Kanagawa Dragon theme]]). Implemented in `src/core/palette.rs` —
the only place colours are defined.

## UI tokens

| Token | Use | Value |
|-------|-----|-------|
| `bg` | Canvas background | `#181616` |
| `surface` | Toolbar background | `#0d0c0c` |
| `border` | Toolbar / button outlines | `#a6a69c` |
| `text` | Icons, labels | `#c5c9c5` |
| `accent` | Active tool, selection outline | `#7fa8bc` |
| `selection` | Selection / marquee fill | `#2d4f67` |

## Drawing palette (keys `1`–`6`)

| Key | Name | Value |
|-----|------|-------|
| `1` | ink | `#c5c9c5` |
| `2` | red | `#d16961` |
| `3` | green | `#8aa86e` |
| `4` | blue | `#7fa8bc` |
| `5` | yellow | `#ceb680` |
| `6` | magenta | `#aa88ac` |

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
