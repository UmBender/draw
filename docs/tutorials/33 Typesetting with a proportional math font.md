---
title: Typesetting with a proportional math font
step: 33
requires: ["[[30 Parsing and laying out a small TeX subset]]", "[[32 Batching glyphs into the document mesh]]"]
feature: "[[TeX formulas]]"
code: ["src/core/math_metrics.rs", "src/core/tex.rs", "src/shell/glyphs.rs", "src/shell/render.rs", "scripts/math_metrics.py"]
tags: [tutorial]
---

# Typesetting with a proportional math font

> **Learning path step 33.** Requires: step 30 (TeX subset), step 32
> (glyph atlas) · Next: —

## Why this matters here

Step 30 typeset formulas with the monospaced text font: every glyph a
0.6 em cell. That made the layout simple — one number for every width —
but formulas looked like code: `x` upright next to `×`, `l` next to `1`.
LaTeX readers expect italic variables and upright operators. Switching to
a real math font means every glyph has its own width and height, and the
layout must learn them **without** `core` ever touching a font file.

## The concept

### Math alphabets are separate characters

Unicode has a block of *Mathematical Alphanumeric Symbols*: `𝑥`
(U+1D465, MATHEMATICAL ITALIC SMALL X) is a different character from
`x`, `𝒜` (U+1D49C) is the script A. A math font draws them, so "italic"
is a **character mapping**, not a font style:

```text
x  → 𝑥   U+1D44E + (x − a)      h → ℎ  U+210E (encoded earlier)
α  → 𝛼   U+1D6FC + (α − α)
A  → 𝒜   U+1D49C + (A − A)      B → ℬ  U+212C (a "hole" in the block)
```

The holes are letters Unicode already had in *Letterlike Symbols*
(`ℎ ℬ ℰ ℱ ℋ ℐ ℒ ℳ ℛ`); the math block leaves their slots empty.

### Metrics: advance, ink box, italic correction

A proportional glyph needs three things for layout:

```text
        ┌────────┐ y_max        advance: where the next glyph starts
        │   𝑓    │──┐           ink box: x_min…x_max, y_min…y_max
 pen ●──┼────────┼──┼─● next    italic correction: how far the ink
        │        │  │             leans past the advance — room a
        └────────┘ y_min           superscript leaves (𝑓² vs 𝑓₂)
           advance  italic
```

TeX puts a superscript after the italic correction and a subscript
right at the advance, so `𝑓²` does not hit the hook of the `𝑓` while
`𝑓ᵢ` tucks under it.

### The MATH table

OpenType math fonts carry a `MATH` table: the axis height, rule
thickness, script shifts and gaps the font was designed for. We read it
once, by hand, and wrote its values as constants (`AXIS`, `RULE`,
`SUP_SHIFT`, `SUB_TOP_MAX`, `FRAC_NUM_SHIFT`, …) — the font never
changes at run time.

## How draw implements it

- **Generated table.** `scripts/math_metrics.py` reads the original font
  with fontTools and rewrites the lines between `BEGIN GENERATED` and
  `END GENERATED` in `src/core/math_metrics.rs`: one row per character of
  the embedded subset, sorted, so `math_metrics::get` is a binary search.
  The hand-written type, lookup and tests around it survive regeneration.
- **Trust, but verify.** `math_metrics_match_the_embedded_font`
  (`src/shell/glyphs.rs`) loads the font with fontdue and compares every
  row. fontdue measures flattened curves, so it is up to 0.0032 em off
  the exact ink box; the test allows 0.005 em.
- **Faces.** `tex::Face` is `Text` or `Math`; every `Ink::Glyph` carries
  one. `MathBox::math` builds a glyph box from the table, and falls back
  to `MathBox::glyph` — a text-face cell — when the math font lacks the
  character, so the layout stays total.
- **Styles.** `layout_styled` threads a `Style` (`Italic` or
  `Calligraphic`) through the tree; `\mathcal{…}` lays out its argument
  in `Calligraphic`. Function names, `\text` and digits go through
  `MathBox::upright`, which does not map.
- **Italic correction.** `MathBox` keeps the italic correction of its
  last glyph; `scripts` places the superscript at `width + italic` and
  the subscript at `width`.
- **Overhangs.** Italic glyphs reach outside their advance (`𝑗` left,
  `𝑓` right). `Ink::bounds` covers both the advance and the ink box, and
  `layout` moves a formula starting a line right by its left overhang and
  widens the line to its rightmost ink — the fuzz invariant "a block's
  box holds its ink" keeps holding.
- **Atlas.** `GlyphAtlas` keys entries by `(face, char, px)` and picks
  the font per face; `with_math` adds the math font. Both faces share
  pages and the single batch of step 32.

## Try it

1. Change `SUP_SHIFT` in `src/core/tex.rs` to `0.5` and run
   `cargo test tex_layout_superscript`. Then draw `$x^2$` and compare.
2. Add `\mathbb` for double-struck capitals (`ℝ ℕ ℤ` are holes again,
   the rest at U+1D538): extend the subset command in
   `assets/fonts/README.md`, regenerate the table, and add a `Style`.
3. Find the widest glyph: `TABLE.iter().max_by(|a, b| a.1.advance.total_cmp(&b.1.advance))`.

## Further reading

- Unicode Technical Report #25, *Unicode Support for Mathematics*.
- Microsoft, *OpenType MATH table* specification.
- Donald Knuth, *The TeXbook*, Appendix G — the rules `core::tex`
  reduces.
