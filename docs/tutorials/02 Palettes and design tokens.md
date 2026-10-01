---
title: Palettes and design tokens
step: 2
requires: ["[[00 Project layout, lints and the TDD loop]]"]
feature: "[[Theme and palette]]"
code: ["src/core/palette.rs"]
tags: [tutorial]
---

# Palettes and design tokens

> **Learning path step 2.** Requires: [[00 Project layout, lints and the TDD loop]] · Next: [[05 Modelling shapes and hit-testing]] (shapes store a `ColorId`), [[07 Rendering with macroquad and culling]] (draws with `THEME`)

## Why this matters here

A paint app touches colour everywhere: the canvas, the toolbar, every stroke,
the selection outline. If each module picks its own RGB values, the UI drifts
and changing the theme means a hunt through the code. `draw` puts every colour
in one small module and lets everything else refer to colours by *name* or by
*index*.

## The concept

### Design tokens

A **design token** is a named role, not a value: `bg`, `surface`, `text`,
`accent`. Code says "draw the toolbar in `surface`", and the theme decides
that `surface` is `#0d0c0c`. Swap the theme and every use follows.

```
code ──uses──▶ token (THEME.accent) ──resolves to──▶ value (#7fa8bc)
```

### Indexed colour

Drawn shapes go one step further: they store a palette **index**
(`ColorId`), like an old 16-colour video mode. A shape is then one byte of
colour instead of four, and recolouring the palette recolours every shape
already on the canvas.

### Make invalid states unrepresentable

An index is only safe if it is in range. Instead of checking at every lookup,
`ColorId` wraps a private `u8` and can only be built through constructors
that check the range. Once you hold a `ColorId`, it is valid — so the lookup
function needs no `Option` and cannot panic. This is "parse, don't validate".

## How draw implements it

All in `src/core/palette.rs`:

- `Rgba::from_hex` is a `const fn`: it shifts and truncates the `u32`
  (`(hex >> 16) as u8` is red, and so on) and sets alpha to `0xff`. Because it
  is `const`, `PALETTE` and `THEME` are computed by the compiler — there is no
  runtime cost and no parsing that could fail mid-contest.
- `Rgba::to_f32` maps each channel through `f32::from(c) / 255.0` with
  `array::map`; `f32::from(u8)` is a lossless conversion, so no `as` cast.
- `ColorId::new` accepts `index < PALETTE_LEN`; `ColorId::from_key_digit`
  reuses it with `digit.wrapping_sub(1)` — key `0` wraps to `255` and is
  rejected by the same check, so there is only one range test in the code.
- `palette(id)` is just `PALETTE[id.index()]`, in bounds by construction.
- Literals are written `0xc5c9c5`, matching `#c5c9c5` in [[Theme]]. Clippy
  read the grouped form `0x18_16_16` as a mistyped `i16` suffix, so the module
  allows `clippy::unreadable_literal` with a `reason` instead.

The tests in the same file show the TDD shape: exact tables from [[Theme]]
for the values, and `proptest` properties for the edges — every `u8` at or
above `PALETTE_LEN` is rejected, every digit outside `1..=6` is rejected, and
the lookup returns an opaque colour for every id that can be built.

## Try it

1. Change `selection` in `THEME` to `0xc8c093` and run `cargo test palette`.
   `theme_matches_documented_tokens` fails: the test is the executable form of
   [[Theme]]. Revert it.
2. Add a seventh colour: bump `PALETTE_LEN` to 7 and add an entry. Which tests
   fail, and which would you update? (Hint: `key_digit_mapping` still only
   covers `1`–`6`.) Then revert — a real change needs an ADR.
3. Try to write `ColorId(9)` outside the module. Why does it not compile?

## Further reading

- [[ADR-0012 Fixed palette and theme tokens]], [[ADR-0015 Kanagawa Dragon theme]]
- Alexis King, *Parse, don't validate* (2019).
- W3C Design Tokens Community Group format.
