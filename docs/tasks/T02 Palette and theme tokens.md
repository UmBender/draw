---
id: T02
title: Palette and theme tokens
status: in-progress
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

All items live in `core::palette`. Colour values are exactly those in [[Theme]].

- **AC-1** — `Rgba { r, g, b, a: u8 }` with `const fn from_hex(u32)` (0xRRGGBB, opaque;
  bits above the low 24 are ignored) and `to_f32() -> [f32; 4]` (each channel / 255).
  *Tests:* `palette::tests::from_hex_splits_channels_and_is_opaque`,
  `from_hex_ignores_bits_above_24`, `to_f32_maps_extremes_to_unit_range`,
  `to_f32_scales_each_channel_by_255`.
- **AC-2** — `ColorId(u8)` newtype (field private); `ColorId::new(u8) -> Option<ColorId>`
  rejects `>= PALETTE_LEN` (6); `ColorId::INK` is index 0; `ColorId::index()` returns
  the index as `usize`. *Tests:* `color_id_new_accepts_indices_below_palette_len`,
  proptest `color_id_new_rejects_indices_at_or_above_palette_len`,
  `color_id_ink_is_index_zero`.
- **AC-3** — `palette(id: ColorId) -> Rgba` total (never panics) and the 6 drawing
  colours match [[Theme]] in key order. *Tests:* proptest
  `palette_lookup_total_for_valid_ids`, `palette_matches_documented_drawing_colours`.
- **AC-4** — `Theme { bg, surface, border, text, accent, selection }` and `const THEME: Theme`
  match [[Theme]]. *Test:* `theme_matches_documented_tokens`.
- **AC-5** — `ColorId::from_key_digit(1..=6)` maps keys `1`–`6` to ids 0–5 and returns
  `None` for any other digit. *Tests:* `key_digit_mapping`,
  proptest `key_digit_outside_one_to_six_is_rejected`.

Float comparisons in tests use a local tolerance helper: `geom::approx_eq` belongs
to [[T01 Geometry primitives]], which runs in parallel and is not a dependency.

## Out of scope

Runtime theme switching, colour pickers.

## Files owned

`src/core/palette.rs`

## Subtasks (one commit each)

- [ ] spec · [ ] tests · [ ] models · [ ] behaviour · [ ] quality · [ ] docs

## Learning path

Step 2 — requires step 0.

## Log
