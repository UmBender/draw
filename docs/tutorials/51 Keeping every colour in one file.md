---
title: Keeping every colour in one file
step: 51
requires: ["[[02 Palettes and design tokens]]", "[[13 Property-based testing and fuzzing]]"]
feature: "[[Themes]]"
code: ["src/theme.rs", "src/core/palette.rs", "src/shell/render.rs"]
tags: [tutorial]
---

# Keeping every colour in one file

> **Learning path step 51.** Requires: step 02 (palettes and design
> tokens), step 13 (property-based testing) · Next: —

## Why this matters here

Step 02 put the colours in `core::palette` as *design tokens*: `bg`,
`accent`, `selection`. Over time the shell grew colours of its own — an
alpha here, a "use the background colour for text on a fill" there — and
one token (`accent`) ended up colouring four unrelated things. Someone
who wants draw to match their desktop had to read the whole renderer.
T51 moves every colour into `src/theme.rs` and adds a test so it stays
that way.

## The concept

**Tokens named by place, not by role.** A role token (`accent`) is
convenient for a designer but couples places a user may want apart.
Naming by place (`SELECTION_OUTLINE`, `GUIDE`, `MARQUEE_OUTLINE`,
`TOOLBAR_ACTIVE`) costs a few duplicate values and buys independent
control.

**Derived colours keep their rule, not their value.** "Text on a filled
shape is drawn in the background colour" becomes "…in `TEXT_ON_FILL`":
the rule lives in code, its input in the theme.

**Contrast, measured.** WCAG defines the relative luminance of an sRGB
colour (linearise each channel, then `0.2126 R + 0.7152 G + 0.0722 B`)
and the contrast ratio `(L1 + 0.05) / (L2 + 0.05)`, from 1:1 (same
colour) to 21:1 (black on white). 3:1 is the usual floor for graphics and
large text.

**A guard test.** A convention ("colours only in the theme") decays
unless something checks it. A test that scans the source turns it into a
rule the compiler-adjacent tooling enforces.

## How draw implements it

- `src/theme.rs` — only `pub const`s, `0xRRGGBB` plus separate `u8`
  alphas.
- `Rgba::from_hex_alpha`, `Rgba::faded` (integer alpha scaling: opaque →
  exactly `alpha`), `Rgba::relative_luminance` and `contrast_ratio` in
  `src/core/palette.rs`; `contrast_ratio_symmetric_and_bounded` is a
  proptest.
- `render::hex_color` / `hex_color_alpha` are the only way the shell
  turns a theme constant into a macroquad `Color`.
- The old `f32` factors (`0.25`, `0.35`, `0.3`) became bytes (`63`,
  `89`, `76`); `render::tests::theme_alphas_give_previous_vertex_bytes`
  proves the vertex bytes sent to the GPU did not change.
- `theme::tests::colour_literals` is a small line scanner: it skips
  comments and `#[cfg(test)]` blocks and flags hex colours, colour
  constructors with numeric arguments, `Color { … }` / `Rgba { … }`
  literals, `.a *=` style alpha edits and macroquad named colours.

## Try it

1. Set `CANVAS` to `0xf2ecbc` and run `cargo test theme`: watch
   `ink_contrasts_with_canvas` fail and print the ratio. Darken `INK`
   until it passes.
2. Add `let c = macroquad::color::RED;` to any shell function and run
   `cargo test guard`: the guard names the file and line.

## Further reading

- WCAG 2.2, *contrast ratio* and *relative luminance* definitions.
- [[ADR-T51-1 One theme file with every colour]]
