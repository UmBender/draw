---
title: Parsing and laying out a small TeX subset
step: 30
requires: ["[[29 Typing on a canvas without losing the shortcuts]]"]
feature: "[[TeX formulas]]"
code: ["src/core/tex.rs", "src/core/text_layout.rs", "src/shell/render.rs", "tests/fuzz.rs"]
tags: [tutorial]
---

# Parsing and laying out a small TeX subset

> **Learning path step 30.** Requires: step 29 (text blocks) · Next: —

## Why this matters here

Contest notes are full of subscripts and sums: `dp[i][j]` is fine as
text, but `∑_{k=1}^{n} k²` is not. A real TeX engine is huge and a math
crate is a dependency; what a scratch canvas needs is a few dozen
commands, laid out well enough to read. Writing that in-house is a
classic exercise in two parts: a **total parser** (any input gives a
tree, never an error) and a **box layout** (TeX's model, shrunk to fit a
monospaced font).

## The concept

**Lossless parse trees.** A parser that must never fail turns every
problem into a node: an unknown `\foo`, an unmatched `{` or a `^` with
nothing after it becomes a `Literal` holding its source. If every node
also keeps the whitespace it skipped, printing the tree gives the input
back — and that one property, `print(parse(s)) == s`, checked on random
strings, proves the parser neither drops nor invents anything.

Brace matching is done **before** parsing, with a stack: each `{` learns
the index of its `}`, so the recursive descent knows whether a `{` opens
a group or is a literal. The same pass caps nesting depth, so a block of
100 000 `{` cannot overflow the stack.

**Boxes.** TeX lays out everything as boxes with a width, a height above
the baseline (ascent) and a depth below it (descent):

```
          ┌───┐  ← superscript, scale 0.7, baseline raised 0.4 em
  ┌─────┐ │ 2 │
  │  x  │ └───┘
──┼─────┼────────── baseline
  └─────┘ ┌───┐
          │ i │  ← subscript, lowered until 0.1 em clear of the 2
          └───┘
```

A fraction stacks two boxes around a bar on the **math axis** (the middle
of a minus sign); a big operator puts its scripts above and below as
*limits*; a root draws its sign from four strokes. Between atoms, space
depends on their classes: relations (`=`) get more than binary operators
(`+`), and a `+` with nothing on its left is a unary sign, so `-a` stays
tight.

## How draw implements it

- `tex::parse` splits a block into `Segment::Text`, `Segment::Dollar`
  (`\$`) and `Segment::Math`; `Parser` (`list`, `atom`, `script`, `arg`,
  `command`) builds the `Node`s, using `match_braces` for groups.
  `tex::print` is the inverse.
- `tex::layout_math` turns nodes into atoms `(MathBox, Class)`, demotes
  operators with a missing operand, inserts `spacing`, and joins the row.
  `scripts`, `limits`, `fraction`, `binomial` and `root` are each a few
  lines of box arithmetic on the constants at the top of the file.
- `tex::layout` places plain characters in their T29 cells and formulas
  in between, growing a line only when a formula needs it — so a block
  without `$` lays out exactly as before.
- `text_layout::typeset` memoises the layout per text (in em, so it
  serves every size and zoom); `text_layout::bounds` uses it, which is
  how hit-testing, culling and snapping learn the typeset size without
  any change to `Shape`.
- `render::push_text_block` draws each `Ink::Glyph` from the atlas at
  `size × scale` and each `Ink::Rule` as a quad in the same batch.

## Try it

1. In `tex_layout_superscript_is_raised_and_scaled`, change `SUP_SHIFT`
   to 0.5 and watch which tests fail — then look at `$x^2$` in the app.
2. Add `\mid` (`∣`, a relation) to `SYMBOLS`; the font-coverage test in
   `render.rs` tells you whether the glyph exists.
3. Run `cargo test --test fuzz fuzz_tex` and read `arb_formula_text`:
   how would you make the fuzzer generate deeper nesting?

## Further reading

- Donald Knuth, *The TeXbook*, Appendix G — the real rules for
  typesetting formulas, which ADR-T30-1 reduces.
- [[ADR-T30-1 TeX subset, box layout and metrics]]
