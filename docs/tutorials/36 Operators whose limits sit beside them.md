---
title: Operators whose limits sit beside them
step: 36
requires: ["[[30 Parsing and laying out a small TeX subset]]", "[[33 Typesetting with a proportional math font]]"]
feature: "[[TeX formulas]]"
code: ["src/core/tex.rs"]
tags: [tutorial]
---

# Operators whose limits sit beside them

> **Learning path step 36.** Requires: step 30 (TeX subset and box
> layout), step 33 (a proportional math font) · Next: —

## Why this matters here

`\sum_{k=1}^{n}` puts its limits above and below the sign. Written the
same way, `\int_0^1` must not: in inline LaTeX the limits of an integral
are ordinary scripts to its right. Both signs are enlarged, both are
"operators" for spacing — they differ only in where `_` and `^` go. The
integral sign is also slanted, which makes "to its right" two different
places for the two scripts.

## The concept

**1. The atom class decides the layout.** TeX gives every atom a class
(ordinary, operator, binary, relation, …). The class sets the space
between neighbours *and* how scripts attach. An integral is an operator
for spacing (`\int f` gets a thin space, `\int -f` has a unary minus) but
takes scripts like a letter. A new class says that in one place.

**2. The italic correction is a horizontal offset per glyph.** A slanted
glyph's top sticks out past its advance. The font stores how far, the
*italic correction* (0.332 em for `∫` in Latin Modern Math). A script
placed beside a slanted sign uses it twice:

```text
        ┌── superscript at advance + italic
   ╭─╮  b
   │
   │
 ╰─╯ a
     └── subscript at advance − italic (tucked under the slant)
```

The superscript clears the top hook by starting after the correction (as
for `𝑓^2`, step 33). The bottom of the sign curls *left*, so the space
under the slant is free: the OpenType `MATH` rule moves the subscript
left by the same correction.

**3. Scale the correction with the glyph.** The sign is enlarged
(`BIG_OP_SCALE` = 1.3), so the correction is the enlarged glyph's, not
the text-size one. Keeping it in the box (`MathBox::italic`) instead of
re-reading the font keeps one source of truth.

## How draw implements it

- `Class::Integral` in `src/core/tex.rs`; the `SYMBOLS` rows `int`,
  `iint`, `iiint`, `oint`, `oiint` use it
  ([[ADR-T36-1 Integral class with scripts beside the sign]]).
- `symbol_box` sends `BigOp` and `Integral` to `big_operator`, which
  centres the enlarged glyph on the axis and now keeps its italic
  correction (zero for `∑` and `∏`, so they are unchanged).
- In `atom`, a `Node::Scripts` whose base is a `BigOp` or `Limits` goes
  to `limits`; an `Integral` goes to `scripts` with `tuck = base.italic`,
  so the subscript starts at `width − tuck` and the superscript, as for
  every base, at `width + italic`.
- `spacing` and the unary-binary rule treat `Integral` like the other
  operators.
- The physics symbols are plain table rows: a symbol is `(name, text,
  class)`, so adding one is data, not code. Even `\,` is a row — the
  six-per-em space U+2006, 1/6 em wide, TeX's thin space.
- Tests: `tex_layout_integral_is_enlarged_like_sum`,
  `tex_layout_integral_scripts_sit_beside_the_sign`,
  `tex_layout_integral_gets_operator_space`; the bounds proptests type
  `\int` and `\oint`, so the tucked subscript is checked to stay inside
  the box.

## Try it

1. Set `tuck` to `0.0` for integrals in `atom` and type
   `$\int_0^1 x\,dx$`: the `0` floats away from the foot of the sign.
   `tex_layout_integral_scripts_sit_beside_the_sign` fails.
2. Add `\bigcup` (`⋃`, U+22C3) as `Class::BigOp`: one row, and it takes
   limits above and below like `\sum`.
3. Change `\oint` to `Class::BigOp` and compare `$\oint_C$` — that is
   what `\oint\limits_C` would look like in LaTeX.

## Further reading

- *The TeXbook*, Appendix G, rules 13 (operators) and 18 (scripts).
- OpenType `MATH` table specification: italic correction and "limits"
  placement of large operators.
- [[ADR-T33-1 Latin Modern Math for formulas with a metrics table in core]]
