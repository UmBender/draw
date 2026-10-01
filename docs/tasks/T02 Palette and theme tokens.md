---
id: T02
title: Palette and theme tokens
status: ready
wave: 1
branch: task/T02-palette
depends_on: [T00]
adrs: ["[[ADR-0012 Fixed palette and theme tokens]]", "[[ADR-0015 Kanagawa Dragon theme]]"]
feature: "[[Theme and palette]]"
tutorial: "[[02 Palettes and design tokens]]"
tags: [task]
---

# T02 Palette and theme tokens

## Goal

One file holding every colour the app uses, with drawing colours addressed by
index, using the Kanagawa Dragon theme from
[[ADR-0015 Kanagawa Dragon theme]] (values in [[Theme]]).

## Spec

- **AC-1** — `Rgba { r, g, b, a: u8 }` with `const fn from_hex(u32)` (0xRRGGBB, opaque)
  and `to_f32() -> [f32; 4]`. *Tests:* `palette::tests::from_hex_*`, `to_f32_*`.
- **AC-2** — `ColorId(u8)` newtype; `ColorId::new(u8) -> Option<ColorId>` rejects
  `>= PALETTE_LEN` (6); `ColorId::INK` is index 0. *Tests:* `color_id_*`.
- **AC-3** — `palette(id: ColorId) -> Rgba` total (never panics). *Test:* proptest
  `palette_lookup_total_for_valid_ids`.
- **AC-4** — `Theme { bg, surface, border, text, accent, selection }` and `const THEME: Theme`
  match [[Theme]]. *Test:* `theme_matches_documented_tokens`.
- **AC-5** — `ColorId::from_key_digit(1..=6)` maps keys `1`–`6` to ids 0–5. *Test:* `key_digit_mapping`.

## Out of scope

Runtime theme switching, colour pickers.

## Files owned

`src/core/palette.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 2 — requires step 0.

## Log
