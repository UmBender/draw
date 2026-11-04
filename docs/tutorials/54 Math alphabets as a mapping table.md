---
title: Math alphabets as a mapping table
step: 54
requires: ["[[33 Typesetting with a proportional math font]]", "[[39 Accents that sit on their letters]]"]
feature: "[[TeX formulas]]"
code: ["src/core/tex.rs"]
tags: [tutorial]
---

# Math alphabets as a mapping table

> **Learning path step 54.** Requires: step 33 (a proportional math
> font), step 39 (accents that sit on their letters) · Next: —

## Why this matters here

`$\mathrm{d}x$`, `$\mathcal{F}$`, `$\mathnormal{\Gamma}$`: in a formula
the *same* typed letter can be drawn in many alphabets — italic,
upright roman, calligraphic, and later bold, blackboard, fraktur. Each
of them is a function from characters to characters. Written as code,
every new alphabet is another `match` with the same offset arithmetic;
written as **data**, it is one row of a table, and the rules for nesting
and fallback are written once.

## The concept

Unicode has a block for this, *Mathematical Alphanumeric Symbols*
(U+1D400–1D7FF). It is regular: every alphabet is laid out the same way,
in runs per kind of character.

| Kind | Run | Italic starts at |
|------|-----|------------------|
| Latin capitals | `A`…`Z` (26) | U+1D434 `𝐴` |
| Latin lowercase | `a`…`z` (26) | U+1D44E `𝑎` |
| Greek capitals | `Α`…`Ω` (25, plus `∇`) | U+1D6E2 `𝛢` |
| Greek lowercase | `α`…`ω` (25), `∂`, `ϵ ϑ ϰ ϕ ϱ ϖ` | U+1D6FC `𝛼` |
| Digits | `0`…`9` (10) | (italic has none) |

So one alphabet maps a character with **its kind, its index in the run,
and the run's start**: `start + index`. The block has a few *holes*,
characters Unicode already had before the block (the italic `h` is the
Planck constant `ℎ` U+210E; script `B` is `ℬ` U+212C). A short list of
exceptions per alphabet covers them.

Two more ideas make it a style system rather than a lookup:

1. **Not every alphabet maps every kind.** `\mathcal` maps capitals
   only; `\mathrm` maps Latin and digits but not Greek. A missing
   mapping means *"as the enclosing style draws it"*.
2. **Nesting.** In `\mathrm{\mathcal{A} b}` the `A` is calligraphic (the
   innermost alphabet that maps capitals) and the `b` is upright (the
   `\mathcal` does not map lowercase, so the `\mathrm` around it
   decides).

Both rules fall out of one representation: the current style is, *per
kind*, the alphabet that decides it. Entering an alphabet overwrites the
kinds it maps and keeps the others.

```text
formula:              Upper=italic  Lower=italic  Greek↑=upright …
\mathrm{ … }          Upper=rm      Lower=rm      Greek↑=upright …
  \mathcal{ … }       Upper=cal     Lower=rm      Greek↑=upright …
```

## How draw implements it

All of it is in `src/core/tex.rs`
([[ADR-T54-1 Math alphabets as a table of per-kind mappings]]).

- `Kind::of(ch)` returns the kind and the index in its run, or `None` for
  characters no alphabet changes (Hebrew, `∂`, `∞`, operators). The
  unassigned U+03A2 between `Ρ` and `Σ` is skipped explicitly.
- `Map` is plain data: `Inherit`, `Upright` (as typed) or `Block(start)`.
- `AlphabetDef` is a row: a command name, one `Map` per kind and the
  `holes`. `AlphabetDef::map` checks the holes first, then computes
  `start + index` (with `checked_add` and `char::from_u32`, so a bad row
  can never panic — it falls back to the typed character).
- `ALPHABETS` holds `\mathcal`, `\mathrm`, `\mathit`, `\mathnormal`;
  `MATH_ITALIC` is the root row a formula starts in. It maps every kind,
  which is what guarantees that every fallback ends somewhere.
- `Style` is `[&'static AlphabetDef; 5]`: one row per kind. `Style::with`
  overwrites the slots a row maps; `Style::map` asks the row in the
  character's slot. It is `Copy` and five pointers wide, so the layout
  passes it down by value into groups, scripts, fractions and accents —
  no stack, no allocation.
- The parser treats every row as a one-argument command
  (`alphabet(name)` in `Parser::command`), and the layout lays the
  argument out with `style.with(row)`.

The accent code reuses the same question: an accent is skewed over a
letter when `style.map(ch) != ch`, so `\hat{\mathnormal{\Gamma}}` leans
with the italic capital and `\mathrm{\hat{x}}` stays centred.

## Try it

1. Add bold as a row (glyphs permitting): `upper: Map::Block(0x1D400)`,
   `lower: Map::Block(0x1D41A)`, `digit: Map::Block(0x1D7CE)`. Write a
   test that `\mathbf{x}` draws U+1D431. The test fails at
   `math_metrics` — the font subset does not have the glyphs yet, which
   is exactly what T56 has to add.
2. Change `\mathrm`'s `greek_upper` from `Inherit` to `Upright` and run
   `tex_alphabets_nest_innermost_first`: `\mathnormal{\mathrm{\Gamma}}`
   now gives the upright Γ. Which behaviour does LaTeX have, and why?
3. Count the bytes of `Style` with `std::mem::size_of` and compare it to
   a `Vec<&AlphabetDef>` stack.

## Further reading

- Unicode Technical Report #25, *Unicode Support for Mathematics* —
  the layout of the alphanumeric block and its holes.
- The `unicode-math` LaTeX package documentation — the `\mathXX`
  commands and their mapping to Unicode alphabets.
- *The TeXbook*, chapter 17 — math families and why `\mathrm` leaves
  lowercase Greek alone.
