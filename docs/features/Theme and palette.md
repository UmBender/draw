---
title: Theme and palette
task: "[[T02 Palette and theme tokens]]"
adrs: ["[[ADR-0012 Fixed palette and theme tokens]]", "[[ADR-0015 Kanagawa Dragon theme]]"]
tutorial: "[[02 Palettes and design tokens]]"
shortcuts: ["1", "2", "3", "4", "5", "6"]
tags: [feature]
---

# Theme and palette

## What it does

`draw` uses six fixed drawing colours and a dark Kanagawa Dragon UI theme.
There is no colour picker: number keys `1`–`6` choose a colour, and every
colour in the app comes from one file, so the look is consistent and a theme
change recolours existing drawings too.

## How to use

| Action | Shortcut / gesture |
|--------|--------------------|
| Ink (default, `#c5c9c5`) | `1` |
| Red (`#d16961`) | `2` |
| Green (`#8aa86e`) | `3` |
| Blue (`#7fa8bc`) | `4` |
| Yellow (`#ceb680`) | `5` |
| Magenta (`#aa88ac`) | `6` |

The key binding itself is wired by [[T11 Keymap and macros]] (see [[Keymap]]);
this feature provides the mapping `ColorId::from_key_digit`.

## How it works

`src/core/palette.rs` is the only place colours are defined
([[ADR-0012 Fixed palette and theme tokens]]):

- `Rgba` — 8-bit RGBA, built at compile time with `Rgba::from_hex(0xRRGGBB)`;
  `to_f32()` gives the `[0, 1]` floats the renderer needs.
- `ColorId` — a validated palette index (`0..PALETTE_LEN`). Shapes store this,
  not an RGBA value, so they stay small and follow theme changes.
  `ColorId::new` and `ColorId::from_key_digit` return `None` for anything out
  of range, which makes `palette(id)` total — it can never index out of bounds.
- `THEME` — the UI tokens `bg`, `surface`, `border`, `text`, `accent`,
  `selection`, with values from [[ADR-0015 Kanagawa Dragon theme]] as listed in
  [[Theme]].

## Limits

- No runtime theme switching and no custom colours (deliberate, see ADR-0012).
- Only the six documented colours; changing them needs a new ADR plus edits to
  `core::palette` and [[Theme]].
- All colours are opaque; translucent fills (e.g. marquee) are the renderer's
  choice of alpha on top of a token.
